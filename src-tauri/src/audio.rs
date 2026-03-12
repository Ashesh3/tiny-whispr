use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const TARGET_SAMPLE_RATE: u32 = 16_000;
const MAX_DURATION_SECS: f64 = 300.0;
const RESAMPLE_CHUNK_SIZE: usize = 1024;

pub struct AudioRecorder {
    is_recording: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(AtomicBool::new(false)),
            samples: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(TARGET_SAMPLE_RATE)),
        }
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn start_recording(&self) -> Result<(), String> {
        if self.is_recording() {
            return Err("Already recording".to_string());
        }

        // Clear samples buffer before starting
        {
            let mut buf = self.samples.lock().map_err(|e| format!("Lock error: {e}"))?;
            buf.clear();
        }

        let is_recording = Arc::clone(&self.is_recording);
        let samples = Arc::clone(&self.samples);
        let sample_rate_shared = Arc::clone(&self.sample_rate);

        // Use a channel to communicate any startup errors back to the caller
        let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();

        // Build and play stream on a dedicated thread so Stream never crosses thread boundaries
        std::thread::spawn(move || {
            let host = cpal::default_host();
            let device = match host.default_input_device() {
                Some(d) => d,
                None => {
                    let _ = tx.send(Err("No input device available".to_string()));
                    return;
                }
            };

            let config = match device.default_input_config() {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to get default input config: {e}")));
                    return;
                }
            };

            let native_rate = config.sample_rate().0;
            let channels = config.channels() as usize;

            // Store the native sample rate
            if let Ok(mut rate) = sample_rate_shared.lock() {
                *rate = native_rate;
            }

            // Calculate max samples for the 5-minute limit
            let max_samples = (MAX_DURATION_SECS * native_rate as f64) as usize;

            let is_recording_cb = Arc::clone(&is_recording);
            let samples_cb = Arc::clone(&samples);

            let stream = match device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if !is_recording_cb.load(Ordering::SeqCst) {
                        return;
                    }
                    let mut buf = match samples_cb.lock() {
                        Ok(b) => b,
                        Err(_) => return,
                    };

                    // Mix to mono
                    for frame in data.chunks(channels) {
                        let mono: f32 = frame.iter().sum::<f32>() / channels as f32;
                        buf.push(mono);
                    }

                    // Enforce max duration
                    if buf.len() >= max_samples {
                        is_recording_cb.store(false, Ordering::SeqCst);
                    }
                },
                move |err| {
                    eprintln!("Audio stream error: {err}");
                },
                None,
            ) {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to build input stream: {e}")));
                    return;
                }
            };

            if let Err(e) = stream.play() {
                let _ = tx.send(Err(format!("Failed to start audio stream: {e}")));
                return;
            }

            // Signal success to the caller
            is_recording.store(true, Ordering::SeqCst);
            let _ = tx.send(Ok(()));

            // Keep stream alive by holding ownership until recording stops
            while is_recording.load(Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            // Stream drops here when recording stops
            drop(stream);
        });

        // Wait for the background thread to report success or error
        rx.recv()
            .map_err(|_| "Recording thread terminated unexpectedly".to_string())?
    }

    pub fn stop_recording(&self) -> Result<Vec<u8>, String> {
        if !self.is_recording() {
            return Err("Not recording".to_string());
        }

        self.is_recording.store(false, Ordering::SeqCst);

        // Wait briefly for stream to flush
        std::thread::sleep(std::time::Duration::from_millis(100));

        let samples = {
            let buf = self.samples.lock().map_err(|e| format!("Lock error: {e}"))?;
            buf.clone()
        };

        let native_rate = {
            let rate = self.sample_rate.lock().map_err(|e| format!("Lock error: {e}"))?;
            *rate
        };

        if samples.is_empty() {
            return Err("No audio data captured".to_string());
        }

        // Resample to 16kHz if needed
        let resampled = if native_rate != TARGET_SAMPLE_RATE {
            resample(&samples, native_rate, TARGET_SAMPLE_RATE)?
        } else {
            samples
        };

        // Encode to WAV
        let wav_bytes = encode_wav(&resampled, TARGET_SAMPLE_RATE)?;

        Ok(wav_bytes)
    }

    /// Returns the current recording duration in milliseconds.
    pub fn duration_ms(&self) -> u64 {
        let samples_len = match self.samples.lock() {
            Ok(buf) => buf.len(),
            Err(_) => return 0,
        };
        let rate = match self.sample_rate.lock() {
            Ok(r) => *r,
            Err(_) => return 0,
        };
        if rate == 0 {
            return 0;
        }
        (samples_len as u64 * 1000) / rate as u64
    }
}

/// Resamples mono f32 audio from `from_rate` to `to_rate` using rubato.
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>, String> {
    use rubato::{FastFixedIn, PolynomialDegree, Resampler};

    let ratio = to_rate as f64 / from_rate as f64;

    let mut resampler = FastFixedIn::<f32>::new(
        ratio,
        1.0, // max relative ratio (not variable)
        PolynomialDegree::Cubic,
        RESAMPLE_CHUNK_SIZE,
        1, // mono
    )
    .map_err(|e| format!("Failed to create resampler: {e}"))?;

    let mut output: Vec<f32> = Vec::new();
    let mut pos = 0;

    while pos < samples.len() {
        let end = (pos + RESAMPLE_CHUNK_SIZE).min(samples.len());
        let mut chunk = samples[pos..end].to_vec();

        let is_last = end >= samples.len();

        // Pad the last chunk if it's smaller than chunk_size
        if chunk.len() < RESAMPLE_CHUNK_SIZE {
            chunk.resize(RESAMPLE_CHUNK_SIZE, 0.0);
        }

        let input = vec![chunk];
        let result = if is_last {
            resampler
                .process_partial(Some(&input), None)
                .map_err(|e| format!("Resampling error: {e}"))?
        } else {
            resampler
                .process(&input, None)
                .map_err(|e| format!("Resampling error: {e}"))?
        };

        if let Some(channel) = result.first() {
            output.extend_from_slice(channel);
        }

        pos += RESAMPLE_CHUNK_SIZE;
    }

    // Handle any remaining samples in the resampler
    if let Ok(result) = resampler.process_partial(None::<&[Vec<f32>]>, None) {
        if let Some(channel) = result.first() {
            output.extend_from_slice(channel);
        }
    }

    Ok(output)
}

/// Encodes mono f32 samples as 16-bit PCM WAV bytes.
fn encode_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| format!("Failed to create WAV writer: {e}"))?;

        for &sample in samples {
            // Clamp to [-1.0, 1.0] and convert to i16
            let clamped = sample.clamp(-1.0, 1.0);
            let value = (clamped * i16::MAX as f32) as i16;
            writer
                .write_sample(value)
                .map_err(|e| format!("Failed to write WAV sample: {e}"))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV: {e}"))?;
    }

    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_wav_produces_valid_header() {
        let samples = vec![0.0f32; 16000]; // 1 second of silence
        let wav = encode_wav(&samples, 16000).unwrap();

        // WAV files start with "RIFF"
        assert_eq!(&wav[0..4], b"RIFF");
        // Format should be "WAVE"
        assert_eq!(&wav[8..12], b"WAVE");
        // Should have reasonable length (header + 16000 samples * 2 bytes)
        assert!(wav.len() > 44); // WAV header is 44 bytes
    }

    #[test]
    fn test_encode_wav_clamps_values() {
        let samples = vec![-2.0, 2.0, 0.5, -0.5];
        let wav = encode_wav(&samples, 16000).unwrap();
        assert!(!wav.is_empty());
    }

    #[test]
    fn test_audio_recorder_initial_state() {
        let recorder = AudioRecorder::new();
        assert!(!recorder.is_recording());
        assert_eq!(recorder.duration_ms(), 0);
    }

    #[test]
    fn test_resample_downsample() {
        // Resampling from 48kHz to 16kHz should produce ~1/3 the samples
        let num_samples = 48000; // 1 second at 48kHz
        let samples: Vec<f32> = (0..num_samples).map(|i| (i as f32 * 0.001).sin()).collect();
        let result = resample(&samples, 48000, 16000).unwrap();
        let expected = 16000; // 1 second at 16kHz
        let ratio = result.len() as f64 / expected as f64;
        assert!(
            (0.9..1.2).contains(&ratio),
            "Unexpected length ratio: {ratio} (got {} samples, expected ~{expected})",
            result.len()
        );
    }

    #[test]
    fn test_resample_upsample() {
        // Resampling from 8kHz to 16kHz should produce ~2x the samples
        let num_samples = 8000; // 1 second at 8kHz
        let samples: Vec<f32> = (0..num_samples).map(|i| (i as f32 * 0.001).sin()).collect();
        let result = resample(&samples, 8000, 16000).unwrap();
        let expected = 16000; // 1 second at 16kHz
        let ratio = result.len() as f64 / expected as f64;
        assert!(
            (0.9..1.2).contains(&ratio),
            "Unexpected length ratio: {ratio} (got {} samples, expected ~{expected})",
            result.len()
        );
    }
}
