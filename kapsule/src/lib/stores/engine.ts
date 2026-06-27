/**
 * stores/engine.ts — Svelte store for container engine state.
 *
 * Wraps the `get_engine_status` and `set_engine` Tauri commands and exposes
 * reactive stores that the rest of the UI can subscribe to.
 */

import { writable, derived, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Types (mirror Rust structs)
// ---------------------------------------------------------------------------

export type Engine = "podman" | "docker";

export interface EngineStatus {
  podman_available: boolean;
  docker_available: boolean;
  podman_socket: string | null;
  docker_socket: string | null;
  active_engine: Engine | null;
  diagnostics: string[];
}

// ---------------------------------------------------------------------------
// Stores
// ---------------------------------------------------------------------------

/** Full status object returned from Rust on startup. */
export const engineStatus = writable<EngineStatus | null>(null);

/** Loading / error state for the engine probe. */
export const engineLoading = writable(true);
export const engineError   = writable<string | null>(null);

/** Convenient derived: which engine is currently active. */
export const activeEngine = derived(engineStatus, ($s) => $s?.active_engine ?? null);

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/** Run engine auto-detection. Call this once in the root layout's `onMount`. */
export async function detectEngines(): Promise<void> {
  engineLoading.set(true);
  engineError.set(null);
  try {
    const status = await invoke<EngineStatus>("get_engine_status");
    engineStatus.set(status);
  } catch (err) {
    engineError.set(String(err));
  } finally {
    engineLoading.set(false);
  }
}

/** Switch the active engine both in Rust state and in the Svelte store.
 *  Returns an error message string if the switch fails, or null on success. */
export async function switchEngine(engine: Engine): Promise<string | null> {
  try {
    await invoke("set_engine", { engine });
    engineStatus.update((s) =>
      s ? { ...s, active_engine: engine } : s
    );
    return null;
  } catch (err) {
    return String(err);
  }
}
