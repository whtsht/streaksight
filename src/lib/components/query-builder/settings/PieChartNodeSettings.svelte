<script lang="ts">
  type PieChartNodeData = {
    categoryColumn?: string;
    valueColumn?: string;
  };

  type ColumnInfo = {
    name: string;
    type: string;
  };

  type Props = {
    nodeData: PieChartNodeData;
    previewColumns: ColumnInfo[];
    onDataChange: (
      field: 'categoryColumn' | 'valueColumn',
      value: string
    ) => void;
  };

  let { nodeData, previewColumns, onDataChange }: Props = $props();
</script>

<div class="space-y-4">
  <h3 class="text-sm font-semibold text-gray-700 mb-3">Pie Chart Settings</h3>

  {#if previewColumns.length > 0}
    <div>
      <label
        for="category-column-select"
        class="block text-xs font-medium text-gray-700 mb-1"
      >
        Category Column
      </label>
      <select
        id="category-column-select"
        value={nodeData.categoryColumn || ''}
        onchange={(e) => onDataChange('categoryColumn', e.currentTarget.value)}
        class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-purple-500"
      >
        <option value="">Select column...</option>
        {#each previewColumns as column}
          <option value={column.name}>{column.name}</option>
        {/each}
      </select>
    </div>

    <div>
      <label
        for="value-column-select"
        class="block text-xs font-medium text-gray-700 mb-1"
      >
        Value Column
      </label>
      <select
        id="value-column-select"
        value={nodeData.valueColumn || ''}
        onchange={(e) => onDataChange('valueColumn', e.currentTarget.value)}
        class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-purple-500"
      >
        <option value="">Select column...</option>
        {#each previewColumns as column}
          <option value={column.name}>{column.name}</option>
        {/each}
      </select>
    </div>
  {:else}
    <div class="text-xs text-gray-500">
      Connect a data source to configure columns
    </div>
  {/if}
</div>
