<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$lib/components/ui/button';
  import * as Dialog from '$lib/components/ui/dialog';
  import { Trash2 } from 'lucide-svelte';

  interface Props {
    open: boolean;
    tableName: string;
    onAddToEditor: (tableName: string) => void;
    onDeleteComplete: () => void;
  }

  let {
    open = $bindable(),
    tableName,
    onAddToEditor,
    onDeleteComplete
  }: Props = $props();

  type ColumnInfo = {
    name: string;
    type: string;
  };

  type SchemaResponse = {
    table_name: string;
    columns: ColumnInfo[];
  };

  let schema = $state<SchemaResponse | null>(null);
  let showDeleteConfirm = $state(false);

  $effect(() => {
    if (open && tableName) {
      loadSchema();
    } else {
      schema = null;
      showDeleteConfirm = false;
    }
  });

  async function loadSchema() {
    try {
      const result = await invoke<string>('table_schema', { tableName });
      schema = JSON.parse(result);
    } catch (e) {
      console.error('Failed to load table schema:', e);
    }
  }

  function handleAddToEditor() {
    onAddToEditor(tableName);
    open = false;
  }

  async function handleDelete() {
    try {
      await invoke('drop_table', { tableName });
      onDeleteComplete();
      open = false;
    } catch (e) {
      console.error('Failed to delete table:', e);
      alert(`Failed to delete table: ${e}`);
    }
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-w-2xl">
    <Dialog.Header>
      <Dialog.Title>Table: {tableName}</Dialog.Title>
    </Dialog.Header>

    {#if schema}
      <div class="space-y-4 py-4">
        <div>
          <h3 class="text-base mb-2 font-semibold">Schema</h3>
          <div class="rounded border">
            <table class="w-full text-sm">
              <thead class="border-b bg-gray-50">
                <tr>
                  <th class="p-2 text-left">Column</th>
                  <th class="p-2 text-left">Type</th>
                </tr>
              </thead>
              <tbody>
                {#each schema.columns as column}
                  <tr class="border-b last:border-b-0">
                    <td class="p-2">{column.name}</td>
                    <td class="p-2">{column.type}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    {/if}

    <Dialog.Footer>
      {#if !showDeleteConfirm}
        <div class="flex w-full justify-between">
          <Button
            variant="destructive"
            onclick={() => (showDeleteConfirm = true)}
          >
            <Trash2 class="mr-2 h-4 w-4" />
            Delete Table
          </Button>
          <Button onclick={handleAddToEditor}>Add to Editor</Button>
        </div>
      {:else}
        <div class="flex w-full flex-col gap-3">
          <p class="text-sm text-red-600">
            Are you sure you want to delete "{tableName}"? This action cannot be
            undone.
          </p>
          <div class="flex justify-end gap-2">
            <Button
              variant="outline"
              onclick={() => (showDeleteConfirm = false)}
            >
              Cancel
            </Button>
            <Button variant="destructive" onclick={handleDelete}>
              Confirm Delete
            </Button>
          </div>
        </div>
      {/if}
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
