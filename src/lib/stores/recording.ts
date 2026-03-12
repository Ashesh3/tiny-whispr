import { writable } from "svelte/store";
import type { RecordingState } from "../types";

export const recordingState = writable<RecordingState>("idle");
