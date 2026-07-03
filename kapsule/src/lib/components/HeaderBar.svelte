<script lang="ts">
  // HeaderBar.svelte — The 48px application header.
  //
  // Contains:
  //  - App title (centered)
  //  - Engine toggle (Podman / Docker) on the left
  //  - Global action buttons on the right (Add Vessel, Search)
  //  - Window controls (minimize, maximize, close)
  import { engineStatus, activeEngine, switchEngine, engineLoading } from "$lib/stores/engine";
  import type { Engine } from "$lib/stores/engine";
  import { uiState } from "$lib/stores/ui.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const appWindow = getCurrentWindow();

  let isMaximized = $state(false);

  // Check initial maximized state and listen for changes
  import { onMount } from "svelte";
  onMount(() => {
    appWindow.isMaximized().then(m => isMaximized = m);
    let unlisten: () => void;
    appWindow.onResized(async () => {
      isMaximized = await appWindow.isMaximized();
    }).then(u => unlisten = u);
    return () => {
      if (unlisten) unlisten();
    };
  });

  interface Props {
    onAddVessel?: () => void;
  }

  let { onAddVessel }: Props = $props();

  async function toggle(engine: Engine) {
    const err = await switchEngine(engine);
    if (err) {
      toast.error(`Failed to switch to ${engine}: ${err}`);
    }
  }

  function handlePointerDown(e: PointerEvent) {
    // Only start dragging if the target isn't an interactive element
    const target = e.target as HTMLElement;
    if (!target.closest('button, input, select, a, .no-drag')) {
      appWindow.startDragging();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<header class="kap-headerbar select-none" data-tauri-drag-region onpointerdown={handlePointerDown}>
  <!-- Engine toggle segmented control -->
  <div class="relative flex items-center bg-[var(--color-kap-window)] rounded-lg p-1 w-[200px] h-[34px]">
    {#if $engineLoading}
      <span class="w-full text-center text-xs text-[var(--color-kap-muted)] animate-pulse-dot">Detecting…</span>
    {:else}
      <!-- Sliding Background -->
      <div 
        class="absolute top-1 bottom-1 w-[calc(50%-4px)] bg-[var(--color-kap-accent)] rounded-md transition-all duration-300 ease-out z-0"
        style="left: {$activeEngine === 'docker' ? 'calc(50% + 2px)' : '4px'};"
      ></div>

      <!-- Podman button -->
      <button
        id="engine-toggle-podman"
        class="relative flex-1 z-10 flex items-center justify-center gap-1.5 text-[11px] font-medium rounded-md h-full transition-colors duration-200
               {$activeEngine === 'podman' ? 'text-white' : 'text-[var(--color-kap-muted)] hover:text-white'}"
        disabled={!$engineStatus?.podman_available}
        onclick={() => toggle("podman")}
        title={$engineStatus?.podman_available ? "Use Podman (rootless)" : "Podman not detected"}
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="7" width="20" height="14" rx="2"/><path d="M16 7V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2"/><line x1="12" y1="12" x2="12" y2="16"/><line x1="10" y1="14" x2="14" y2="14"/></svg>
        Podman
      </button>

      <!-- Docker button -->
      <button
        id="engine-toggle-docker"
        class="relative flex-1 z-10 flex items-center justify-center gap-1.5 text-[11px] font-medium rounded-md h-full transition-colors duration-200
               {$activeEngine === 'docker' ? 'text-white' : 'text-[var(--color-kap-muted)] hover:text-white'}"
        disabled={!$engineStatus?.docker_available}
        onclick={() => toggle("docker")}
        title={$engineStatus?.docker_available ? "Use Docker" : "Docker not detected"}
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 12.5c0 .5-.1 1-.3 1.5H14v-3h4v-2h-4V7h-2v2H8V7H6v2H2.3C2.1 8.5 2 8 2 7.5 2 4.4 4.4 2 7.5 2h9C19.6 2 22 4.4 22 7.5v5z"/><path d="M2.3 10C1 10.5 0 11.9 0 13.5 0 15.4 1.6 17 3.5 17h17c1.9 0 3.5-1.6 3.5-3.5 0-1.6-1.1-3-2.7-3.4"/></svg>
        Docker
      </button>
    {/if}
  </div>

  <!-- Centered title -->
  <div class="flex-1 flex justify-center">
    <span class="text-sm font-semibold tracking-wide text-[var(--color-kap-text)]">Kapsule</span>
  </div>

  <!-- Right-side actions -->
  <div class="flex items-center gap-2">
    <!-- Add Vessel -->
    <button
      id="btn-add-vessel"
      class="btn btn-primary text-xs py-1.5 px-3"
      onclick={() => {
        if (onAddVessel) onAddVessel();
        uiState.isWizardOpen = true;
      }}
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <line x1="12" y1="5" x2="12" y2="19"/>
        <line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      New Vessel
    </button>

    <!-- Window controls separator -->
    <div class="w-px h-5 bg-[var(--color-kap-border)] ml-2"></div>

    <!-- Minimize -->
    <button
      id="btn-window-minimize"
      class="window-control"
      onclick={() => appWindow.minimize()}
      title="Minimize"
    >
      <svg width="12" height="12" viewBox="0 0 12 12">
        <rect x="2" y="5.5" width="8" height="1" fill="currentColor" rx="0.5" />
      </svg>
    </button>

    <!-- Maximize / Restore -->
    <button
      id="btn-window-maximize"
      class="window-control"
      onclick={() => appWindow.toggleMaximize()}
      title={isMaximized ? "Restore" : "Maximize"}
    >
      {#if isMaximized}
        <!-- Restore icon (two overlapping rectangles) -->
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect x="3" y="3" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1" rx="0.5" />
          <path d="M4.5 3V2h6v6h-1" fill="none" stroke="currentColor" stroke-width="1" />
        </svg>
      {:else}
        <!-- Maximize icon -->
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect x="2" y="2" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1.2" rx="0.5" />
        </svg>
      {/if}
    </button>

    <!-- Close -->
    <button
      id="btn-window-close"
      class="window-control window-control-close"
      onclick={() => appWindow.close()}
      title="Close"
    >
      <svg width="12" height="12" viewBox="0 0 12 12">
        <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      </svg>
    </button>
  </div>
</header>

<style>
  .window-control {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 28px;
    border: none;
    background: transparent;
    color: var(--color-kap-text-dim);
    border-radius: 6px;
    cursor: pointer;
    transition: background-color 0.15s ease, color 0.15s ease;
  }
  .window-control:hover {
    background-color: var(--color-kap-surface2);
    color: var(--color-kap-text);
  }
  .window-control-close:hover {
    background-color: var(--color-kap-destruct);
    color: #fff;
  }
</style>
