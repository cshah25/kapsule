<script lang="ts">
  import { toast } from "$lib/stores/toast.svelte";
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
  {#each toast.toasts as t (t.id)}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div 
      class="pointer-events-auto flex items-center gap-3 px-4 py-3 rounded-lg shadow-xl text-sm font-medium animate-fade-in
             {t.type === 'error' ? 'bg-[var(--color-kap-destruct)] text-white' : 
              t.type === 'success' ? 'bg-[var(--color-kap-success)] text-white' : 
              'bg-[var(--color-kap-surface2)] border border-[var(--color-kap-border)] text-[var(--color-kap-text)]'}"
      onclick={() => toast.remove(t.id)}
    >
      {#if t.type === 'error'}
        <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
      {:else if t.type === 'success'}
        <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
      {:else}
        <svg class="w-4 h-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
      {/if}
      {t.message}
    </div>
  {/each}
</div>
