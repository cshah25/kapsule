<script lang="ts">
  /**
   * VesselCard.svelte — Dashboard card representing a single container ("vessel").
   *
   * Displays:
   *  - Name, image tag, status badge
   *  - Real-time CPU/RAM meters (animated SVG arcs)
   *  - Action buttons: Start/Stop/Resume, Logs, Delete
   */
  import type { VesselInfo } from "$lib/types";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { toast } from "$lib/stores/toast.svelte";

  interface Props {
    vessel: VesselInfo;
    onStart?: (id: string) => void;
    onResume?: (id: string) => void;
    onStop?: (id: string) => void;
    onDelete?: (id: string) => void;
    onLogs?: (id: string) => void;
    /** Set while an action is in flight (e.g. "Stopping…"); disables the actions. */
    busyLabel?: string;
  }

  let { vessel, onStart, onResume, onStop, onDelete, onLogs, busyLabel }: Props = $props();

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

  /** Format memory values for display */
  function formatMem(mb: number): string {
    if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
    return `${mb.toFixed(1)} MB`;
  }

  const cpuStyle = $derived(arcStyle(vessel.cpu_percent, "var(--color-kap-accent)"));
  const memPercent = $derived(vessel.mem_limit_mb > 0
    ? (vessel.mem_used_mb / vessel.mem_limit_mb) * 100
    : 0);
  const memStyle = $derived(arcStyle(memPercent, "var(--color-kap-success)"));

  const isRunning = $derived(vessel.status === "running");

  async function openPort(hostPort: number) {
    try {
      await openUrl(`http://localhost:${hostPort}`);
    } catch (err) {
      toast.error(`Couldn't open localhost:${hostPort}: ${err}`);
    }
  }

  // One main action per state; restarting containers are usually crash-looping,
  // so stopping them is the useful action
  const primary = $derived(
    vessel.status === "running" || vessel.status === "restarting"
      ? { label: "Stop", run: onStop, style: "btn-ghost", icon: "stop" }
      : vessel.status === "paused"
        ? { label: "Resume", run: onResume, style: "btn-primary", icon: "play" }
        : { label: "Start", run: onStart, style: "btn-primary", icon: "play" }
  );

  const idleMessage: Record<string, string> = {
    paused: "Container is paused",
    restarting: "Container is restarting…",
    error: "Container is dead or being removed",
  };
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
      <div class="flex flex-col items-center gap-1" title="Share of total host CPU">
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
        <span>{formatMem(vessel.mem_used_mb)}</span>
        <span>/ {formatMem(vessel.mem_limit_mb)}</span>
      </div>
    </div>
  {:else}
    <div class="h-14 flex items-center justify-center text-xs text-[var(--color-kap-muted)]">
      {idleMessage[vessel.status] ?? "Container is stopped"}
    </div>
  {/if}

  <!-- Published ports (engines only report these while the container runs) -->
  {#if vessel.ports.length > 0}
    <div class="flex flex-wrap gap-1.5">
      {#each vessel.ports as port (`${port.host_port}-${port.container_port}`)}
        <button
          class="inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] font-mono bg-[var(--color-kap-surface2)] text-[var(--color-kap-text-dim)] hover:text-[var(--color-kap-accent)] hover:bg-[color-mix(in_srgb,var(--color-kap-accent)_12%,transparent)] transition-colors"
          onclick={() => openPort(port.host_port)}
          title="Open http://localhost:{port.host_port} in your browser{port.exposed_lan ? ' — also reachable from other devices on your network' : ''}"
        >
          :{port.host_port} → {port.container_port}
          {#if port.exposed_lan}
            <span class="font-sans font-semibold text-[9px] uppercase text-[var(--color-kap-warning)]">LAN</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}

  <!-- Action buttons -->
  <div class="flex items-center flex-wrap gap-2 pt-1 mt-auto border-t border-[var(--color-kap-border)]">
    <button
      id="btn-{primary.label.toLowerCase()}-{vessel.id}"
      class="btn {primary.style} text-xs py-1 px-2.5 flex-1 justify-center min-w-[70px] disabled:opacity-60 disabled:cursor-wait"
      onclick={() => primary.run?.(vessel.id)}
      disabled={!!busyLabel}
    >
      {#if busyLabel}
        <span class="w-3 h-3 rounded-full border-2 border-current/30 border-t-current animate-spin"></span>
        {busyLabel}
      {:else}
        {#if primary.icon === "stop"}
          <svg class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="1"/></svg>
        {:else}
          <svg class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>
        {/if}
        {primary.label}
      {/if}
    </button>
    <!-- Available in every state: logs matter most after a crash -->
    <button
      id="btn-logs-{vessel.id}"
      class="btn btn-ghost text-xs py-1 px-2.5 flex-1 justify-center min-w-[70px]"
      onclick={() => onLogs?.(vessel.id)}
      title="View container output"
    >
      <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/>
        <line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/>
      </svg>
      Logs
    </button>
    <button
      id="btn-delete-{vessel.id}"
      class="btn btn-ghost text-xs py-1 px-2 shrink-0 text-[var(--color-kap-destruct)] hover:bg-[color-mix(in_srgb,var(--color-kap-destruct)_15%,transparent)] disabled:opacity-40"
      onclick={() => onDelete?.(vessel.id)}
      disabled={!!busyLabel}
      title="Delete vessel"
    >
      <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/>
        <path d="M9 6V4h6v2"/>
      </svg>
    </button>
  </div>
</article>
