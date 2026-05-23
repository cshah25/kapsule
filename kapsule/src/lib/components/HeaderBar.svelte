<script lang="ts">
  /**
   * HeaderBar.svelte — The 48px application header.
   *
   * Contains:
   *  - App title (centered)
   *  - Engine toggle (Podman / Docker) on the left
   *  - Global action buttons on the right (Add Vessel, Search)
   */
  import { engineStatus, activeEngine, switchEngine, engineLoading } from "$lib/stores/engine";
  import type { Engine } from "$lib/stores/engine";
  import { uiState } from "$lib/stores/ui.svelte";
  import { toast } from "$lib/stores/toast.svelte";

  interface Props {
    onAddVessel?: () => void;
  }

  let { onAddVessel }: Props = $props();

  async function toggle(engine: Engine) {
    await switchEngine(engine);
  }
</script>

<header class="kap-headerbar select-none">
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

  </div>
</header>
