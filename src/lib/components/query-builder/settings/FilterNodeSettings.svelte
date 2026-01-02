<script lang="ts">
  import type { FilterCondition } from '../nodes/FilterNode.svelte';

  type FilterNodeData = {
    conditions?: FilterCondition[];
  };

  type ColumnInfo = {
    name: string;
    type: string;
  };

  type Props = {
    nodeData: FilterNodeData;
    previewColumns: ColumnInfo[];
    onDataChange: (field: 'conditions', value: any) => void;
  };

  let { nodeData, previewColumns, onDataChange }: Props = $props();

  let availableColumns = $state<ColumnInfo[]>([]);

  $effect(() => {
    if (availableColumns.length === 0 && previewColumns.length > 0) {
      availableColumns = previewColumns;
    }
  });

  const operators = ['==', '!=', '>', '<', '>=', '<=', 'in'] as const;

  function addCondition() {
    if (availableColumns.length === 0) return;

    const conditions = nodeData.conditions || [];
    const newCondition: FilterCondition = {
      column: availableColumns[0].name,
      operator: '==',
      value: '',
      negate: false
    };
    onDataChange('conditions', [...conditions, newCondition]);
  }

  function removeCondition(index: number) {
    const conditions = nodeData.conditions || [];
    onDataChange(
      'conditions',
      conditions.filter((_, i) => i !== index)
    );
  }

  function updateCondition(
    index: number,
    field: keyof FilterCondition,
    value: any
  ) {
    const conditions = nodeData.conditions || [];
    const updated = conditions.map((c, i) => {
      if (i === index) {
        return { ...c, [field]: value };
      }
      return c;
    });
    onDataChange('conditions', updated);
  }

  function parseValue(operator: string, valueStr: string): string | string[] {
    if (operator === 'in') {
      return valueStr
        .split(',')
        .map((v) => v.trim())
        .filter((v) => v !== '');
    }
    return valueStr;
  }

  function formatValue(value: string | string[]): string {
    if (Array.isArray(value)) {
      return value.join(', ');
    }
    return value;
  }
</script>

<div class="space-y-4">
  <h3 class="text-sm font-semibold text-gray-700 mb-3">Filter Settings</h3>
  <button
    onclick={addCondition}
    disabled={availableColumns.length === 0}
    class="w-full px-3 py-2 text-sm bg-green-500 text-white rounded hover:bg-green-600 disabled:bg-gray-300 disabled:cursor-not-allowed"
  >
    Add Condition
  </button>

  {#if nodeData.conditions && nodeData.conditions.length > 0}
    <div class="space-y-3">
      {#each nodeData.conditions as condition, index}
        <div class="border border-gray-200 rounded p-3 space-y-2">
          <div>
            <label
              for="filter-column-{index}"
              class="block text-xs font-medium text-gray-700 mb-1"
            >
              Column
            </label>
            <select
              id="filter-column-{index}"
              value={condition.column}
              onchange={(e) =>
                updateCondition(index, 'column', e.currentTarget.value)}
              class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-green-500"
            >
              {#each availableColumns as col}
                <option value={col.name}>{col.name}</option>
              {/each}
            </select>
          </div>

          <div>
            <label
              for="filter-operator-{index}"
              class="block text-xs font-medium text-gray-700 mb-1"
            >
              Operator
            </label>
            <select
              id="filter-operator-{index}"
              value={condition.operator}
              onchange={(e) =>
                updateCondition(index, 'operator', e.currentTarget.value)}
              class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-green-500"
            >
              {#each operators as op}
                <option value={op}>{op}</option>
              {/each}
            </select>
          </div>

          <div>
            <label
              for="filter-value-{index}"
              class="block text-xs font-medium text-gray-700 mb-1"
            >
              Value {condition.operator === 'in' ? '(comma-separated)' : ''}
            </label>
            <input
              id="filter-value-{index}"
              type="text"
              value={formatValue(condition.value)}
              oninput={(e) =>
                updateCondition(
                  index,
                  'value',
                  parseValue(condition.operator, e.currentTarget.value)
                )}
              class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-green-500"
              placeholder={condition.operator === 'in'
                ? 'value1, value2, value3'
                : 'Enter value'}
            />
          </div>

          <div class="flex items-center justify-between">
            <label class="flex items-center space-x-2 text-xs">
              <input
                type="checkbox"
                checked={condition.negate || false}
                onchange={(e) =>
                  updateCondition(index, 'negate', e.currentTarget.checked)}
                class="rounded text-green-500 focus:ring-green-500"
              />
              <span>NOT</span>
            </label>

            <button
              onclick={() => removeCondition(index)}
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
      No conditions added. Click "Add Condition" to start.
    </div>
  {/if}
</div>
