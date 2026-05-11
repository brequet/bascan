<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto, replaceState } from "$app/navigation";
  import { fetchPages, imageUrl } from "$lib/api";
  import { getProgress, saveProgress } from "$lib/progress";
  import type { Page } from "@bascan/shared";

  let volumeId: string = $state("");
  let pages: Page[] = $state([]);
  let currentIndex: number = $state(0);
  let loading = $state(true);
  let mode: "scroll" | "page" = $state("page");
  let isFullscreen = $state(false);
  let zoom = $state(1);
  let showControls = $state(true);
  let controlsTimeout: ReturnType<typeof setTimeout> | null = null;
  let scrollContainer: HTMLDivElement | undefined = $state(undefined);

  // Derived
  let currentPage = $derived(pages[currentIndex]);
  let progress = $derived(pages.length ? Math.round(((currentIndex + 1) / pages.length) * 100) : 0);

  onMount(() => {
    const id = $page.params?.id;
    if (id) {
      volumeId = decodeURIComponent(id);
      // Read initial page from URL search params
      const urlPage = new URLSearchParams(window.location.search).get("p");
      if (urlPage) currentIndex = Math.max(0, parseInt(urlPage, 10) - 1);
      loadVolume();
    }
  });

  async function loadVolume() {
    loading = true;
    pages = await fetchPages(volumeId);
    // Restore from URL param first, then localStorage
    if (currentIndex === 0) {
      const saved = getProgress(volumeId);
      if (saved) currentIndex = Math.min(saved.pageIndex, pages.length - 1);
    } else {
      currentIndex = Math.min(currentIndex, pages.length - 1);
    }
    updateUrl();
    loading = false;
  }

  function updateUrl() {
    const url = new URL(window.location.href);
    url.searchParams.set("p", String(currentIndex + 1));
    replaceState(url.toString(), {});
  }

  function goTo(index: number) {
    if (index < 0 || index >= pages.length) return;
    currentIndex = index;
    saveProgress(volumeId, currentIndex);
    updateUrl();
    if (mode === "scroll") scrollToPage(index);
  }

  function scrollToPage(index: number) {
    if (!scrollContainer) return;
    requestAnimationFrame(() => {
      const images = scrollContainer!.querySelectorAll<HTMLElement>(".page-img");
      if (images[index]) {
        images[index].scrollIntoView({ behavior: "instant", block: "start" });
      }
    });
  }

  function prev() { goTo(currentIndex - 1); }
  function next() { goTo(currentIndex + 1); }

  function toggleMode() {
    mode = mode === "scroll" ? "page" : "scroll";
    if (mode === "scroll") {
      // Wait for DOM to render scroll view, then scroll to current page
      requestAnimationFrame(() => scrollToPage(currentIndex));
    }
  }

  function toggleFullscreen() {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen();
      isFullscreen = true;
    } else {
      document.exitFullscreen();
      isFullscreen = false;
    }
  }

  function handleKey(e: KeyboardEvent) {
    switch (e.key) {
      case "ArrowLeft": prev(); break;
      case "ArrowRight": next(); break;
      case "f": case "F": toggleFullscreen(); break;
      case "+": case "=": zoom = Math.min(zoom + 0.25, 3); break;
      case "-": zoom = Math.max(zoom - 0.25, 0.5); break;
      case "0": zoom = 1; break;
      case "Escape": goto("/"); break;
      case "m": case "M": toggleMode(); break;
    }
    // Don't show controls on page turns — only on mouse move
  }

  let showCursor = $state(true);

  function showControlsBriefly() {
    showControls = true;
    showCursor = true;
    if (controlsTimeout) clearTimeout(controlsTimeout);
    controlsTimeout = setTimeout(() => {
      showControls = false;
      showCursor = false;
    }, 3000);
  }

  function handleMouseMove() {
    showControlsBriefly();
  }

  function handleScroll() {
    if (mode !== "scroll" || !scrollContainer) return;
    // Determine which page is in viewport center
    const centerY = scrollContainer.scrollTop + scrollContainer.clientHeight / 2;
    const images = scrollContainer.querySelectorAll<HTMLElement>(".page-img");
    for (let i = 0; i < images.length; i++) {
      const img = images[i];
      if (img.offsetTop + img.offsetHeight > centerY) {
        if (i !== currentIndex) {
          currentIndex = i;
          saveProgress(volumeId, currentIndex);
          updateUrl();
        }
        break;
      }
    }
  }

</script>

<svelte:window onkeydown={handleKey} />

<div class="reader" class:hide-cursor={!showCursor} onmousemove={handleMouseMove} role="application">
  {#if loading}
    <p class="loading-msg">Loading...</p>
  {:else}
    <!-- Top bar -->
    <header class="topbar" class:visible={showControls}>
      <a href="/" class="back-btn">← Library</a>
      <span class="title">{volumeId}</span>
      <span class="page-info">{currentIndex + 1} / {pages.length}</span>
    </header>

    <!-- Reader content -->
    {#if mode === "scroll"}
      <div
        class="scroll-view"
        bind:this={scrollContainer}
        onscroll={handleScroll}
        style="--zoom: {zoom}"
      >
        {#each pages as p, i}
          <img
            class="page-img"
            class:spread={p.isSpread}
            src={imageUrl(p.url)}
            alt="Page {i + 1}"
            loading={Math.abs(i - currentIndex) < 5 ? "eager" : "lazy"}
          />
        {/each}
      </div>
    {:else}
      <div class="page-view" style="--zoom: {zoom}">
        <!-- Click zones for prev/next -->
        <button class="click-zone left" onclick={prev} aria-label="Previous page"></button>
        <button class="click-zone right" onclick={next} aria-label="Next page"></button>

        {#if currentPage}
          <img
            class="page-img"
            class:spread={currentPage.isSpread}
            src={imageUrl(currentPage.url)}
            alt="Page {currentIndex + 1}"
          />
        {/if}
      </div>
    {/if}

    <!-- Bottom controls -->
    <footer class="bottombar" class:visible={showControls}>
      <div class="progress-bar">
        <div class="progress-fill" style="width: {progress}%"></div>
      </div>
      <div class="controls">
        <button onclick={prev} disabled={currentIndex === 0}>←</button>
        <button onclick={toggleMode}>
          {mode === "scroll" ? "📖 Page" : "📜 Scroll"}
        </button>
        <button onclick={() => { zoom = Math.max(zoom - 0.25, 0.5); }}>−</button>
        <span class="zoom-label">{Math.round(zoom * 100)}%</span>
        <button onclick={() => { zoom = Math.min(zoom + 0.25, 3); }}>+</button>
        <button onclick={toggleFullscreen}>{isFullscreen ? "⊡" : "⊞"} Full</button>
        <button onclick={next} disabled={currentIndex === pages.length - 1}>→</button>
      </div>
    </footer>
  {/if}
</div>

<style>
  .reader {
    height: 100vh;
    width: 100vw;
    display: flex;
    flex-direction: column;
    background: #000;
    position: relative;
    overflow: hidden;
  }

  .reader.hide-cursor {
    cursor: none;
  }

  .loading-msg {
    margin: auto;
    color: var(--text-muted);
  }

  /* Top bar */
  .topbar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1.5rem;
    background: linear-gradient(to bottom, rgba(0,0,0,0.85), transparent);
    opacity: 0;
    transition: opacity 0.3s;
    pointer-events: none;
  }

  .topbar.visible {
    opacity: 1;
    pointer-events: all;
  }

  .back-btn {
    color: var(--text);
    font-weight: 500;
  }

  .title {
    font-size: 0.9rem;
    color: var(--text-muted);
  }

  .page-info {
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  /* Scroll mode */
  .scroll-view {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 60px 0;
  }

  .scroll-view .page-img {
    width: calc(min(100vw, 800px) * var(--zoom));
    height: auto;
    display: block;
  }

  .scroll-view .page-img.spread {
    width: calc(min(100vw, 1200px) * var(--zoom));
  }

  /* Page mode */
  .page-view {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  .page-view .page-img {
    max-height: calc(100vh * var(--zoom));
    max-width: calc(100vw * var(--zoom));
    object-fit: contain;
  }

  .page-view .page-img.spread {
    max-width: calc(100vw * var(--zoom));
  }

  .click-zone {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 35%;
    background: none;
    border: none;
    cursor: pointer;
    z-index: 10;
  }

  .click-zone.left { left: 0; }
  .click-zone.right { right: 0; }

  /* Bottom bar */
  .bottombar {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 100;
    background: linear-gradient(to top, rgba(0,0,0,0.85), transparent);
    padding: 1rem 1.5rem 0.75rem;
    opacity: 0;
    transition: opacity 0.3s;
    pointer-events: none;
  }

  .bottombar.visible {
    opacity: 1;
    pointer-events: all;
  }

  .progress-bar {
    height: 3px;
    background: rgba(255,255,255,0.15);
    border-radius: 2px;
    margin-bottom: 0.75rem;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    transition: width 0.2s;
  }

  .controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
  }

  .controls button {
    background: rgba(255,255,255,0.1);
    border: none;
    color: var(--text);
    padding: 0.4rem 0.8rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
  }

  .controls button:hover { background: rgba(255,255,255,0.2); }
  .controls button:disabled { opacity: 0.3; cursor: default; }

  .zoom-label {
    font-size: 0.8rem;
    color: var(--text-muted);
    min-width: 3em;
    text-align: center;
  }
</style>
