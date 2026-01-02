<script lang="ts">
  interface Props {
    currentPage: number;
    totalPages: number;
    onPageChange: (page: number) => void;
  }

  let { currentPage, totalPages, onPageChange }: Props = $props();

  const canGoPrevious = $derived(currentPage > 1);
  const canGoNext = $derived(currentPage < totalPages);

  function goToPrevious() {
    if (canGoPrevious) {
      onPageChange(currentPage - 1);
    }
  }

  function goToNext() {
    if (canGoNext) {
      onPageChange(currentPage + 1);
    }
  }

  function goToPage(page: number) {
    if (page >= 1 && page <= totalPages) {
      onPageChange(page);
    }
  }

  type PageItem = { type: 'page'; number: number } | { type: 'ellipsis' };

  const visiblePages = $derived((): PageItem[] => {
    const items: PageItem[] = [];
    const maxVisible = 5;
    const halfVisible = Math.floor(maxVisible / 2);

    let start = Math.max(1, currentPage - halfVisible);
    let end = Math.min(totalPages, start + maxVisible - 1);

    if (end - start < maxVisible - 1) {
      start = Math.max(1, end - maxVisible + 1);
    }

    // Always show first page
    items.push({ type: 'page', number: 1 });

    // Add ellipsis if there's a gap between 1 and start
    if (start > 2) {
      items.push({ type: 'ellipsis' });
    }

    // Add middle pages (excluding first and last)
    for (let i = Math.max(2, start); i <= Math.min(totalPages - 1, end); i++) {
      items.push({ type: 'page', number: i });
    }

    // Add ellipsis if there's a gap between end and last page
    if (end < totalPages - 1) {
      items.push({ type: 'ellipsis' });
    }

    // Always show last page (if more than 1 page)
    if (totalPages > 1) {
      items.push({ type: 'page', number: totalPages });
    }

    return items;
  });
</script>

<div class="flex items-center justify-center gap-2 mt-4">
  <button
    onclick={goToPrevious}
    disabled={!canGoPrevious}
    class="px-3 py-1 rounded border border-gray-300 disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-100 transition-colors"
  >
    Previous
  </button>

  {#each visiblePages() as item}
    {#if item.type === 'page'}
      <button
        onclick={() => goToPage(item.number)}
        class="px-3 py-1 rounded border transition-colors {item.number ===
        currentPage
          ? 'bg-blue-500 text-white border-blue-500'
          : 'border-gray-300 hover:bg-gray-100'}"
      >
        {item.number}
      </button>
    {:else}
      <span class="px-3 py-1 text-gray-400">...</span>
    {/if}
  {/each}

  <button
    onclick={goToNext}
    disabled={!canGoNext}
    class="px-3 py-1 rounded border border-gray-300 disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-100 transition-colors"
  >
    Next
  </button>
</div>
