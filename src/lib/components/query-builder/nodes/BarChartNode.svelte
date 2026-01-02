<script module lang="ts">
  import type { Node } from '@xyflow/svelte';

  type BarChartNodeData = {
    xAxis?: string;
    yAxis?: string;
  };

  export type BarChartNodeType = Node<BarChartNodeData, 'barChart'> & {
    kind: 'chart';
  };
</script>

<script lang="ts">
  import { Handle, Position, type NodeProps } from '@xyflow/svelte';
  import { ChartColumn } from 'lucide-svelte';
  import QueryNode from './QueryNode.svelte';

  let { id, selected }: NodeProps<BarChartNodeType> = $props();
</script>

<QueryNode {id} {selected} borderColor="purple">
  <div class="flex items-center gap-1 font-semibold text-sm text-gray-700 mb-1">
    <ChartColumn class="w-4 h-4" />
    <span>Bar Chart</span>
  </div>

  <Handle type="target" position={Position.Left} class="!bg-purple-500" />
</QueryNode>
