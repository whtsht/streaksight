<script lang="ts">
  import Pagination from '$lib/components/ui/pagination/Pagination.svelte';

  type ColumnInfo = {
    name: string;
  };

  type Props = {
    data: Array<Record<string, any>>;
    columns: ColumnInfo[];
    rowCount: number;
    currentPage?: number;
    totalPages?: number;
    totalRows?: number;
    onPageChange?: (page: number) => void;
  };

  let {
    data,
    columns,
    rowCount,
    currentPage = 1,
    totalPages = 1,
    totalRows,
    onPageChange
  }: Props = $props();

  const showPagination = $derived(totalPages > 1 && onPageChange !== undefined);
  const pageStart = $derived((currentPage - 1) * rowCount + 1);
  const pageEnd = $derived(
    Math.min(currentPage * rowCount, totalRows ?? rowCount)
  );
</script>

{#if showPagination && onPageChange}
  <Pagination {currentPage} {totalPages} {onPageChange} />
{/if}

<div class="text-xs text-gray-500 mb-2">
  {#if totalRows !== undefined}
    Showing {pageStart}-{pageEnd} of {totalRows} rows
  {:else}
    {rowCount} rows
  {/if}
</div>
<div class="overflow-x-auto">
  <table class="min-w-full text-xs border-collapse">
    <thead>
      <tr class="bg-gray-100">
        {#each columns as column}
          <th class="border border-gray-300 px-2 py-1 text-left font-medium">
            {column.name}
          </th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each data as row}
        <tr class="hover:bg-gray-50">
          {#each columns as column}
            <td class="border border-gray-300 px-2 py-1">
              {row[column.name] ?? 'NULL'}
            </td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>
