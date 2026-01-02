<script lang="ts">
  import type { SvelteComponent } from 'svelte';
  import type { IconProps } from 'lucide-svelte';

  type LucideIcon = typeof SvelteComponent<IconProps>;

  interface Props {
    icon: LucideIcon;
    label: string;
    onAdd: () => void;
  }

  let { icon: IconComponent, label, onAdd }: Props = $props();
</script>

<div
  class="flex items-center justify-between px-3 py-2 text-sm bg-white border border-gray-300 rounded"
>
  <span class="flex items-center gap-1 text-gray-700">
    <IconComponent class="w-4 h-4" />
    <span>{label}</span>
  </span>
  <button
    onclick={onAdd}
    class="px-2 py-1 text-xs bg-primary text-primary-foreground rounded hover:bg-primary/90"
    title="Add {label.toLowerCase()} node"
  >
    +
  </button>
</div>
