<script lang="ts">
  import { useSvelteFlow } from '@xyflow/svelte';
  import { onMount } from 'svelte';

  type Props = {
    onReady: (fitView: () => Promise<boolean>) => void;
  };

  let { onReady }: Props = $props();

  const { fitView } = useSvelteFlow();

  onMount(() => {
    onReady(fitView);
  });
</script>
