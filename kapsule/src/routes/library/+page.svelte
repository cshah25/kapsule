<script lang="ts">
  import { invoke, Channel } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { toast } from "$lib/stores/toast.svelte";

  interface SearchResult {
    repo_name: string;
    short_description: string;
    star_count: number;
    is_official: boolean;
  }

  interface PullProgress {
    id?: string;
    status: string;
    progress: string;
    current?: number;
    total?: number;
  }

  let query = $state("");
  let searching = $state(false);
  let results = $state<SearchResult[]>([]);
  
  let selectedImage = $state<string | null>(null);
  let loadingTags = $state(false);
  let tags = $state<string[]>([]);
  let selectedTag = $state<string>("latest");
  let isTagDropdownOpen = $state(false);
  
  let pulling = $state(false);
  let pullLog = $state<PullProgress[]>([]);
  let layers = $state<Record<string, { current: number, total: number }>>({});
  let logContainer = $state<HTMLDivElement | undefined>();
  let forceComplete = $state(false);

  let totalBytes = $derived(Object.values(layers).reduce((acc, l) => acc + l.total, 0));
  let currentBytes = $derived(Object.values(layers).reduce((acc, l) => acc + l.current, 0));
  let progressPercent = $derived(forceComplete ? 100 : (totalBytes > 0 ? (currentBytes / totalBytes) * 100 : 0));

  let debounceTimer: ReturnType<typeof setTimeout>;

  onMount(() => {
    loadPopular();
  });

  function loadPopular() {
    results = [
      { repo_name: "ubuntu", short_description: "Ubuntu is a Debian-based Linux operating system based on free software.", star_count: 16000, is_official: true },
      { repo_name: "nginx", short_description: "Official build of Nginx.", star_count: 19000, is_official: true },
      { repo_name: "redis", short_description: "Redis is an open source key-value store that functions as a data structure server.", star_count: 12000, is_official: true },
      { repo_name: "node", short_description: "Node.js is a JavaScript-based platform for server-side and networking applications.", star_count: 13000, is_official: true },
      { repo_name: "postgres", short_description: "The PostgreSQL object-relational database system provides reliability and data integrity.", star_count: 13500, is_official: true },
      { repo_name: "python", short_description: "Python is an interpreted, interactive, object-oriented, open-source programming language.", star_count: 8500, is_official: true },
      { repo_name: "mysql", short_description: "MySQL is a widely used, open-source relational database management system.", star_count: 14000, is_official: true },
      { repo_name: "mongo", short_description: "MongoDB document databases provide high availability and easy scalability.", star_count: 9500, is_official: true },
      { repo_name: "alpine", short_description: "A minimal Docker image based on Alpine Linux with a complete package index and only 5 MB in size!", star_count: 10500, is_official: true },
      { repo_name: "busybox", short_description: "Busybox base image.", star_count: 3200, is_official: true },
      { repo_name: "golang", short_description: "Go (golang) is a general purpose, higher-level, imperative programming language.", star_count: 5100, is_official: true },
      { repo_name: "httpd", short_description: "The Apache HTTP Server Project", star_count: 4800, is_official: true }
    ];
    searching = false;
  }

  function onInput() {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      if (query.trim() === "") {
        loadPopular();
      } else {
        handleSearch();
      }
    }, 400); // 400ms debounce
  }

  async function handleSearch() {
    if (!query.trim()) return;
    searching = true;
    selectedImage = null;
    tags = [];
    try {
      results = await invoke<SearchResult[]>("search_images", { query: query.trim() });
    } catch (err) {
      toast.error(`Search failed: ${err}`);
    } finally {
      searching = false;
    }
  }

  async function selectImage(repo_name: string) {
    selectedImage = repo_name;
    loadingTags = true;
    tags = [];
    selectedTag = "latest";
    try {
      tags = await invoke<string[]>("get_image_tags", { image: repo_name });
      if (!tags.includes("latest") && tags.length > 0) {
        selectedTag = tags[0];
      }
    } catch (err) {
      toast.error(`Failed to load tags: ${err}`);
    } finally {
      loadingTags = false;
    }
  }

  async function handlePull() {
    if (!selectedImage) return;
    pulling = true;
    forceComplete = false;
    pullLog = [];
    layers = {};
    
    const channel = new Channel<PullProgress>();
    channel.onmessage = (msg) => {
      if (msg.status === "Done") {
        pulling = false;
        forceComplete = true;
        return;
      }
      
      if (msg.id && msg.total && msg.status === "Downloading") {
        layers[msg.id] = { current: msg.current || 0, total: msg.total };
      }
      
      pullLog = [...pullLog, msg];
      // Keep only last 200 logs so it doesn't flood UI
      if (pullLog.length > 200) pullLog = pullLog.slice(pullLog.length - 200);
      
      setTimeout(() => {
        if (logContainer) logContainer.scrollTop = logContainer.scrollHeight;
      }, 10);
    };

    const imageToPull = `${selectedImage}:${selectedTag}`;
    try {
      await invoke("pull_image", { image: imageToPull, onMessage: channel });
    } catch (err) {
      toast.error(`Pull failed: ${err}`);
      pulling = false;
    }
  }
</script>

<svelte:head>
  <title>Library — Kapsule</title>
</svelte:head>

<div class="flex flex-col gap-6 animate-fade-in h-full">
  <div>
    <h1 class="text-lg font-semibold">Library (Registry Explorer)</h1>
    <p class="text-sm text-[var(--color-kap-muted)] mt-1">
      Browse and pull images from Docker Hub.
    </p>
  </div>

  <form class="flex gap-2 max-w-2xl" onsubmit={(e) => { e.preventDefault(); handleSearch(); }}>
    <input
      id="library-search"
      class="kap-input flex-1"
      placeholder="Search Docker Hub… e.g. node, ubuntu, nginx"
      bind:value={query}
      oninput={onInput}
    />
    <button type="submit" class="btn btn-primary px-4" disabled={searching}>
      {#if searching}
        <div class="w-4 h-4 rounded-full border-2 border-white/20 border-t-white animate-spin"></div>
      {:else}
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
      {/if}
      Search
    </button>
  </form>

  <div class="flex flex-1 gap-6 min-h-0">
    <!-- Results grid -->
    <div class="flex-1 overflow-y-auto pr-2 kap-scrollbar grid gap-4 content-start" style="grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));">
      {#each results as res}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div 
          class="kap-card p-4 flex flex-col gap-3 cursor-pointer transition-colors duration-200 hover:border-[var(--color-kap-accent)]
                 {selectedImage === res.repo_name ? 'border-[var(--color-kap-accent)] bg-[var(--color-kap-accent)]/5' : ''}"
          onclick={() => selectImage(res.repo_name)}
        >
          <div class="flex items-start justify-between gap-2">
            <h3 class="font-medium text-[var(--color-kap-text)] flex flex-wrap items-center gap-2 break-all">
              {res.repo_name}
              {#if res.is_official}
                <span class="badge badge-running text-[10px] py-0 shrink-0">Official</span>
              {/if}
            </h3>
            <span class="text-xs text-[var(--color-kap-muted)] flex items-center gap-1 shrink-0 mt-1">
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
              {res.star_count}
            </span>
          </div>
          <p class="text-xs text-[var(--color-kap-muted)] line-clamp-3 mt-auto">{res.short_description}</p>
        </div>
      {/each}
      
      {#if !searching && query && results.length === 0}
        <div class="col-span-full p-8 text-center text-sm text-[var(--color-kap-muted)]">No results found.</div>
      {/if}
    </div>

    <!-- Inspector pane -->
    {#if selectedImage}
      <div class="w-80 kap-card p-5 flex flex-col gap-6 animate-fade-in shrink-0 overflow-y-auto kap-scrollbar">
        <div>
          <h2 class="font-semibold text-lg break-words">{selectedImage}</h2>
        </div>

        <div class="flex flex-col gap-2">
          <label class="text-xs font-semibold uppercase tracking-wider text-[var(--color-kap-muted)]">Select Tag</label>
          {#if loadingTags}
            <div class="text-sm text-[var(--color-kap-muted)] flex items-center gap-2">
              <div class="w-3 h-3 rounded-full border-2 border-white/20 border-t-white animate-spin"></div>
              Loading tags...
            </div>
          {:else}
            <div class="relative">
              <button 
                type="button"
                class="kap-input w-full flex items-center justify-between text-left {isTagDropdownOpen ? 'rounded-b-none border-b-[var(--color-kap-border)]' : ''}" 
                onclick={() => isTagDropdownOpen = !isTagDropdownOpen}
                disabled={pulling}
              >
                <span class="truncate pr-4">{selectedTag}</span>
                <svg class="w-4 h-4 text-white/50 shrink-0 transition-transform {isTagDropdownOpen ? 'rotate-180' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
              </button>
              
              {#if isTagDropdownOpen}
                <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
                <div class="fixed inset-0 z-40" onclick={() => isTagDropdownOpen = false}></div>
                <div class="absolute top-full left-0 right-0 z-50 bg-[var(--color-kap-surface2)] border border-[var(--color-kap-border)] border-t-0 rounded-b-[var(--radius-kap-sm)] shadow-xl max-h-60 overflow-y-auto kap-scrollbar flex flex-col">
                  {#each tags as tag}
                    <button 
                      type="button"
                      class="px-3 py-2 text-sm text-left hover:bg-white/5 {selectedTag === tag ? 'bg-white/10' : ''}"
                      onclick={() => { selectedTag = tag; isTagDropdownOpen = false; }}
                    >
                      {tag}
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <button 
          class="btn btn-primary w-full justify-center" 
          onclick={handlePull}
          disabled={pulling || loadingTags || tags.length === 0}
        >
          {pulling ? 'Pulling...' : 'Pull Image'}
        </button>

        {#if pulling || pullLog.length > 0}
          <div class="flex flex-col gap-2 mt-2">
            <div class="flex items-center justify-between">
              <span class="text-xs font-semibold uppercase tracking-wider text-[var(--color-kap-muted)] shrink-0">Pull Progress</span>
              {#if pulling || progressPercent > 0}
                <span class="text-xs font-mono text-[var(--color-kap-muted)]">{Math.round(progressPercent)}%</span>
              {/if}
            </div>
            
            <div class="h-1.5 w-full bg-black/40 rounded-full overflow-hidden border border-black/20">
              <div 
                class="h-full bg-[var(--color-kap-accent)] transition-all duration-300"
                style="width: {progressPercent}%"
              ></div>
            </div>

            <div bind:this={logContainer} class="bg-[var(--color-kap-window)] rounded-md p-3 font-mono text-[10px] text-[var(--color-kap-muted)] flex flex-col gap-1 overflow-y-auto kap-scrollbar h-64">
              {#each pullLog as log}
                <div class="whitespace-nowrap overflow-hidden text-ellipsis shrink-0">
                  <span class="text-white/80">{log.status}</span> {log.progress}
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
