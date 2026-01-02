<script module lang="ts">
  import type { Node } from '@xyflow/svelte';

  export type TableNodeType = Node<{ table_name: string }, 'table'> & {
    kind: 'source';
  };
</script>

<script lang="ts">
  import { Handle, Position, type NodeProps } from '@xyflow/svelte';
  import QueryNode from './QueryNode.svelte';

  let { id, data, selected }: NodeProps<TableNodeType> = $props();
</script>

<QueryNode {id} {selected} borderColor="blue">
  <div class="font-semibold text-sm text-gray-700">Table</div>
  <div class="text-xs text-gray-600">{data.table_name}</div>
  <Handle type="source" position={Position.Right} class="!bg-blue-500" />
</QueryNode>
