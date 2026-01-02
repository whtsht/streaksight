<script module lang="ts">
  import type { Node } from '@xyflow/svelte';

  export type FilterCondition = {
    column: string;
    operator: '==' | '!=' | '>' | '<' | '>=' | '<=' | 'in';
    value: string | string[];
    negate?: boolean;
  };

  type FilterNodeData = {
    conditions?: FilterCondition[];
  };

  export type FilterNodeType = Node<FilterNodeData, 'filter'> & {
    kind: 'translator';
  };
</script>

<script lang="ts">
  import { Handle, Position, type NodeProps } from '@xyflow/svelte';
  import { Funnel } from 'lucide-svelte';
  import QueryNode from './QueryNode.svelte';

  let { id, selected }: NodeProps<FilterNodeType> = $props();
</script>

<QueryNode {id} {selected} borderColor="green">
  <div class="flex items-center gap-1 font-semibold text-sm text-gray-700 mb-1">
    <Funnel class="w-4 h-4" />
    <span>Filter</span>
  </div>

  <Handle type="target" position={Position.Left} class="!bg-green-500" />
  <Handle type="source" position={Position.Right} class="!bg-green-500" />
</QueryNode>
