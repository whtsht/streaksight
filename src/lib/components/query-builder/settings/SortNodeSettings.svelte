<script lang="ts">
  type OrderBy = {
    column: string;
    direction: string;
  };

  type SortNodeData = {
    order?: OrderBy[];
  };

  type ColumnInfo = {
    name: string;
    type: string;
  };

  type Props = {
    nodeData: SortNodeData;
    previewColumns: ColumnInfo[];
    onDataChange: (field: 'order', value: any) => void;
  };

  let { nodeData, previewColumns, onDataChange }: Props = $props();

  let availableColumns = $state<ColumnInfo[]>([]);

  $effect(() => {
    if (previewColumns.length > 0) {
      availableColumns = previewColumns;
    }
  });

  function addOrderBy() {
    if (availableColumns.length === 0) return;

    const currentOrder = nodeData.order || [];
    const newOrder = [
      ...currentOrder,
      { column: availableColumns[0].name, direction: 'asc' }
    ];
    onDataChange('order', newOrder);
  }

  function updateOrderBy(
    index: number,
    field: 'column' | 'direction',
    value: string
  ) {
    const currentOrder = nodeData.order || [];
    const newOrder = currentOrder.map((item, i) =>
      i === index ? { ...item, [field]: value } : item
    );
    onDataChange('order', newOrder);
  }

  function removeOrderBy(index: number) {
    const currentOrder = nodeData.order || [];
    const newOrder = currentOrder.filter((_, i) => i !== index);
    onDataChange('order', newOrder);
  }
</script>

<div class="space-y-4">
  <h3 class="text-sm font-semibold text-gray-700 mb-3">Sort Settings</h3>

  {#if availableColumns.length > 0}
    <div>
      <div class="flex items-center justify-between mb-2">
        <button
          onclick={addOrderBy}
          class="px-2 py-1 text-xs bg-green-500 text-white rounded hover:bg-green-600"
        >
          Add +
        </button>
      </div>
      <div class="space-y-2">
        {#each nodeData.order || [] as orderItem, index}
          <div class="flex items-center space-x-2">
            <select
              value={orderItem.column}
              onchange={(e) =>
                updateOrderBy(index, 'column', e.currentTarget.value)}
              class="flex-1 px-2 py-1 text-xs border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-green-500"
            >
              {#each availableColumns as column}
                <option value={column.name}>{column.name}</option>
              {/each}
            </select>
            <select
              value={orderItem.direction}
              onchange={(e) =>
                updateOrderBy(index, 'direction', e.currentTarget.value)}
              class="px-2 py-1 text-xs border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-green-500"
            >
              <option value="asc">ASC</option>
              <option value="desc">DESC</option>
            </select>
            <button
              onclick={() => removeOrderBy(index)}
              class="px-2 py-1 text-xs bg-red-500 text-white rounded hover:bg-red-600"
            >
              ×
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
