<script lang="ts">
  /**
   * VesselCard.svelte — Dashboard card representing a single container ("vessel").
   *
   * Displays:
   *  - Name, image tag, status badge
   *  - Real-time CPU/RAM meters (animated SVG arcs)
   *  - Action buttons: Start/Stop, Terminal, Delete
   *
   * Props are typed to match the future Rust `VesselInfo` struct output.
   */

  interface VesselInfo {
    id: string;
    name: string;
    image: string;
    status: "running" | "stopped" | "error";
    cpu_percent: number;   // 0–100
    mem_used_mb: number;
    mem_limit_mb: number;
  }

  interface Props {
    vessel: VesselInfo;
    onStart?: (id: string) => void;
    onStop?: (id: string) => void;
    onDelete?: (id: string) => void;
    onTerminal?: (id: string) => void;
  }

  import { invoke } from "@tauri-apps/api/core";
  import { toast } from "$lib/stores/toast.svelte";
  let { vessel, onStart, onStop, onDelete, onTerminal }: Props = $props();

  async function addToDesktop() {
    try {
      await invoke("generate_desktop_entry", {
        payload: {
          vessel_name: vessel.name,
          icon_path: null
        }
      });
      toast.success(`Created desktop shortcut for ${vessel.name}`);
    } catch (e: any) {
      toast.error(`Failed to create shortcut: ${e.toString()}`);
    }
  }

  // ---------------------------------------------------------------------------
  // SVG arc meter helpers
  // ---------------------------------------------------------------------------
  const R = 20; // circle radius (viewBox units)
  const C = 2 * Math.PI * R; // circumference

  function arcStyle(percent: number, color: string) {
    const clamped = Math.max(0, Math.min(100, percent));
    const dash = (clamped / 100) * C;
    return `stroke-dasharray: ${dash} ${C}; stroke: ${color};`;
  }

  const cpuStyle = $derived(arcStyle(vessel.cpu_percent, "var(--color-kap-accent)"));
  const memPercent = $derived(vessel.mem_limit_mb > 0
    ? (vessel.mem_used_mb / vessel.mem_limit_mb) * 100
    : 0);
  const memStyle = $derived(arcStyle(memPercent, "var(--color-kap-success)"));

  const isRunning = $derived(vessel.status === "running");
</script>

<article
  id="vessel-{vessel.id}"
  class="kap-card p-4 flex flex-col gap-3 animate-fade-in hover:border-[var(--color-kap-accent)] transition-colors duration-200"
>
  <!-- Header row -->
  <div class="flex items-start justify-between">
    <div class="flex flex-col gap-0.5 min-w-0">
      <span class="text-sm font-semibold truncate">{vessel.name}</span>
      <span class="text-xs text-[var(--color-kap-muted)] truncate">{vessel.image}</span>
    </div>
    <!-- Status badge -->
    <span class="badge badge-{vessel.status} shrink-0">
      <span class="w-1.5 h-1.5 rounded-full bg-current {isRunning ? 'animate-pulse-dot' : ''}"></span>
      {vessel.status}
    </span>
  </div>

  <!-- Resource meters -->
  {#if isRunning}
    <div class="flex items-center gap-4">
      <!-- CPU arc -->
      <div class="flex flex-col items-center gap-1">
        <svg width="52" height="52" viewBox="0 0 48 48" fill="none">
          <!-- Track -->
          <circle cx="24" cy="24" r={R} stroke="var(--color-kap-border)" stroke-width="4"
                  fill="none" transform="rotate(-90 24 24)"
                  stroke-dasharray="{C} {C}" stroke-dashoffset="0"/>
          <!-- Arc -->
          <circle cx="24" cy="24" r={R} stroke-width="4" fill="none"
                  transform="rotate(-90 24 24)"
                  style="{cpuStyle}"
                  stroke-linecap="round"
                  class="transition-all duration-700"/>
          <text x="24" y="28" text-anchor="middle" font-size="9" fill="var(--color-kap-text)"
                font-family="Inter, sans-serif" font-weight="600">
            {Math.round(vessel.cpu_percent)}%
          </text>
        </svg>
        <span class="text-[10px] text-[var(--color-kap-muted)]">CPU</span>
      </div>

      <!-- RAM arc -->
      <div class="flex flex-col items-center gap-1">
        <svg width="52" height="52" viewBox="0 0 48 48" fill="none">
          <circle cx="24" cy="24" r={R} stroke="var(--color-kap-border)" stroke-width="4"
                  fill="none" transform="rotate(-90 24 24)"
                  stroke-dasharray="{C} {C}" stroke-dashoffset="0"/>
          <circle cx="24" cy="24" r={R} stroke-width="4" fill="none"
                  transform="rotate(-90 24 24)"
                  style="{memStyle}"
                  stroke-linecap="round"
                  class="transition-all duration-700"/>
          <text x="24" y="28" text-anchor="middle" font-size="9" fill="var(--color-kap-text)"
                font-family="Inter, sans-serif" font-weight="600">
            {Math.round(memPercent)}%
          </text>
        </svg>
        <span class="text-[10px] text-[var(--color-kap-muted)]">RAM</span>
      </div>

      <!-- RAM text -->
      <div class="flex flex-col gap-0.5 text-xs text-[var(--color-kap-muted)]">
        <span>{vessel.mem_used_mb} MB</span>
        <span>/ {vessel.mem_limit_mb} MB</span>
      </div>
    </div>
  {:else}
    <div class="h-14 flex items-center justify-center text-xs text-[var(--color-kap-muted)]">
      Container is stopped
    </div>
  {/if}

  <!-- Action buttons -->
  <div class="flex items-center gap-2 pt-1 border-t border-[var(--color-kap-border)]">
    {#if isRunning}
      <button
        id="btn-stop-{vessel.id}"
        class="btn btn-ghost text-xs py-1 px-2.5 flex-1"
        onclick={() => onStop?.(vessel.id)}
      >
        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="1"/></svg>
        Stop
      </button>
      <button
        id="btn-terminal-{vessel.id}"
        class="btn btn-ghost text-xs py-1 px-2.5 flex-1"
        onclick={() => onTerminal?.(vessel.id)}
      >
        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>
        </svg>
        Shell
      </button>
    {:else}
      <button
        id="btn-start-{vessel.id}"
        class="btn btn-primary text-xs py-1 px-2.5 flex-1"
        onclick={() => onStart?.(vessel.id)}
      >
        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>
        Start
      </button>
    {/if}
    <button
      id="btn-delete-{vessel.id}"
      class="btn btn-ghost text-xs py-1 px-2 text-[var(--color-kap-destruct)] hover:bg-[color-mix(in_srgb,var(--color-kap-destruct)_15%,transparent)]"
      onclick={() => onDelete?.(vessel.id)}
      title="Delete vessel"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/>
        <path d="M9 6V4h6v2"/>
      </svg>
    </button>
    <button
      id="btn-desktop-{vessel.id}"
      class="btn btn-ghost text-xs py-1 px-2 hover:text-[var(--color-kap-accent)]"
      onclick={addToDesktop}
      title="Add to Desktop"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
    </button>
  </div>
</article>
