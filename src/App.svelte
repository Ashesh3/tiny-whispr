<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { get } from "svelte/store";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import TabBar from "./lib/components/TabBar.svelte";
  import Settings from "./lib/components/Settings.svelte";
  import History from "./lib/components/History.svelte";
  import { recordingState } from "./lib/stores/recording";

  let activeTab = $state("settings");
  let cleanups: UnlistenFn[] = [];

  async function startRecording() {
    if (get(recordingState) !== "idle") {
      return;
    }

    recordingState.set("recording");
    try {
      await invoke("start_recording");
    } catch {
      recordingState.set("idle");
    }
  }

  async function stopRecording() {
    if (get(recordingState) !== "recording") {
      return;
    }

    recordingState.set("processing");
    try {
      await invoke("stop_recording");
    } catch {
      recordingState.set("idle");
    }
  }

  onMount(async () => {
    try {
      const initialState = await invoke<"idle" | "recording">("get_recording_state");
      recordingState.set(initialState);
    } catch {
      recordingState.set("idle");
    }

    cleanups.push(
      await listen<string>("navigate", (event) => {
        activeTab = event.payload;
      }),

      await listen("tray-toggle-recording", async () => {
        const currentState = get(recordingState);

        if (currentState === "idle") {
          await startRecording();
        } else if (currentState === "recording") {
          await stopRecording();
        }
      }),

      await listen("shortcut-start-recording", async () => {
        await startRecording();
      }),

      await listen("shortcut-stop-recording", async () => {
        await stopRecording();
      }),

      await listen("recording-started", () => {
        recordingState.set("recording");
      }),

      await listen("recording-stopped", () => {
        recordingState.set("processing");
      }),

      await listen("transcription-complete", () => {
        recordingState.set("idle");
      }),

      await listen("transcription-error", () => {
        recordingState.set("idle");
      }),
    );
  });

  onDestroy(() => cleanups.forEach((fn) => fn()));
</script>

<main class="bg-background text-text-primary h-screen font-sans text-[13px] flex flex-col overflow-hidden">
  <TabBar bind:activeTab />
  <div class="flex-1 overflow-hidden">
    {#if activeTab === "settings"}
      <Settings />
    {:else}
      <History />
    {/if}
  </div>
</main>
