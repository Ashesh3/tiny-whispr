<script lang="ts">
  import type { Transcription } from "../types";

  let { item, onDelete, onCopy }: {
    item: Transcription;
    onDelete: (id: number) => void;
    onCopy: (text: string) => void;
  } = $props();

  function formatTimestamp(dateStr: string): string {
    const date = new Date(dateStr + "Z"); // SQLite stores UTC without Z suffix
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);
    const itemDate = new Date(date.getFullYear(), date.getMonth(), date.getDate());

    const timeStr = date.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });

    if (itemDate.getTime() === today.getTime()) {
      return `Today, ${timeStr}`;
    } else if (itemDate.getTime() === yesterday.getTime()) {
      return `Yesterday, ${timeStr}`;
    } else {
      return date.toLocaleDateString([], { month: "short", day: "numeric", year: "numeric" }) + `, ${timeStr}`;
    }
  }
</script>

<div class="bg-surface border border-border rounded-lg p-3 space-y-2 group">
  <div class="flex items-start justify-between gap-2">
    <span class="text-[11px] text-text-muted">{formatTimestamp(item.created_at)}</span>
    <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
      <button
        class="px-2 py-0.5 text-[11px] text-text-muted hover:text-text-primary bg-transparent border border-border rounded hover:border-accent transition-colors cursor-pointer"
        onclick={() => onCopy(item.text)}
        title="Copy text"
      >
        Copy
      </button>
      <button
        class="px-2 py-0.5 text-[11px] text-text-muted hover:text-red-400 bg-transparent border border-border rounded hover:border-red-400 transition-colors cursor-pointer"
        onclick={() => onDelete(item.id)}
        title="Delete"
      >
        Delete
      </button>
    </div>
  </div>
  <p class="text-[13px] text-text-primary line-clamp-2 m-0">{item.text}</p>
</div>
