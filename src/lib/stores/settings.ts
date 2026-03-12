import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types";

export const settings = writable<Settings | null>(null);
export const settingsError = writable<string | null>(null);

export async function loadSettings() {
  try {
    const s = await invoke<Settings>("get_settings");
    settings.set(s);
    settingsError.set(null);
  } catch (e) {
    settingsError.set(String(e));
  }
}

export async function saveSettings(s: Settings) {
  try {
    const saved = await invoke<Settings>("save_settings", { settings: s });
    settings.set(saved);
    settingsError.set(null);
  } catch (e) {
    settingsError.set(String(e));
    throw e;
  }
}
