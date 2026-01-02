<script module lang="ts">
  import type { Node } from '@xyflow/svelte';

  type LineChartNodeData = {
    xAxis?: string;
    yAxis?: string;
  };

  export type LineChartNodeType = Node<LineChartNodeData, 'lineChart'> & {
    kind: 'chart';
  };
</script>

<script lang="ts">
  import { Handle, Position, type NodeProps } from '@xyflow/svelte';
  import { ChartLine } from 'lucide-svelte';
  import QueryNode from './QueryNode.svelte';

  let { id, selected }: NodeProps<LineChartNodeType> = $props();
</script>

<QueryNode {id} {selected} borderColor="purple">
  <div class="flex items-center gap-1 font-semibold text-sm text-gray-700 mb-1">
    <ChartLine class="w-4 h-4" />
    <span>Line Chart</span>
  </div>

  <Handle type="target" position={Position.Left} class="!bg-purple-500" />
</QueryNode>
