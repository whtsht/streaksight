<script module lang="ts">
  export type BorderColor = 'blue' | 'green' | 'purple';
</script>

<script lang="ts">
  import { useSvelteFlow } from '@xyflow/svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    id: string;
    selected: boolean;
    borderColor: BorderColor;
    children: Snippet;
  }

  let { id, selected, borderColor, children }: Props = $props();

  const { deleteElements } = useSvelteFlow();

  function handleDelete(event: MouseEvent) {
    event.stopPropagation();
    deleteElements({ nodes: [{ id }] });
  }

  const borderColorMap = {
    blue: 'border-blue-500',
    green: 'border-green-500',
    purple: 'border-purple-500'
  };

  const hoverColorMap = {
    blue: '#3b82f6',
    green: '#22c55e',
    purple: '#a855f7'
  };

  const selectColorShadowMap = {
    blue: 'rgba(59, 130, 246, 0.5)',
    green: 'rgba(34, 197, 94, 0.5)',
    purple: 'rgba(168, 85, 247, 0.5)'
  };

  const borderClass = $derived(borderColorMap[borderColor]);
  const hoverColor = $derived(hoverColorMap[borderColor]);
  const selectColorShadow = $derived(selectColorShadowMap[borderColor]);
</script>

<div
  class="query-node rounded-lg border-2 bg-white p-3 min-w-[150px] {borderClass}"
  class:selected
  style="--hover-color: {hoverColor}; --select-color: {hoverColor}; --select-color-shadow: {selectColorShadow}"
>
  <div class="content-wrapper">
    {@render children()}
  </div>
  <button
    onclick={handleDelete}
    class="delete-btn nodrag text-gray-400 hover:text-red-500 text-xs font-bold leading-none"
    title="Delete node"
  >
    ×
  </button>
</div>

<style>
  .query-node {
    cursor: pointer;
    transition: all 0.2s ease-in-out;
    position: relative;
  }

  .query-node:hover {
    border-color: var(--hover-color);
  }

  .query-node.selected {
    box-shadow: 0 0 0 3px var(--select-color-shadow);
    border-color: var(--select-color);
  }

  .content-wrapper {
    padding-right: 20px;
  }

  .delete-btn {
    position: absolute;
    top: 8px;
    right: 8px;
  }
</style>
