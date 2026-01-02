<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open as openFile } from '@tauri-apps/plugin-dialog';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import * as Dialog from '$lib/components/ui/dialog';
  import {
    CONNECTOR_TYPES,
    type ConnectorType,
    type ConnectorConfig,
    type Schema
  } from '$lib/types/connector';

  interface Props {
    open: boolean;
    existingTableNames: string[];
    onImportComplete: () => void;
  }

  let {
    open = $bindable(),
    existingTableNames,
    onImportComplete
  }: Props = $props();

  let currentStep = $state(1);
  let selectedConnectorType = $state<ConnectorType>('LocalFileCSV');
  let connectorConfig = $state<ConnectorConfig>([]);
  let configValues = $state<Record<string, any>>({});
  let discoveredSchema = $state<Schema | null>(null);
  let tableName = $state('');
  let isLoading = $state(false);
  let error = $state<string | null>(null);

  async function goToStep2() {
    currentStep = 2;
    error = null;
    discoveredSchema = null;
    tableName = '';
    configValues = {};

    try {
      isLoading = true;
      const configJson = await invoke<string>('config', {
        ty: selectedConnectorType
      });
      connectorConfig = JSON.parse(configJson);

      connectorConfig.forEach((item) => {
        const name = item.name as string;
        configValues[name] = item.default !== undefined ? item.default : '';
      });
    } catch (e: any) {
      error = `Failed to load connector config: ${e.message || e}`;
    } finally {
      isLoading = false;
    }
  }

  async function openFileDialog(fieldName: string) {
    const filters =
      selectedConnectorType === 'LocalFileCSV'
        ? [{ name: 'CSV/TSV Files', extensions: ['csv', 'tsv', 'tab'] }]
        : [{ name: 'JSON Files', extensions: ['json', 'jsonl'] }];

    const file = await openFile({
      multiple: false,
      directory: false,
      filters
    });
    if (file) {
      configValues[fieldName] = file;
    }
  }

  async function discovery() {
    try {
      isLoading = true;
      error = null;
      const schemaJson = await invoke<string>('discovery', {
        ty: selectedConnectorType,
        config: JSON.stringify(configValues)
      });
      discoveredSchema = JSON.parse(schemaJson);
    } catch (e: any) {
      error = `Failed to discover schema: ${e.message || e}`;
    } finally {
      isLoading = false;
    }
  }

  async function sync() {
    if (!tableName || tableName.trim() === '') {
      error = 'Table name is required';
      return;
    }

    if (existingTableNames.includes(tableName)) {
      error = `Table "${tableName}" already exists. Please choose a different name.`;
      return;
    }

    if (!discoveredSchema) {
      error = 'Please discover schema first';
      return;
    }

    try {
      isLoading = true;
      error = null;
      await invoke('sync', {
        ty: selectedConnectorType,
        name: tableName,
        config: JSON.stringify(configValues),
        schema: JSON.stringify(discoveredSchema)
      });

      onImportComplete();
      resetDialog();
    } catch (e: any) {
      error = `Failed to import data: ${e.message || e}`;
    } finally {
      isLoading = false;
    }
  }

  function resetDialog() {
    currentStep = 1;
    selectedConnectorType = 'LocalFileCSV';
    connectorConfig = [];
    configValues = {};
    discoveredSchema = null;
    tableName = '';
    error = null;
    isLoading = false;
  }

  function goBackToStep1() {
    currentStep = 1;
    error = null;
    discoveredSchema = null;
  }

  $effect(() => {
    if (!open) {
      resetDialog();
    }
  });
</script>

<Dialog.Root bind:open>
  <Dialog.Content class="max-w-2xl max-h-[80vh] overflow-y-auto">
    <Dialog.Header>
      <Dialog.Title>Import Data</Dialog.Title>
      <Dialog.Description>
        {currentStep === 1
          ? 'Select a data source'
          : 'Configure and import data'}
      </Dialog.Description>
    </Dialog.Header>

    {#if error}
      <div class="rounded bg-red-50 p-3 text-sm text-red-600">
        {error}
      </div>
    {/if}

    {#if currentStep === 1}
      <div class="space-y-4 py-4">
        <div class="space-y-3">
          <Label>Data Source Type</Label>
          {#each Object.entries(CONNECTOR_TYPES) as [type, label]}
            <div class="flex items-center space-x-2">
              <input
                type="radio"
                id={type}
                name="connector"
                value={type}
                checked={selectedConnectorType === type}
                onchange={() => (selectedConnectorType = type as ConnectorType)}
                class="h-4 w-4"
              />
              <Label for={type} class="cursor-pointer font-normal">
                {label}
              </Label>
            </div>
          {/each}
        </div>
      </div>
    {:else if currentStep === 2}
      <div class="space-y-4 py-4">
        <div class="space-y-4">
          <div class="space-y-2">
            <Label for="table-name">Table Name:</Label>
            <Input
              id="table-name"
              bind:value={tableName}
              type="text"
              placeholder="e.g., my_data"
            />
          </div>

          {#each connectorConfig as configItem}
            {#if configItem.type === 'file'}
              <div class="space-y-2">
                <Label>{configItem.name}:</Label>
                <div class="flex gap-2">
                  <Button
                    type="button"
                    onclick={() => openFileDialog(configItem.name as string)}
                    disabled={isLoading}
                  >
                    Select File
                  </Button>
                  {#if configValues[configItem.name as string]}
                    <span class="flex items-center text-sm text-gray-600">
                      {configValues[configItem.name as string]}
                    </span>
                  {/if}
                </div>
              </div>
            {:else if configItem.type === 'string'}
              <div class="space-y-2">
                <Label for={`config-${configItem.name}`}
                  >{configItem.name}:</Label
                >
                <Input
                  id={`config-${configItem.name}`}
                  type="text"
                  bind:value={configValues[configItem.name as string]}
                  disabled={isLoading}
                />
              </div>
            {:else if configItem.type === 'boolean'}
              <div class="flex items-center space-x-2">
                <Checkbox
                  id={`config-${configItem.name}`}
                  bind:checked={configValues[configItem.name as string]}
                  disabled={isLoading}
                />
                <Label for={`config-${configItem.name}`}
                  >{configItem.name}</Label
                >
              </div>
            {/if}
          {/each}

          {#if !discoveredSchema}
            <Button
              type="button"
              onclick={discovery}
              disabled={isLoading || !configValues.filePath}
              class="w-full"
            >
              {isLoading ? 'Discovering Schema...' : 'Discover Schema'}
            </Button>
          {/if}
        </div>

        {#if discoveredSchema}
          <div class="space-y-2">
            <h3 class="text-base font-semibold">Discovered Schema:</h3>
            <div class="rounded border">
              <table class="w-full text-sm">
                <thead class="border-b bg-gray-50">
                  <tr>
                    <th class="p-2 text-left">Column</th>
                    <th class="p-2 text-left">Type</th>
                  </tr>
                </thead>
                <tbody>
                  {#each discoveredSchema.columns as column}
                    <tr class="border-b last:border-b-0">
                      <td class="p-2">{column.name}</td>
                      <td class="p-2">{column.type}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <Dialog.Footer>
      <div class="flex w-full justify-between">
        {#if currentStep === 2}
          <Button
            type="button"
            variant="outline"
            onclick={goBackToStep1}
            disabled={isLoading}
          >
            Back
          </Button>
        {:else}
          <div></div>
        {/if}

        <div class="flex gap-2">
          {#if currentStep === 1}
            <Button type="button" onclick={goToStep2} disabled={isLoading}>
              Next
            </Button>
          {:else if currentStep === 2 && discoveredSchema}
            <Button type="button" onclick={sync} disabled={isLoading}>
              {isLoading ? 'Importing...' : 'Import'}
            </Button>
          {/if}
        </div>
      </div>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
