use serde::Deserialize;
use std::fmt;

/// Holds the result of a successful transcription.
#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
}

/// Errors that can occur during transcription.
#[derive(Debug)]
pub enum TranscriptionError {
    Network(String),
    InvalidApiKey,
    RateLimited,
    ApiError(String),
    EmptyResult,
    Request(String),
}

impl fmt::Display for TranscriptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranscriptionError::Network(msg) => write!(f, "Network error: {msg}"),
            TranscriptionError::InvalidApiKey => write!(f, "Invalid API key"),
            TranscriptionError::RateLimited => write!(f, "Rate limited — please try again later"),
            TranscriptionError::ApiError(msg) => write!(f, "API error: {msg}"),
            TranscriptionError::EmptyResult => write!(f, "Transcription returned empty text"),
            TranscriptionError::Request(msg) => write!(f, "Request error: {msg}"),
        }
    }
}

impl std::error::Error for TranscriptionError {}

#[derive(Deserialize)]
struct ApiResponse {
    text: Option<String>,
}

#[derive(Deserialize)]
struct ApiErrorResponse {
    error: Option<ApiErrorDetail>,
}

#[derive(Deserialize)]
struct ApiErrorDetail {
    message: Option<String>,
}

/// Sends audio data to a Whisper-compatible transcription API.
///
/// - `base_url`: The API base URL (already includes `/v1`), e.g. `https://api.openai.com/v1`
/// - `api_key`: Bearer token for authentication
/// - `model`: Model name, e.g. `whisper-1`
/// - `language`: Language code or `"auto"` to omit the language parameter
/// - `audio_data`: WAV file bytes
pub async fn transcribe(
    base_url: &str,
    api_key: &str,
    model: &str,
    language: &str,
    audio_data: Vec<u8>,
) -> Result<TranscriptionResult, TranscriptionError> {
    let url = format!("{}/audio/transcriptions", base_url.trim_end_matches('/'));

    let client = reqwest::Client::new();

    let file_part = reqwest::multipart::Part::bytes(audio_data)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| TranscriptionError::Request(format!("Failed to create file part: {e}")))?;

    let mut form = reqwest::multipart::Form::new()
        .part("file", file_part)
        .text("model", model.to_string())
        .text("response_format", "json".to_string());

    if language != "auto" {
        form = form.text("language", language.to_string());
    }

    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                TranscriptionError::Network(e.to_string())
            } else {
                TranscriptionError::Request(e.to_string())
            }
        })?;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();

        return match status.as_u16() {
            401 | 403 => Err(TranscriptionError::InvalidApiKey),
            429 => Err(TranscriptionError::RateLimited),
            _ => {
                // Try to parse error message from API response
                let msg = if let Ok(err_resp) = serde_json::from_str::<ApiErrorResponse>(&body) {
                    err_resp
                        .error
                        .and_then(|e| e.message)
                        .unwrap_or_else(|| format!("HTTP {status}: {body}"))
                } else {
                    format!("HTTP {status}: {body}")
                };
                Err(TranscriptionError::ApiError(msg))
            }
        };
    }

    let body = response
        .text()
        .await
        .map_err(|e| TranscriptionError::Network(format!("Failed to read response body: {e}")))?;

    let api_response: ApiResponse = serde_json::from_str(&body)
        .map_err(|e| TranscriptionError::ApiError(format!("Failed to parse response: {e}")))?;

    let text = api_response.text.unwrap_or_default().trim().to_string();

    if text.is_empty() {
        return Err(TranscriptionError::EmptyResult);
    }

    Ok(TranscriptionResult { text })
}
