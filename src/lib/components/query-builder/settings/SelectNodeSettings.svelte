<script lang="ts">
  type SelectNodeData = {
    columns?: string[];
  };

  type ColumnInfo = {
    name: string;
    type: string;
  };

  type Props = {
    nodeData: SelectNodeData;
    previewColumns: ColumnInfo[];
    onDataChange: (field: 'columns', value: any) => void;
  };

  let { nodeData, previewColumns, onDataChange }: Props = $props();

  let availableColumns = $state<ColumnInfo[]>([]);

  $effect(() => {
    if (availableColumns.length === 0 && previewColumns.length > 0) {
      availableColumns = previewColumns;
    }
  });

  function toggleColumn(columnName: string) {
    if (availableColumns.length === 0) return;

    let currentColumns = nodeData.columns || [];

    if (currentColumns.length === 0) {
      currentColumns = availableColumns.map((c) => c.name);
    }

    const newColumns = currentColumns.includes(columnName)
      ? currentColumns.filter((c) => c !== columnName)
      : [...currentColumns, columnName];

    onDataChange('columns', newColumns);
  }
</script>

<div class="space-y-4">
  <h3 class="text-sm font-semibold text-gray-700 mb-3">Select Settings</h3>

  <!-- Columns Selection -->
  {#if availableColumns.length > 0}
    <div>
      <div class="block text-xs font-medium text-gray-700 mb-2">Columns</div>
      <div
        class="space-y-1 max-h-48 overflow-y-auto border border-gray-200 rounded p-2"
      >
        {#each availableColumns as column}
          {@const columns = nodeData.columns || []}
          {@const isChecked =
            columns.length === 0 ? true : columns.includes(column.name)}
          <label class="flex items-center space-x-2 text-xs">
            <input
              type="checkbox"
              checked={isChecked}
              onchange={() => toggleColumn(column.name)}
              class="rounded text-green-500 focus:ring-green-500"
            />
            <span>{column.name}</span>
          </label>
        {/each}
      </div>
    </div>
  {/if}
</div>
