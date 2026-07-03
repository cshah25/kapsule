/**
 * types.ts — Shared TypeScript types for Kapsule.
 *
 * Single source of truth for data structures passed between
 * the Tauri backend and the Svelte frontend.
 */

export type VesselStatus = "running" | "stopped" | "error";

export interface VesselInfo {
  id: string;
  name: string;
  image: string;
  status: VesselStatus;
  cpu_percent: number;
  mem_used_mb: number;
  mem_limit_mb: number;
}
