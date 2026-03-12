<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let state = $state<"hidden" | "recording" | "processing">("hidden");
  const appWindow = getCurrentWindow();

  listen("recording-started", () => {
    state = "recording";
    appWindow.show();
  });

  listen("recording-stopped", () => {
    state = "processing";
  });

  listen("transcription-complete", () => {
    state = "hidden";
    appWindow.hide();
  });

  listen("transcription-error", () => {
    state = "hidden";
    appWindow.hide();
  });
</script>

<div
  class="indicator"
  class:recording={state === "recording"}
  class:processing={state === "processing"}
  class:visible={state !== "hidden"}
>
  {#if state === "recording"}
    <span class="dot recording-dot"></span>
    <span class="label">REC</span>
  {:else if state === "processing"}
    <span class="spinner"></span>
    <span class="label">...</span>
  {/if}
</div>

<style>
  .indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    border-radius: 20px;
    font-family: system-ui, sans-serif;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.5px;
    backdrop-filter: blur(8px);
    user-select: none;
    opacity: 0;
    transform: translateY(-4px);
    transition: opacity 0.2s ease, transform 0.2s ease;
    pointer-events: none;
  }

  .indicator.visible {
    opacity: 1;
    transform: translateY(0);
  }

  .recording {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #fca5a5;
  }

  .processing {
    background: rgba(59, 130, 246, 0.15);
    border: 1px solid rgba(59, 130, 246, 0.4);
    color: #93c5fd;
  }

  .recording-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #ef4444;
    box-shadow: 0 0 8px rgba(239, 68, 68, 0.6);
    animation: pulse 1.5s ease-in-out infinite;
  }

  .spinner {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 2px solid transparent;
    border-top-color: #3b82f6;
    animation: spin 0.8s linear infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
