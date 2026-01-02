<script lang="ts">
  type BarChartNodeData = {
    xAxis?: string;
    yAxis?: string;
  };

  type ColumnInfo = {
    name: string;
    type: string;
  };

  type Props = {
    nodeData: BarChartNodeData;
    previewColumns: ColumnInfo[];
    onDataChange: (field: 'xAxis' | 'yAxis', value: string) => void;
  };

  let { nodeData, previewColumns, onDataChange }: Props = $props();
</script>

<div class="space-y-4">
  <h3 class="text-sm font-semibold text-gray-700 mb-3">Bar Chart Settings</h3>

  {#if previewColumns.length > 0}
    <div>
      <label
        for="x-axis-select"
        class="block text-xs font-medium text-gray-700 mb-1"
      >
        X Axis
      </label>
      <select
        id="x-axis-select"
        value={nodeData.xAxis || ''}
        onchange={(e) => onDataChange('xAxis', e.currentTarget.value)}
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
        for="y-axis-select"
        class="block text-xs font-medium text-gray-700 mb-1"
      >
        Y Axis
      </label>
      <select
        id="y-axis-select"
        value={nodeData.yAxis || ''}
        onchange={(e) => onDataChange('yAxis', e.currentTarget.value)}
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
      Connect a data source to configure axes
    </div>
  {/if}
</div>
