<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import TabBar from "./lib/components/TabBar.svelte";
  import Settings from "./lib/components/Settings.svelte";
  import History from "./lib/components/History.svelte";
  import { recordingState } from "./lib/stores/recording";

  let activeTab = $state("settings");
  let unlistenNavigate: UnlistenFn | null = null;
  let unlistenTray: UnlistenFn | null = null;
  let unlistenClose: (() => void) | null = null;
  let unlistenComplete: UnlistenFn | null = null;
  let unlistenError: UnlistenFn | null = null;

  onMount(async () => {
    // Listen for tray navigation events
    unlistenNavigate = await listen<string>("navigate", (event) => {
      activeTab = event.payload;
    });

    // Listen for tray toggle recording
    unlistenTray = await listen("tray-toggle-recording", async () => {
      let currentState: string;
      const unsub = recordingState.subscribe((s) => (currentState = s));
      unsub();

      if (currentState! === "idle") {
        recordingState.set("recording");
        try {
          await invoke("start_recording");
        } catch {
          recordingState.set("idle");
        }
      } else if (currentState! === "recording") {
        recordingState.set("processing");
        try {
          await invoke("stop_recording");
        } catch {
          recordingState.set("idle");
        }
      }
    });

    // Reset state when transcription finishes or fails
    unlistenComplete = await listen("transcription-complete", () => {
      recordingState.set("idle");
    });

    unlistenError = await listen("transcription-error", () => {
      recordingState.set("idle");
    });

    // Hide window on close instead of quitting
    const appWindow = getCurrentWindow();
    unlistenClose = await appWindow.onCloseRequested(async (event) => {
      event.preventDefault();
      await appWindow.hide();
    });
  });

  onDestroy(() => {
    if (unlistenNavigate) unlistenNavigate();
    if (unlistenTray) unlistenTray();
    if (unlistenComplete) unlistenComplete();
    if (unlistenError) unlistenError();
    if (unlistenClose) unlistenClose();
  });
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
