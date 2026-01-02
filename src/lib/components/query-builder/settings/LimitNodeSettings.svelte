<script lang="ts">
  type LimitNodeData = {
    limit?: number;
  };

  type Props = {
    nodeData: LimitNodeData;
    onDataChange: (field: 'limit', value: any) => void;
  };

  let { nodeData, onDataChange }: Props = $props();
</script>

<div class="space-y-4">
  <h3 class="text-sm font-semibold text-gray-700 mb-3">Limit Settings</h3>

  <div>
    <label
      for="limit-input"
      class="block text-xs font-medium text-gray-700 mb-1"
    >
      Limit
    </label>
    <input
      id="limit-input"
      type="number"
      min="1"
      value={nodeData.limit || 10}
      onchange={(e) => onDataChange('limit', parseInt(e.currentTarget.value))}
      class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-green-500"
    />
  </div>
</div>
