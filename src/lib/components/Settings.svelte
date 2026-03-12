<script lang="ts">
  import { onMount } from "svelte";
  import { settings, settingsError, loadSettings, saveSettings } from "../stores/settings";
  import { PROVIDER_PRESETS, type Settings } from "../types";

  let form = $state<Settings>({
    provider: "openai",
    api_key: "",
    base_url: "https://api.openai.com/v1",
    model: "whisper-1",
    language: "auto",
    hotkey: "Ctrl+Shift+Space",
    activation_mode: "toggle",
    output_mode: "auto_paste",
    launch_at_startup: false,
  });

  let saving = $state(false);
  let saveSuccess = $state(false);
  let capturingHotkey = $state(false);
  let error = $state<string | null>(null);
  let hotkeyEl = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (capturingHotkey && hotkeyEl) {
      hotkeyEl.focus();
    }
  });

  const languages = [
    { value: "auto", label: "Auto-detect" },
    { value: "en", label: "English" },
    { value: "es", label: "Spanish" },
    { value: "fr", label: "French" },
    { value: "de", label: "German" },
    { value: "it", label: "Italian" },
    { value: "pt", label: "Portuguese" },
    { value: "nl", label: "Dutch" },
    { value: "ja", label: "Japanese" },
    { value: "ko", label: "Korean" },
    { value: "zh", label: "Chinese" },
    { value: "ru", label: "Russian" },
    { value: "ar", label: "Arabic" },
    { value: "hi", label: "Hindi" },
    { value: "pl", label: "Polish" },
    { value: "uk", label: "Ukrainian" },
    { value: "sv", label: "Swedish" },
    { value: "da", label: "Danish" },
    { value: "fi", label: "Finnish" },
    { value: "no", label: "Norwegian" },
  ];

  const currentPreset = $derived(PROVIDER_PRESETS[form.provider]);
  const isCustomProvider = $derived(form.provider === "custom");

  // Sync form from store when settings load
  const unsubscribe = settings.subscribe((s) => {
    if (s) {
      form = { ...s };
    }
  });

  onMount(() => {
    loadSettings();
    return () => {
      unsubscribe();
    };
  });

  function onProviderChange() {
    const preset = PROVIDER_PRESETS[form.provider];
    if (preset) {
      form.base_url = preset.base_url;
      if (preset.models.length > 0) {
        form.model = preset.models[0];
      } else {
        form.model = "";
      }
    }
  }

  function handleHotkeyKeydown(e: KeyboardEvent) {
    e.preventDefault();
    e.stopPropagation();

    const modifiers: string[] = [];
    if (e.ctrlKey) modifiers.push("Ctrl");
    if (e.altKey) modifiers.push("Alt");
    if (e.shiftKey) modifiers.push("Shift");

    const modifierKeys = ["Control", "Alt", "Shift", "Meta"];
    const key = e.key;

    if (modifierKeys.includes(key)) {
      // Only modifier pressed, wait for a non-modifier
      return;
    }

    if (modifiers.length === 0) {
      // Needs at least one modifier
      return;
    }

    // Map common key names
    let keyName = key;
    if (key === " ") keyName = "Space";
    else if (key.length === 1) keyName = key.toUpperCase();

    const combo = [...modifiers, keyName].join("+");
    form.hotkey = combo;
    capturingHotkey = false;
  }

  async function handleSave() {
    saving = true;
    error = null;
    saveSuccess = false;
    try {
      await saveSettings(form);
      saveSuccess = true;
      setTimeout(() => (saveSuccess = false), 2000);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="flex flex-col h-full overflow-y-auto p-4 space-y-4">
  {#if !form.api_key || form.api_key === ""}
    <div class="bg-yellow-900/30 border border-yellow-700/50 rounded-lg px-3 py-2 text-yellow-200 text-[12px]">
      No API key configured. Set an API key to enable transcription.
    </div>
  {/if}

  {#if error}
    <div class="bg-red-900/30 border border-red-700/50 rounded-lg px-3 py-2 text-red-300 text-[12px]">
      {error}
    </div>
  {/if}

  {#if saveSuccess}
    <div class="bg-green-900/30 border border-green-700/50 rounded-lg px-3 py-2 text-green-300 text-[12px]">
      Settings saved successfully.
    </div>
  {/if}

  <!-- Provider -->
  <div class="space-y-1.5">
    <label class="text-[12px] text-text-secondary font-medium" for="provider">Provider</label>
    <select
      id="provider"
      class="w-full bg-surface border border-border rounded-md px-3 py-2 text-[13px] text-text-primary outline-none focus:border-accent"
      bind:value={form.provider}
      onchange={onProviderChange}
    >
      {#each Object.entries(PROVIDER_PRESETS) as [key, preset]}
        <option value={key}>{preset.name}</option>
      {/each}
    </select>
  </div>

  <!-- API Key -->
  <div class="space-y-1.5">
    <label class="text-[12px] text-text-secondary font-medium" for="api_key">API Key</label>
    <input
      id="api_key"
      type="password"
      class="w-full bg-surface border border-border rounded-md px-3 py-2 text-[13px] text-text-primary outline-none focus:border-accent"
      bind:value={form.api_key}
      placeholder="Enter your API key"
    />
  </div>

  <!-- Base URL -->
  <div class="space-y-1.5">
    <label class="text-[12px] text-text-secondary font-medium" for="base_url">Base URL</label>
    <input
      id="base_url"
      type="text"
      class="w-full bg-surface border border-border rounded-md px-3 py-2 text-[13px] text-text-primary outline-none focus:border-accent disabled:opacity-50 disabled:cursor-not-allowed"
      bind:value={form.base_url}
      disabled={!isCustomProvider}
      placeholder="https://api.example.com/v1"
    />
  </div>

  <!-- Model -->
  <div class="space-y-1.5">
    <label class="text-[12px] text-text-secondary font-medium" for="model">Model</label>
    {#if !isCustomProvider && currentPreset && currentPreset.models.length > 0}
      <select
        id="model"
        class="w-full bg-surface border border-border rounded-md px-3 py-2 text-[13px] text-text-primary outline-none focus:border-accent"
        bind:value={form.model}
      >
        {#each currentPreset.models as model}
          <option value={model}>{model}</option>
        {/each}
      </select>
    {:else}
      <input
        id="model"
        type="text"
        class="w-full bg-surface border border-border rounded-md px-3 py-2 text-[13px] text-text-primary outline-none focus:border-accent"
        bind:value={form.model}
        placeholder="Model name"
      />
    {/if}
  </div>

  <!-- Hotkey -->
  <div class="space-y-1.5">
    <label class="text-[12px] text-text-secondary font-medium" for="hotkey">Hotkey</label>
    {#if capturingHotkey}
      <div
        bind:this={hotkeyEl}
        class="w-full bg-accent/10 border border-accent rounded-md px-3 py-2 text-[13px] text-accent cursor-pointer text-center animate-pulse"
        role="button"
        tabindex="0"
        onkeydown={handleHotkeyKeydown}
        onblur={() => capturingHotkey = false}
      >
        Press a key combo...
      </div>
    {:else}
      <button
        id="hotkey"
        class="w-full bg-surface border border-border rounded-md px-3 py-2 text-[13px] text-text-primary outline-none cursor-pointer text-left hover:border-accent transition-colors"
        onclick={() => capturingHotkey = true}
      >
        {form.hotkey}
      </button>
    {/if}
  </div>

  <!-- Activation Mode -->
  <div class="space-y-1.5">
    <label class="text-[12px] text-text-secondary font-medium" for="activation_mode">Activation Mode</label>
    <select
      id="activation_mode"
      class="w-full bg-surface border border-border rounded-md px-3 py-2 text-[13px] text-text-primary outline-none focus:border-accent"
      bind:value={form.activation_mode}
    >
      <option value="toggle">Toggle</option>
      <option value="push_to_talk">Push to Talk</option>
    </select>
  </div>

  <!-- Output Mode -->
  <div class="space-y-1.5">
    <label class="text-[12px] text-text-secondary font-medium" for="output_mode">Output Mode</label>
    <select
      id="output_mode"
      class="w-full bg-surface border border-border rounded-md px-3 py-2 text-[13px] text-text-primary outline-none focus:border-accent"
      bind:value={form.output_mode}
    >
      <option value="auto_paste">Auto-paste</option>
      <option value="clipboard">Clipboard only</option>
    </select>
  </div>

  <!-- Language -->
  <div class="space-y-1.5">
    <label class="text-[12px] text-text-secondary font-medium" for="language">Language</label>
    <select
      id="language"
      class="w-full bg-surface border border-border rounded-md px-3 py-2 text-[13px] text-text-primary outline-none focus:border-accent"
      bind:value={form.language}
    >
      {#each languages as lang}
        <option value={lang.value}>{lang.label}</option>
      {/each}
    </select>
  </div>

  <!-- Launch at Startup -->
  <div class="flex items-center gap-2">
    <input
      id="launch_at_startup"
      type="checkbox"
      class="w-4 h-4 rounded border-border bg-surface accent-accent"
      bind:checked={form.launch_at_startup}
    />
    <label class="text-[13px] text-text-primary cursor-pointer" for="launch_at_startup">Launch at startup</label>
  </div>

  <!-- Save Button -->
  <button
    class="w-full bg-accent hover:bg-accent/80 text-white font-medium py-2 px-4 rounded-md text-[13px] transition-colors disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
    onclick={handleSave}
    disabled={saving}
  >
    {saving ? "Saving..." : "Save Settings"}
  </button>
</div>
