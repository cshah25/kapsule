<script lang="ts">
  /**
   * EmptyState.svelte — Shown on the dashboard when no vessels exist yet.
   */
  interface Props {
    onAddVessel?: () => void;
  }
  let { onAddVessel }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center gap-6 py-20 animate-fade-in">
  <!-- Icon -->
  <div class="relative">
    <div class="w-24 h-24 rounded-2xl flex items-center justify-center"
         style="background: linear-gradient(135deg, var(--color-kap-surface) 0%, var(--color-kap-surface2) 100%);
                border: 1px solid var(--color-kap-border);">
      <svg class="w-12 h-12 text-[var(--color-kap-accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="2" y="7" width="20" height="14" rx="3"/>
        <path d="M16 7V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2"/>
        <line x1="12" y1="11" x2="12" y2="17"/>
        <line x1="9" y1="14" x2="15" y2="14"/>
      </svg>
    </div>
    <!-- Decorative ring -->
    <div class="absolute inset-0 rounded-2xl"
         style="box-shadow: 0 0 0 8px color-mix(in srgb, var(--color-kap-accent) 10%, transparent);"></div>
  </div>

  <div class="text-center max-w-xs">
    <h2 class="text-base font-semibold mb-1">No Vessels Yet</h2>
    <p class="text-sm text-[var(--color-kap-muted)] leading-relaxed">
      A vessel is a secure, isolated container with its own home directory.
      Create your first one to get started.
    </p>
  </div>

  <button
    id="btn-create-first-vessel"
    class="btn btn-primary px-6 py-2.5 text-sm"
    onclick={onAddVessel}
  >
    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
      <line x1="12" y1="5" x2="12" y2="19"/>
      <line x1="5" y1="12" x2="19" y2="12"/>
    </svg>
    Create First Vessel
  </button>
</div>
