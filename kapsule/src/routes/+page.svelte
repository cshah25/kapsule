<script lang="ts">
  /**
   * +page.svelte — Dashboard (Vessels overview)
   *
   * Displays a responsive grid of VesselCards.
   * Uses stub data for Week 1; real container list comes in Week 3.
   */
  import VesselCard from "$lib/components/VesselCard.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import TerminalOverlay from "$lib/components/TerminalOverlay.svelte";
  import { engineStatus, activeEngine } from "$lib/stores/engine";
  import { uiState } from "$lib/stores/ui.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  // ---------------------------------------------------------------------------
  // Stub vessels for UI development (will be replaced by Tauri invoke in Week 3)
  // ---------------------------------------------------------------------------
  type VesselStatus = "running" | "stopped" | "error";

  interface VesselInfo {
    id: string;
    name: string;
    image: string;
    status: VesselStatus;
    cpu_percent: number;
    mem_used_mb: number;
    mem_limit_mb: number;
  }

  let vessels = $state<VesselInfo[]>([]);
  let isLoading = $state(true);
  let pollInterval: ReturnType<typeof setInterval>;
  let activeTerminalId = $state<string | null>(null);
  let activeTerminalName = $state<string | null>(null);

  async function fetchVessels() {
    if ($activeEngine) {
      try {
        vessels = await invoke<VesselInfo[]>("list_vessels");
      } catch (err) {
        console.error("Failed to fetch vessels:", err);
      } finally {
        isLoading = false;
      }
    } else if ($engineStatus !== null) {
      // We know engine detection finished but none is selected
      isLoading = false;
    }
  }

  onMount(() => {
    fetchVessels();
    pollInterval = setInterval(fetchVessels, 2000);
    const handler = () => fetchVessels();
    document.addEventListener('refresh-vessels', handler);
    return () => {
      document.removeEventListener('refresh-vessels', handler);
      if (pollInterval) clearInterval(pollInterval);
    };
  });

  $effect(() => {
    if ($activeEngine) {
      fetchVessels();
    }
  });

  // ---------------------------------------------------------------------------
  // Handlers
  // ---------------------------------------------------------------------------
  async function handleStart(id: string) {
    try {
      await invoke("start_vessel", { id });
      await fetchVessels();
    } catch (err) {
      console.error(err);
    }
  }

  async function handleStop(id: string) {
    try {
      await invoke("stop_vessel", { id });
      await fetchVessels();
    } catch (err) {
      console.error(err);
    }
  }

  async function handleDelete(id: string) {
    try {
      await invoke("delete_vessel", { id });
      await fetchVessels();
    } catch (err) {
      console.error(err);
    }
  }

  function handleTerminal(id: string) {
    activeTerminalId = id;
    activeTerminalName = vessels.find((v) => v.id === id)?.name || id;
  }

  function handleAddVessel() {
    uiState.isWizardOpen = true;
  }

  // ---------------------------------------------------------------------------
  // Derived counts for the stats bar
  // ---------------------------------------------------------------------------
  const runningCount = $derived(vessels.filter((v) => v.status === "running").length);
  const stoppedCount = $derived(vessels.filter((v) => v.status === "stopped").length);
</script>

<svelte:head>
  <title>Vessels — Kapsule</title>
</svelte:head>

<div class="flex flex-col gap-6 animate-fade-in">
  <!-- Page header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-lg font-semibold">Vessels</h1>
      <p class="text-xs text-[var(--color-kap-muted)] mt-0.5">
        {#if $activeEngine}
          Engine: <span class="capitalize font-medium text-[var(--color-kap-accent)]">{$activeEngine}</span>
          {#if $activeEngine === "podman"}
            · rootless
          {/if}
        {:else}
          No engine detected — is Podman or Docker running?
        {/if}
      </p>
    </div>

    <!-- Stats pills -->
    <div class="flex items-center gap-2">
      <span class="badge badge-running">{runningCount} running</span>
      <span class="badge badge-stopped">{stoppedCount} stopped</span>
    </div>
  </div>

  <!-- Engine warning banner -->
  {#if !$activeEngine && $engineStatus !== null}
    <div class="kap-card p-4 flex flex-col gap-3"
         style="border-color: var(--color-kap-warning); background: color-mix(in srgb, var(--color-kap-warning) 8%, var(--color-kap-surface));">
      <div class="flex items-center gap-3">
        <svg class="w-4 h-4 shrink-0 text-[var(--color-kap-warning)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
          <line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
        </svg>
        <p class="text-xs font-medium text-[var(--color-kap-warning)]">
          No container engine detected
        </p>
      </div>
      {#if $engineStatus.diagnostics && $engineStatus.diagnostics.length > 0}
        <div class="flex flex-col gap-2 pl-7">
          {#each $engineStatus.diagnostics as hint}
            <div class="text-xs text-[var(--color-kap-text-dim)] bg-[var(--color-kap-surface2)] rounded-lg px-3 py-2 font-mono leading-relaxed"
                 style="word-break: break-word;">
              {hint}
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-xs text-[var(--color-kap-text-dim)] pl-7">
          Start a container engine to manage vessels.
        </p>
      {/if}
    </div>
  {/if}

  <!-- Vessel grid or empty state -->
  {#if isLoading}
    <div class="flex flex-col items-center justify-center h-64 gap-4 animate-fade-in">
      <div class="w-8 h-8 rounded-full border-2 border-white/10 border-t-[var(--color-kap-accent)] animate-spin"></div>
      <span class="text-sm text-[var(--color-kap-muted)]">Loading vessels...</span>
    </div>
  {:else if vessels.length === 0}
    <EmptyState onAddVessel={handleAddVessel} />
  {:else}
    <div class="grid gap-4"
         style="grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));">
      {#each vessels as vessel (vessel.id)}
        <VesselCard
          {vessel}
          onStart={handleStart}
          onStop={handleStop}
          onDelete={handleDelete}
          onTerminal={handleTerminal}
        />
      {/each}
    </div>
  {/if}
</div>

<TerminalOverlay 
  vesselId={activeTerminalId} 
  vesselName={activeTerminalName}
  onClose={() => activeTerminalId = null}
/>
