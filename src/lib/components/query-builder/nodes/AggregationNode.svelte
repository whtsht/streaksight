<script module lang="ts">
  import type { Node } from '@xyflow/svelte';

  export type Metric = {
    function: 'COUNT' | 'COUNT(*)' | 'SUM' | 'AVG' | 'MAX' | 'MIN';
    column: string;
  };

  type AggregationNodeData = {
    dimensions?: string[];
    metrics?: Metric[];
  };

  export type AggregationNodeType = Node<AggregationNodeData, 'aggregation'> & {
    kind: 'translator';
  };
</script>

<script lang="ts">
  import { Handle, Position, type NodeProps } from '@xyflow/svelte';
  import { Sigma } from 'lucide-svelte';
  import QueryNode from './QueryNode.svelte';

  let { id, selected }: NodeProps<AggregationNodeType> = $props();
</script>

<QueryNode {id} {selected} borderColor="green">
  <div class="flex items-center gap-1 font-semibold text-sm text-gray-700 mb-1">
    <Sigma class="w-4 h-4" />
    <span>Aggregation</span>
  </div>

  <Handle type="target" position={Position.Left} class="!bg-green-500" />
  <Handle type="source" position={Position.Right} class="!bg-green-500" />
</QueryNode>
