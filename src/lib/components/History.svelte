<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { history, loadHistory, deleteTranscription, clearHistory, searchHistory } from "../stores/history";
  import HistoryItem from "./HistoryItem.svelte";
  import type { Transcription } from "../types";

  let items = $state<Transcription[]>([]);
  let searchQuery = $state("");
  let confirmClear = $state(false);
  let confirmTimeout: ReturnType<typeof setTimeout> | null = null;
  let unlisten: UnlistenFn | null = null;

  const unsubscribe = history.subscribe((h) => {
    items = h;
  });

  onMount(async () => {
    await loadHistory();
    unlisten = await listen("transcription-complete", () => {
      loadHistory();
    });
  });

  onDestroy(() => {
    unsubscribe();
    if (unlisten) unlisten();
    if (confirmTimeout) clearTimeout(confirmTimeout);
  });

  async function handleSearch() {
    await searchHistory(searchQuery);
  }

  async function handleDelete(id: number) {
    await deleteTranscription(id);
  }

  async function handleCopy(text: string) {
    try {
      await invoke("copy_to_clipboard", { text });
    } catch {
      // Fallback silently
    }
  }

  async function handleClearAll() {
    if (!confirmClear) {
      confirmClear = true;
      confirmTimeout = setTimeout(() => {
        confirmClear = false;
      }, 3000);
      return;
    }
    if (confirmTimeout) clearTimeout(confirmTimeout);
    confirmClear = false;
    await clearHistory();
  }

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  function onSearchInput() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      handleSearch();
    }, 300);
  }
</script>

<div class="flex flex-col h-full overflow-hidden">
  <!-- Search bar -->
  <div class="p-3 border-b border-border">
    <input
      type="text"
      class="w-full bg-surface border border-border rounded-md px-3 py-2 text-[13px] text-text-primary outline-none focus:border-accent placeholder:text-text-muted"
      placeholder="Search transcriptions..."
      bind:value={searchQuery}
      oninput={onSearchInput}
    />
  </div>

  <!-- List -->
  <div class="flex-1 overflow-y-auto p-3 space-y-2">
    {#if items.length === 0}
      <div class="flex items-center justify-center h-full">
        <p class="text-text-muted text-[13px]">
          {searchQuery ? "No matching transcriptions found." : "No transcriptions yet. Use the hotkey to start recording."}
        </p>
      </div>
    {:else}
      {#each items as item (item.id)}
        <HistoryItem {item} onDelete={handleDelete} onCopy={handleCopy} />
      {/each}
    {/if}
  </div>

  <!-- Clear all -->
  {#if items.length > 0}
    <div class="p-3 border-t border-border">
      <button
        class="w-full py-2 text-[12px] rounded-md border cursor-pointer transition-colors {confirmClear ? 'bg-red-900/30 border-red-700/50 text-red-300 hover:bg-red-900/50' : 'bg-transparent border-border text-text-muted hover:text-text-secondary hover:border-accent'}"
        onclick={handleClearAll}
      >
        {confirmClear ? "Click again to confirm" : "Clear All History"}
      </button>
    </div>
  {/if}
</div>
