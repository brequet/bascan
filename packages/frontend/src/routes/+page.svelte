<script lang="ts">
  import { onMount } from "svelte";
  import { fetchVolumes, imageUrl } from "$lib/api";
  import { getProgress } from "$lib/progress";
  import type { Volume } from "@bascan/shared";

  let volumes: Volume[] = $state([]);
  let loading = $state(true);

  onMount(async () => {
    volumes = await fetchVolumes();
    loading = false;
  });

  function progressLabel(vol: Volume): string {
    const p = getProgress(vol.id);
    if (!p) return "";
    const pct = Math.round((p.pageIndex / vol.pageCount) * 100);
    return `${pct}%`;
  }
</script>

<div class="library">
  <header class="library-header">
    <h1>Bascan</h1>
    <p class="subtitle">Local Manga Reader</p>
  </header>

  {#if loading}
    <p class="loading">Loading library...</p>
  {:else}
    <div class="grid">
      {#each volumes as vol}
        <a href="/read/{encodeURIComponent(vol.id)}" class="volume-card">
          <div class="cover-wrapper">
            <img src={imageUrl(vol.coverUrl)} alt={vol.title} loading="lazy" />
            {#if progressLabel(vol)}
              <span class="progress-badge">{progressLabel(vol)}</span>
            {/if}
          </div>
          <span class="volume-title">{vol.title}</span>
          <span class="page-count">{vol.pageCount} pages</span>
        </a>
      {/each}
    </div>
  {/if}
</div>

<style>
  .library {
    height: 100vh;
    overflow-y: auto;
    padding: 2rem;
  }

  .library-header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .library-header h1 {
    font-size: 2rem;
    font-weight: 700;
  }

  .subtitle {
    color: var(--text-muted);
    margin-top: 0.25rem;
  }

  .loading {
    text-align: center;
    color: var(--text-muted);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .volume-card {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    text-decoration: none;
    color: var(--text);
    transition: transform 0.15s;
  }

  .volume-card:hover {
    transform: translateY(-4px);
    text-decoration: none;
  }

  .cover-wrapper {
    position: relative;
    aspect-ratio: 2/3;
    border-radius: var(--radius);
    overflow: hidden;
    background: var(--bg-surface);
  }

  .cover-wrapper img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .progress-badge {
    position: absolute;
    top: 8px;
    right: 8px;
    background: var(--accent);
    color: white;
    font-size: 0.7rem;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .volume-title {
    font-size: 0.85rem;
    font-weight: 500;
  }

  .page-count {
    font-size: 0.75rem;
    color: var(--text-muted);
  }
</style>
