<script lang="ts">
  /**
   * settings/+page.svelte — Application settings (placeholder)
   */
  import { engineStatus, activeEngine } from "$lib/stores/engine";
</script>

<svelte:head>
  <title>Settings — Kapsule</title>
</svelte:head>

<div class="flex flex-col gap-6 animate-fade-in max-w-xl">
  <h1 class="text-lg font-semibold">Settings</h1>

  <!-- Engine info card -->
  <section class="flex flex-col gap-2">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-kap-muted)]">Container Engine</h2>
    <div class="kap-card p-4 flex flex-col gap-3">
      <div class="flex items-center justify-between text-sm">
        <span class="text-[var(--color-kap-text-dim)]">Active engine</span>
        <span class="font-medium capitalize {$activeEngine ? 'text-[var(--color-kap-accent)]' : 'text-[var(--color-kap-muted)]'}">
          {$activeEngine ?? "None detected"}
        </span>
      </div>
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
      <span class="font-mono text-xs text-[var(--color-kap-muted)]">~/.local/share/kapsule/vessels</span>
    </div>
  </section>
</div>
