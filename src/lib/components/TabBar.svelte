<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let { activeTab = $bindable("settings") }: { activeTab: string } = $props();

  async function hideWindow() {
    await getCurrentWindow().hide();
  }
</script>

<div class="flex items-center bg-surface border-b border-border select-none" data-tauri-drag-region>
  <button
    class="px-4 py-2.5 text-[13px] font-medium border-b-2 transition-colors cursor-pointer bg-transparent {activeTab === 'settings' ? 'text-text-primary border-accent' : 'text-text-muted border-transparent hover:text-text-secondary'}"
    onclick={() => activeTab = "settings"}
  >
    Settings
  </button>
  <button
    class="px-4 py-2.5 text-[13px] font-medium border-b-2 transition-colors cursor-pointer bg-transparent {activeTab === 'history' ? 'text-text-primary border-accent' : 'text-text-muted border-transparent hover:text-text-secondary'}"
    onclick={() => activeTab = "history"}
  >
    History
  </button>
  <div class="flex-1" data-tauri-drag-region></div>
  <button
    class="px-3 py-2 text-text-muted hover:text-text-primary text-[14px] cursor-pointer bg-transparent border-none transition-colors"
    onclick={hideWindow}
    title="Close"
  >
    &#10005;
  </button>
</div>
