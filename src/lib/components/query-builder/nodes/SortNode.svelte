<script module lang="ts">
  import type { Node } from '@xyflow/svelte';

  type OrderBy = {
    column: string;
    direction: string;
  };

  type SortNodeData = {
    order?: OrderBy[];
  };

  export type SortNodeType = Node<SortNodeData, 'sort'> & {
    kind: 'translator';
  };
</script>

<script lang="ts">
  import { Handle, Position, type NodeProps } from '@xyflow/svelte';
  import { ArrowUpDown } from 'lucide-svelte';
  import QueryNode from './QueryNode.svelte';

  let { id, selected }: NodeProps<SortNodeType> = $props();
</script>

<QueryNode {id} {selected} borderColor="green">
  <div class="flex items-center gap-1 font-semibold text-sm text-gray-700 mb-1">
    <ArrowUpDown class="w-4 h-4" />
    <span>Sort</span>
  </div>

  <Handle type="target" position={Position.Left} class="!bg-green-500" />
  <Handle type="source" position={Position.Right} class="!bg-green-500" />
</QueryNode>
