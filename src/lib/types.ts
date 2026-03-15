export interface Settings {
  provider: string;
  api_key: string;
  base_url: string;
  model: string;
  language: string;
  hotkey: string;
  activation_mode: string;
  output_mode: string;
  launch_at_startup: boolean;
  microphone_device_id: string;
}

export interface Transcription {
  id: number;
  text: string;
  provider: string;
  model: string;
  language: string | null;
  duration_ms: number | null;
  created_at: string;
}

export interface InputDevice {
  id: string;
  name: string;
  is_default: boolean;
}

export interface ProviderPreset {
  name: string;
  base_url: string;
  models: string[];
}

export const PROVIDER_PRESETS: Record<string, ProviderPreset> = {
  openai: {
    name: "OpenAI",
    base_url: "https://api.openai.com/v1",
    models: ["whisper-1", "gpt-4o-mini-transcribe", "gpt-4o-transcribe"],
  },
  groq: {
    name: "Groq",
    base_url: "https://api.groq.com/openai/v1",
    models: ["whisper-large-v3", "whisper-large-v3-turbo"],
  },
  custom: {
    name: "Custom",
    base_url: "",
    models: [],
  },
};

export type RecordingState = "idle" | "recording" | "processing";
