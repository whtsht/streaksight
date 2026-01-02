<script lang="ts">
  import type { Metric } from '../nodes/AggregationNode.svelte';

  type AggregationNodeData = {
    dimensions?: string[];
    metrics?: Metric[];
  };

  type ColumnInfo = {
    name: string;
    type: string;
  };

  type Props = {
    nodeData: AggregationNodeData;
    previewColumns: ColumnInfo[];
    onDataChange: (field: 'dimensions' | 'metrics', value: any) => void;
  };

  let { nodeData, previewColumns, onDataChange }: Props = $props();

  let availableColumns = $state<ColumnInfo[]>([]);

  $effect(() => {
    if (availableColumns.length === 0 && previewColumns.length > 0) {
      availableColumns = previewColumns;
    }
  });

  const functions = ['COUNT(*)', 'COUNT', 'SUM', 'AVG', 'MAX', 'MIN'] as const;

  function toggleDimension(columnName: string) {
    if (availableColumns.length === 0) return;

    const currentDimensions = nodeData.dimensions || [];
    const newDimensions = currentDimensions.includes(columnName)
      ? currentDimensions.filter((c) => c !== columnName)
      : [...currentDimensions, columnName];

    onDataChange('dimensions', newDimensions);
  }

  function addMetric() {
    if (availableColumns.length === 0) return;

    const metrics = nodeData.metrics || [];
    const newMetric: Metric = {
      function: 'COUNT(*)',
      column: availableColumns[0].name
    };
    onDataChange('metrics', [...metrics, newMetric]);
  }

  function removeMetric(index: number) {
    const metrics = nodeData.metrics || [];
    onDataChange(
      'metrics',
      metrics.filter((_, i) => i !== index)
    );
  }

  function updateMetric(index: number, field: keyof Metric, value: any) {
    const metrics = nodeData.metrics || [];
    const updated = metrics.map((m, i) => {
      if (i === index) {
        return { ...m, [field]: value };
      }
      return m;
    });
    onDataChange('metrics', updated);
  }
</script>

<div class="space-y-4">
  <h3 class="text-sm font-semibold text-gray-700 mb-3">Aggregation Settings</h3>

  {#if availableColumns.length > 0}
    <div>
      <div class="block text-xs font-medium text-gray-700 mb-2">Dimensions</div>
      <div
        class="space-y-1 max-h-48 overflow-y-auto border border-gray-200 rounded p-2"
      >
        {#each availableColumns as column}
          {@const dimensions = nodeData.dimensions || []}
          {@const isChecked = dimensions.includes(column.name)}
          <label class="flex items-center space-x-2 text-xs">
            <input
              type="checkbox"
              checked={isChecked}
              onchange={() => toggleDimension(column.name)}
              class="rounded text-green-500 focus:ring-green-500"
            />
            <span>{column.name}</span>
          </label>
        {/each}
      </div>
    </div>
  {/if}

  <div>
    <div class="flex items-center justify-between mb-2">
      <div class="text-xs font-medium text-gray-700">Metrics</div>
      <button
        onclick={addMetric}
        disabled={availableColumns.length === 0}
        class="px-2 py-1 text-xs bg-green-500 text-white rounded hover:bg-green-600 disabled:bg-gray-300 disabled:cursor-not-allowed"
      >
        + Add Metric
      </button>
    </div>

    {#if nodeData.metrics && nodeData.metrics.length > 0}
      <div class="space-y-3">
        {#each nodeData.metrics as metric, index}
          <div class="border border-gray-200 rounded p-3 space-y-2">
            <div>
              <label
                for="metric-function-{index}"
                class="block text-xs font-medium text-gray-700 mb-1"
              >
                Function
              </label>
              <select
                id="metric-function-{index}"
                value={metric.function}
                onchange={(e) =>
                  updateMetric(index, 'function', e.currentTarget.value)}
                class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-green-500"
              >
                {#each functions as func}
                  <option value={func}>{func}</option>
                {/each}
              </select>
            </div>

            {#if metric.function !== 'COUNT(*)'}
              <div>
                <label
                  for="metric-column-{index}"
                  class="block text-xs font-medium text-gray-700 mb-1"
                >
                  Column
                </label>
                <select
                  id="metric-column-{index}"
                  value={metric.column}
                  onchange={(e) =>
                    updateMetric(index, 'column', e.currentTarget.value)}
                  class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-green-500"
                >
                  {#each availableColumns as col}
                    <option value={col.name}>{col.name}</option>
                  {/each}
                </select>
              </div>
            {/if}

            <div class="flex justify-end">
              <button
                onclick={() => removeMetric(index)}
                class="px-2 py-1 text-xs bg-red-500 text-white rounded hover:bg-red-600"
              >
                Remove
              </button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="text-xs text-gray-500 italic">
        No metrics added. Click "+ Add Metric" to start.
      </div>
    {/if}
  </div>
</div>
