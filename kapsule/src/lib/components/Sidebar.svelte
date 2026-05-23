<script lang="ts">
  /**
   * Sidebar.svelte — Left navigation rail with icon + label items.
   *
   * Uses SvelteKit's `page` store to highlight the active route.
   */
  import { page } from "$app/stores";

  interface NavItem {
    id: string;
    href: string;
    label: string;
    icon: string; // inline SVG path(s)
  }

  const navItems: NavItem[] = [
    {
      id: "nav-dashboard",
      href: "/",
      label: "Vessels",
      icon: `<rect x="2" y="7" width="20" height="14" rx="2"/><path d="M16 7V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2"/>`,
    },
    {
      id: "nav-library",
      href: "/library",
      label: "Library",
      icon: `<circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>`,
    },
    {
      id: "nav-settings",
      href: "/settings",
      label: "Settings",
      icon: `<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>`,
    },
  ];

  function isActive(href: string): boolean {
    if (href === "/") return $page.url.pathname === "/";
    return $page.url.pathname.startsWith(href);
  }
</script>

<nav class="flex flex-col gap-1 py-3 px-2 w-[68px] shrink-0"
     style="background: var(--color-kap-surface); border-right: 1px solid var(--color-kap-border);">
  {#each navItems as item}
    <a
      id={item.id}
      href={item.href}
      class="flex flex-col items-center gap-1 py-2.5 px-1 rounded-lg text-center transition-all duration-150 no-underline group
             {isActive(item.href)
               ? 'bg-[color-mix(in_srgb,var(--color-kap-accent)_20%,transparent)] text-[var(--color-kap-accent)]'
               : 'text-[var(--color-kap-muted)] hover:bg-[var(--color-kap-surface2)] hover:text-[var(--color-kap-text)]'}"
      title={item.label}
    >
      <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
        {@html item.icon}
      </svg>
      <span class="text-[9px] font-medium leading-none">{item.label}</span>
    </a>
  {/each}
</nav>
