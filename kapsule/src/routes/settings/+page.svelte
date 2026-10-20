<script lang="ts">
  /**
   * settings/+page.svelte — Application settings (placeholder)
   */
  import { engineStatus, activeEngine, engineLoading, detectEngines } from "$lib/stores/engine";
  import { getVersion } from "@tauri-apps/api/app";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let appVersion = $state("Loading...");
  let vesselsRoot = $state<string | null>(null);

  onMount(async () => {
    invoke<string | null>("get_vessels_root")
      .then((root) => (vesselsRoot = root))
      .catch((e) => console.error("Failed to get vessels root:", e));
    try {
      appVersion = await getVersion();
    } catch (e) {
      appVersion = "Unknown";
    }
  });
</script>

<svelte:head>
  <title>Settings — Kapsule</title>
</svelte:head>

<div class="flex flex-col gap-6 animate-fade-in max-w-xl">
  <h1 class="text-lg font-semibold">Settings</h1>

  <!-- Engine info card -->
  <section class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <h2 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-kap-muted)]">Container Engine</h2>
      <button class="btn btn-ghost text-xs py-1 px-2.5" onclick={detectEngines} disabled={$engineLoading}
              title="Probe the Podman and Docker sockets again">
        {$engineLoading ? "Detecting…" : "Re-detect engines"}
      </button>
    </div>
    <div class="kap-card p-4 flex flex-col gap-3">
      <div class="flex items-center justify-between text-sm">
        <span class="text-[var(--color-kap-text-dim)]">Active engine</span>
        <span class="font-medium capitalize {$activeEngine ? 'text-[var(--color-kap-accent)]' : 'text-[var(--color-kap-muted)]'}">
          {$activeEngine ?? "None detected"}
        </span>
      </div>
      <p class="text-xs text-[var(--color-kap-muted)] -mt-1">
        Switch engines with the toggle in the top-left; your choice is remembered between launches.
      </p>
      <div class="flex items-center justify-between text-sm">
        <span class="text-[var(--color-kap-text-dim)]">Podman socket</span>
        <span class="font-mono text-xs text-[var(--color-kap-muted)] truncate max-w-48">
          {$engineStatus?.podman_socket ?? "—"}
        </span>
      </div>
      <div class="flex items-center justify-between text-sm">
        <span class="text-[var(--color-kap-text-dim)]">Docker socket</span>
        <span class="font-mono text-xs text-[var(--color-kap-muted)] truncate max-w-48">
          {$engineStatus?.docker_socket ?? "—"}
        </span>
      </div>
    </div>
  </section>

  <!-- Vessels data directory -->
  <section class="flex flex-col gap-2">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-kap-muted)]">Storage</h2>
    <div class="kap-card p-4 flex items-center justify-between text-sm">
      <span class="text-[var(--color-kap-text-dim)]">Vessel home root</span>
      <span class="font-mono text-xs text-[var(--color-kap-muted)] truncate max-w-72" title={vesselsRoot ?? ""}>
        {vesselsRoot ?? "—"}
      </span>
    </div>
  </section>

  <!-- About -->
  <section class="flex flex-col gap-2">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-kap-muted)]">About</h2>
    <div class="kap-card p-4 flex items-center justify-between text-sm">
      <span class="text-[var(--color-kap-text-dim)]">Version</span>
      <span class="font-mono text-xs text-[var(--color-kap-muted)]">{appVersion}</span>
    </div>
  </section>
</div>
