<script lang="ts">
  /**
   * +layout.svelte — Root application layout.
   *
   * Responsibilities:
   *  1. Import global CSS (app.css)
   *  2. Run engine auto-detection on mount
   *  3. Render HeaderBar + Sidebar + main content area
   */
  // Bundled rather than fetched from Google Fonts so the app works offline
  // and the CSP doesn't need to allow third-party origins
  import "@fontsource/inter/400.css";
  import "@fontsource/inter/500.css";
  import "@fontsource/inter/600.css";
  import "@fontsource/inter/700.css";
  import "../app.css";
  import { onMount } from "svelte";
  import HeaderBar from "$lib/components/HeaderBar.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import CreateVesselWizard from "$lib/components/CreateVesselWizard.svelte";
  import Toaster from "$lib/components/Toaster.svelte";
  import { detectEngines } from "$lib/stores/engine";
  import { uiState } from "$lib/stores/ui.svelte";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";

  interface Props {
    children: import("svelte").Snippet;
  }
  let { children }: Props = $props();

  onMount(() => {
    detectEngines();
  });
</script>

<div class="flex flex-col h-screen overflow-hidden" style="background: var(--color-kap-window);">
  <!-- HeaderBar (drag region) -->
  <HeaderBar />

  <!-- Body: Sidebar + main content -->
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <main class="flex-1 overflow-y-auto p-6">
      {@render children()}
    </main>
  </div>
</div>
<Toaster />

<CreateVesselWizard 
  isOpen={uiState.isWizardOpen} 
  onClose={() => uiState.isWizardOpen = false} 
  onSuccess={() => {
    // The wizard can be opened from the Library; show the new vessel on the dashboard
    if (page.url.pathname !== "/") goto("/");
    else document.dispatchEvent(new CustomEvent('refresh-vessels'));
  }} 
/>
