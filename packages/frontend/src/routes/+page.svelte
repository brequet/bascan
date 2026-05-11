<script lang="ts">
  import { onMount } from "svelte";
  import { fetchSeries, imageUrl } from "$lib/api";
  import type { Series } from "$lib/types";

  let series: Series[] = $state([]);
  let loading = $state(true);

  onMount(async () => {
    series = await fetchSeries();
    loading = false;
  });
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
      {#each series as s}
        <a href="/series/{encodeURIComponent(s.id)}" class="volume-card">
          <div class="cover-wrapper">
            {#if s.coverUrl}
              <img src={imageUrl(s.coverUrl)} alt={s.title} loading="lazy" />
            {/if}
          </div>
          <span class="volume-title">{s.title}</span>
          <span class="page-count">{s.volumeCount} volumes</span>
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

  .volume-title {
    font-size: 0.85rem;
    font-weight: 500;
  }

  .page-count {
    font-size: 0.75rem;
    color: var(--text-muted);
  }
</style>
