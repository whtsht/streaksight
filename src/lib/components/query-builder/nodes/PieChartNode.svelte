<script module lang="ts">
  import type { Node } from '@xyflow/svelte';

  type PieChartNodeData = {
    categoryColumn?: string;
    valueColumn?: string;
  };

  export type PieChartNodeType = Node<PieChartNodeData, 'pieChart'> & {
    kind: 'chart';
  };
</script>

<script lang="ts">
  import { Handle, Position, type NodeProps } from '@xyflow/svelte';
  import { ChartPie } from 'lucide-svelte';
  import QueryNode from './QueryNode.svelte';

  let { id, selected }: NodeProps<PieChartNodeType> = $props();
</script>

<QueryNode {id} {selected} borderColor="purple">
  <div class="flex items-center gap-1 font-semibold text-sm text-gray-700 mb-1">
    <ChartPie class="w-4 h-4" />
    <span>Pie Chart</span>
  </div>

  <Handle type="target" position={Position.Left} class="!bg-purple-500" />
</QueryNode>
