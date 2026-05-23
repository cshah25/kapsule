<script lang="ts">
  /**
   * library/+page.svelte — Registry Explorer (placeholder for Week 4)
   */
</script>

<svelte:head>
  <title>Library — Kapsule</title>
</svelte:head>

<div class="flex flex-col gap-4 animate-fade-in">
  <h1 class="text-lg font-semibold">Library</h1>
  <p class="text-sm text-[var(--color-kap-muted)]">
    The Registry Explorer ships in Week 4. Browse and pull images from Docker Hub, Quay.io, and GHCR.
  </p>

  <!-- Stub search bar to show the design -->
  <div class="flex gap-2 max-w-lg">
    <input
      id="library-search"
      class="kap-input"
      placeholder="Search images… e.g. node:24-alpine"
      disabled
    />
    <button class="btn btn-primary px-4 opacity-50 cursor-not-allowed" disabled>
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
      </svg>
      Search
    </button>
  </div>

  <div class="kap-card p-8 flex flex-col items-center gap-3 text-center max-w-md">
    <svg class="w-10 h-10 text-[var(--color-kap-muted)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
    </svg>
    <span class="text-sm text-[var(--color-kap-muted)]">Coming in Week 4</span>
  </div>
</div>
