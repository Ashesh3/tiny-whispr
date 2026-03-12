import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { Transcription } from "../types";

export const history = writable<Transcription[]>([]);

export async function loadHistory() {
  const items = await invoke<Transcription[]>("get_history");
  history.set(items);
}

export async function deleteTranscription(id: number) {
  await invoke("delete_transcription", { id });
  await loadHistory();
}

export async function clearHistory() {
  await invoke("clear_history");
  history.set([]);
}

export async function searchHistory(query: string) {
  if (!query.trim()) {
    await loadHistory();
    return;
  }
  const items = await invoke<Transcription[]>("search_history", { query });
  history.set(items);
}
