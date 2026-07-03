<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { Channel, invoke } from "@tauri-apps/api/core";
  import "@xterm/xterm/css/xterm.css";

  interface Props {
    vesselId: string | null;
    vesselName: string | null;
    onClose: () => void;
  }

  let { vesselId, vesselName, onClose }: Props = $props();

  let terminalContainer = $state<HTMLDivElement>();
  let term: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let resizeObserver: ResizeObserver | null = null;
  let isTerminalOpen = false;
  let activeStreamId: string | null = null;

  function initTerminal() {
    if (term) return; // Already initialised
    term = new Terminal({
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
      },
      fontFamily: 'monospace',
      fontSize: 13,
      convertEol: true,
    });
    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
  }

  function openTerminal() {
    if (!term || !terminalContainer || isTerminalOpen) return;
    term.open(terminalContainer);
    fitAddon?.fit();
    isTerminalOpen = true;

    // Observe container resizes
    resizeObserver = new ResizeObserver(() => {
      if (fitAddon && isTerminalOpen) fitAddon.fit();
    });
    resizeObserver.observe(terminalContainer);
  }

  async function startLogStream(id: string) {
    if (!term) return;
    // Mark which stream is active so stale callbacks are ignored
    activeStreamId = id;
    term.clear();

    const channel = new Channel<string>();
    const capturedId = id;
    channel.onmessage = (message) => {
      // Ignore messages from a stale stream
      if (activeStreamId !== capturedId) return;
      term?.write(message.replace(/\n/g, "\r\n"));
    };

    try {
      await invoke("stream_vessel_logs", { id, onMessage: channel });
    } catch (err) {
      if (activeStreamId === capturedId) {
        console.error("Log stream error:", err);
        term?.write(`\r\n\x1b[31mError streaming logs: ${err}\x1b[0m\r\n`);
      }
    }
  }

  // When vesselId changes, start a new log stream
  $effect(() => {
    if (vesselId) {
      initTerminal();
      // Need to defer opening until the DOM element is rendered
      requestAnimationFrame(() => {
        openTerminal();
        startLogStream(vesselId);
      });
    } else {
      // Vessel closed — stop stream
      activeStreamId = null;
    }
  });

  onDestroy(() => {
    activeStreamId = null;
    if (resizeObserver) resizeObserver.disconnect();
    if (term) {
      term.dispose();
      term = null;
      isTerminalOpen = false;
    }
  });
</script>

{#if vesselId}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-8 animate-fade-in">
    <div class="bg-[var(--color-kap-window)] rounded-xl w-full max-w-4xl h-full max-h-[70vh] shadow-2xl flex flex-col overflow-hidden border border-white/10">
      
      <div class="flex items-center justify-between px-4 py-2 border-b border-white/10 bg-[#1e1e1e]">
        <div class="flex items-center gap-2">
          <svg class="w-4 h-4 text-white/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>
          </svg>
          <span class="text-sm font-semibold tracking-wide text-white/80">{vesselName} logs</span>
        </div>
        <button aria-label="Close terminal" class="btn btn-ghost p-1 rounded-full text-white/50 hover:text-white" onclick={onClose}>
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
        </button>
      </div>
      
      <div class="flex-1 bg-[#1e1e1e] p-2 overflow-hidden relative">
        <div class="absolute inset-2" bind:this={terminalContainer}></div>
      </div>
    </div>
  </div>
{/if}
