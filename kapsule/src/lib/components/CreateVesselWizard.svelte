<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { uiState } from "$lib/stores/ui.svelte";

  let { isOpen, onClose, onSuccess } = $props<{
    isOpen: boolean;
    onClose: () => void;
    onSuccess: () => void;
  }>();

  let form = $state({
    name: "",
    image: "",
    isolated_home: true,
    volatile: false,
    disable_network: false,
    env_vars: [] as { key: string; value: string }[],
    ports: [] as { host: number; container: number }[],
    mounts: [] as { host_path: string; container_path: string; read_only: boolean; isDropdownOpen?: boolean }[],
  });

  let isSubmitting = $state(false);
  let errorMsg = $state("");
  let localImages = $state<string[]>([]);
  let isImageDropdownOpen = $state(false);

  $effect(() => {
    if (isOpen) {
      fetchImages();
    }
  });

  async function fetchImages() {
    try {
      localImages = await invoke<string[]>("list_local_images");
      if (localImages.length > 0 && !form.image) {
        form.image = localImages[0];
      }
    } catch (e) {
      console.error(e);
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    errorMsg = "";
    
    if (!form.name || !form.image) {
      errorMsg = "Name and Image are required.";
      return;
    }

    isSubmitting = true;
    try {
      await invoke("create_vessel", { payload: form });
      onSuccess();
      onClose();
      form = {
        name: "",
        image: form.image,
        isolated_home: true,
        volatile: false,
        disable_network: false,
        env_vars: [],
        ports: [],
        mounts: [],
      };
    } catch (err: any) {
      errorMsg = err.toString();
    } finally {
      isSubmitting = false;
    }
  }

  function addPort() {
    form.ports.push({ host: 8080, container: 80 });
  }

  function removePort(index: number) {
    form.ports.splice(index, 1);
  }

  function addMount() {
    form.mounts.push({ host_path: "", container_path: "", read_only: false });
  }

  function removeMount(index: number) {
    form.mounts.splice(index, 1);
  }

  function addEnv() {
    form.env_vars.push({ key: "", value: "" });
  }

  function removeEnv(index: number) {
    form.env_vars.splice(index, 1);
  }
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-fade-in" onclick={onClose}>
    <div 
      class="bg-[var(--color-kap-surface)] w-full max-w-2xl rounded-xl shadow-2xl border border-white/10 flex flex-col max-h-[90vh] overflow-hidden"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="px-6 py-4 border-b border-white/10 flex items-center justify-between bg-[var(--color-kap-surface)] z-10">
        <h2 class="text-lg font-semibold m-0">Create Vessel</h2>
        <button aria-label="Close wizard" class="btn btn-ghost p-1 rounded-full text-white/50 hover:text-white" onclick={onClose}>
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
        </button>
      </div>

      <!-- Scrollable Form Content -->
      <div class="overflow-y-auto flex-1 p-6 space-y-6">
        {#if errorMsg}
          <div class="bg-red-500/20 text-red-200 border border-red-500/50 rounded-lg p-3 text-sm">
            {errorMsg}
          </div>
        {/if}

        <div class="space-y-4">
          <h3 class="text-sm font-semibold uppercase tracking-wider text-white/50">General</h3>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="space-y-1.5">
              <label for="vessel-name" class="text-sm font-medium">Vessel Name</label>
              <input 
                id="vessel-name"
                type="text" 
                class="kap-input w-full" 
                placeholder="e.g. dev-node24" 
                bind:value={form.name} 
              />
            </div>
            
            <div class="space-y-1.5">
              <label class="text-sm font-medium">Container Image</label>
              <div class="relative">
                <button 
                  type="button"
                  class="kap-input w-full flex items-center justify-between text-left {isImageDropdownOpen ? 'rounded-b-none border-b-[var(--color-kap-border)]' : ''}" 
                  onclick={() => isImageDropdownOpen = !isImageDropdownOpen}
                >
                  <span class="truncate pr-4">
                    {#if form.image}
                      {form.image}
                    {:else}
                      <span class="text-[var(--color-kap-muted)]">Select an image...</span>
                    {/if}
                  </span>
                  <svg class="w-4 h-4 text-white/50 shrink-0 transition-transform {isImageDropdownOpen ? 'rotate-180' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
                </button>
                
                {#if isImageDropdownOpen}
                  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
                  <div class="fixed inset-0 z-40" onclick={() => isImageDropdownOpen = false}></div>
                  
                  <div class="absolute top-full left-0 right-0 z-50 bg-[var(--color-kap-surface2)] border border-[var(--color-kap-border)] border-t-0 rounded-b-[var(--radius-kap-sm)] shadow-xl max-h-60 overflow-y-auto kap-scrollbar flex flex-col">
                    {#if localImages.length === 0}
                      <div class="px-3 py-3 text-sm text-[var(--color-kap-muted)] italic">No local images found</div>
                    {/if}
                    {#each localImages as img}
                      <button 
                        type="button"
                        class="px-3 py-2.5 text-sm text-left hover:bg-white/5 transition-colors {form.image === img ? 'bg-white/10' : ''}"
                        onclick={() => { form.image = img; isImageDropdownOpen = false; }}
                      >
                        {img}
                      </button>
                    {/each}
                    <div class="h-px bg-white/10 my-1 mx-2"></div>
                    <button 
                      type="button"
                      class="px-3 py-2.5 text-sm text-left font-semibold text-[var(--color-kap-accent)] hover:bg-[var(--color-kap-accent)]/10 transition-colors"
                      onclick={() => { isImageDropdownOpen = false; onClose(); goto('/library'); }}
                    >
                      + Pull new image from Library...
                    </button>
                  </div>
                {/if}
              </div>
            </div>
          </div>
        </div>

        <div class="space-y-4">
          <h3 class="text-sm font-semibold uppercase tracking-wider text-white/50">Isolation & Settings</h3>
          
          <div class="bg-white/5 border border-white/10 rounded-lg divide-y divide-white/10">
            <div class="p-4 flex items-center justify-between">
              <div>
                <p class="font-medium text-sm">Isolated $HOME</p>
                <p class="text-xs text-white/50 mt-0.5">Generate and mount a unique home directory for this vessel.</p>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" bind:checked={form.isolated_home} class="sr-only peer">
                <div class="w-11 h-6 bg-white/10 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-kap-accent)]"></div>
              </label>
            </div>
            
            <div class="p-4 flex items-center justify-between">
              <div>
                <p class="font-medium text-sm">Volatile Mode (--rm)</p>
                <p class="text-xs text-white/50 mt-0.5">Automatically destroy the vessel when it exits.</p>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" bind:checked={form.volatile} class="sr-only peer">
                <div class="w-11 h-6 bg-white/10 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-kap-accent)]"></div>
              </label>
            </div>
            
            <div class="p-4 flex items-center justify-between">
              <div>
                <p class="font-medium text-sm text-[var(--color-kap-destruct)]">Air-Gap (Disable Network)</p>
                <p class="text-xs text-white/50 mt-0.5">Run without any internet or host network access.</p>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" bind:checked={form.disable_network} class="sr-only peer">
                <div class="w-11 h-6 bg-white/10 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-kap-destruct)]"></div>
              </label>
            </div>
          </div>
        </div>
        
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold uppercase tracking-wider text-white/50">Environment Variables</h3>
            <button class="text-xs text-[var(--color-kap-accent)] hover:underline" onclick={addEnv}>+ Add Env Var</button>
          </div>
          
          {#if form.env_vars.length === 0}
            <p class="text-xs text-white/40 italic">Host environment variables are automatically scrubbed.</p>
          {/if}
          
          <div class="space-y-2">
            {#each form.env_vars as env, i}
              <div class="flex items-center gap-2">
                <input type="text" class="kap-input flex-1 text-sm" placeholder="KEY (e.g. NODE_ENV)" bind:value={env.key} />
                <span class="text-white/50">=</span>
                <input type="text" class="kap-input flex-1 text-sm" placeholder="Value" bind:value={env.value} />
                <button aria-label="Remove env var" class="btn btn-ghost text-red-400 hover:bg-red-400/10 p-2" onclick={() => removeEnv(i)}>
                  <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
                </button>
              </div>
            {/each}
          </div>
        </div>

        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold uppercase tracking-wider text-white/50">Folder Mounts</h3>
            <button class="text-xs text-[var(--color-kap-accent)] hover:underline" onclick={addMount}>+ Add Mount</button>
          </div>
          
          {#if form.mounts.length === 0}
            <p class="text-xs text-white/40 italic">No extra folders mounted.</p>
          {/if}
          
          <div class="space-y-2">
            {#each form.mounts as mount, i}
              <div class="flex items-center gap-2 flex-wrap sm:flex-nowrap">
                <input type="text" class="kap-input flex-1 min-w-[120px] text-sm" placeholder="Host Path (/mnt/...)" bind:value={mount.host_path} />
                <span class="text-white/50 hidden sm:block">→</span>
                <input type="text" class="kap-input flex-1 min-w-[120px] text-sm" placeholder="Container Path (/app)" bind:value={mount.container_path} />
                
                <div class="relative w-20 shrink-0">
                  <button 
                    type="button"
                    class="kap-input w-full flex items-center justify-between text-left text-sm px-2 {mount.isDropdownOpen ? 'rounded-b-none border-b-[var(--color-kap-border)]' : ''}" 
                    onclick={() => mount.isDropdownOpen = !mount.isDropdownOpen}
                  >
                    <span>{mount.read_only ? 'RO' : 'RW'}</span>
                    <svg class="w-3 h-3 text-white/50 shrink-0 transition-transform {mount.isDropdownOpen ? 'rotate-180' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
                  </button>
                  
                  {#if mount.isDropdownOpen}
                    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
                    <div class="fixed inset-0 z-40" onclick={() => mount.isDropdownOpen = false}></div>
                    <div class="absolute top-full left-0 right-0 z-50 bg-[var(--color-kap-surface2)] border border-[var(--color-kap-border)] border-t-0 rounded-b-[var(--radius-kap-sm)] shadow-xl flex flex-col">
                      <button type="button" class="px-2 py-1.5 text-xs text-left hover:bg-white/5 {mount.read_only ? 'bg-white/10' : ''}" onclick={() => { mount.read_only = true; mount.isDropdownOpen = false; }}>RO</button>
                      <button type="button" class="px-2 py-1.5 text-xs text-left hover:bg-white/5 {!mount.read_only ? 'bg-white/10' : ''}" onclick={() => { mount.read_only = false; mount.isDropdownOpen = false; }}>RW</button>
                    </div>
                  {/if}
                </div>

                <button aria-label="Remove folder mount" class="btn btn-ghost text-red-400 hover:bg-red-400/10 p-2" onclick={() => removeMount(i)}>
                  <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
                </button>
              </div>
            {/each}
          </div>
        </div>

        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold uppercase tracking-wider text-white/50">Port Routing</h3>
            <button class="text-xs text-[var(--color-kap-accent)] hover:underline" onclick={addPort}>+ Add Port</button>
          </div>
          
          {#if form.ports.length === 0}
            <p class="text-xs text-white/40 italic">No ports mapped.</p>
          {/if}
          
          <div class="space-y-2">
            {#each form.ports as port, i}
              <div class="flex items-center gap-2">
                <input type="number" class="kap-input w-24 text-sm" placeholder="Host" bind:value={port.host} />
                <span class="text-white/50">→</span>
                <input type="number" class="kap-input w-24 text-sm" placeholder="Container" bind:value={port.container} />
                <button aria-label="Remove port mapping" class="btn btn-ghost text-red-400 hover:bg-red-400/10 p-2 ml-auto" onclick={() => removePort(i)}>
                  <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
                </button>
              </div>
            {/each}
          </div>
        </div>

      </div>

      <!-- Footer Actions -->
      <div class="p-4 border-t border-white/10 bg-[var(--color-kap-surface)] flex justify-end gap-3 z-10">
        <button class="btn btn-secondary px-6" onclick={onClose} disabled={isSubmitting}>Cancel</button>
        <button class="btn btn-primary px-8" onclick={handleSubmit} disabled={isSubmitting}>
          {isSubmitting ? 'Creating...' : 'Launch Vessel'}
        </button>
      </div>

    </div>
  </div>
{/if}
