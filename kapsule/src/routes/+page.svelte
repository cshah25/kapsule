<script lang="ts">
  /**
   * +page.svelte — Dashboard (Vessels overview)
   *
   * Displays a responsive grid of VesselCards.
   * Uses stub data for Week 1; real container list comes in Week 3.
   */
  import VesselCard from "$lib/components/VesselCard.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
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
  let pollInterval: ReturnType<typeof setInterval>;

  async function fetchVessels() {
    if ($activeEngine) {
      try {
        vessels = await invoke<VesselInfo[]>("list_vessels");
      } catch (err) {
        console.error("Failed to fetch vessels:", err);
      }
    }
  }

  onMount(() => {
    fetchVessels();
    pollInterval = setInterval(fetchVessels, 2000);
  });

  $effect(() => {
    if ($activeEngine) {
      fetchVessels();
    }
  });

  onDestroy(() => {
    if (pollInterval) clearInterval(pollInterval);
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
    console.log("Open terminal for", id); // TODO: launch xterm.js overlay (Week 3)
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
    <div class="kap-card p-3 flex items-center gap-3 border-[var(--color-kap-warning)]"
         style="border-color: var(--color-kap-warning); background: color-mix(in srgb, var(--color-kap-warning) 8%, var(--color-kap-surface));">
      <svg class="w-4 h-4 shrink-0 text-[var(--color-kap-warning)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
        <line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
      </svg>
      <p class="text-xs text-[var(--color-kap-warning)]">
        Neither Podman nor Docker was detected. Start a container engine to manage vessels.
      </p>
    </div>
  {/if}

  <!-- Vessel grid or empty state -->
  {#if vessels.length === 0}
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
