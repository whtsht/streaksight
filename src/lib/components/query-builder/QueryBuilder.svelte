<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    SvelteFlow,
    Background,
    Controls,
    MarkerType,
    type Edge,
    type NodeTypes,
    type BuiltInNode
  } from '@xyflow/svelte';
  import '@xyflow/svelte/dist/style.css';

  import TableNode, { type TableNodeType } from './nodes/TableNode.svelte';
  import FilterNode, { type FilterNodeType } from './nodes/FilterNode.svelte';
  import SelectNode, { type SelectNodeType } from './nodes/SelectNode.svelte';
  import SortNode, { type SortNodeType } from './nodes/SortNode.svelte';
  import LimitNode, { type LimitNodeType } from './nodes/LimitNode.svelte';
  import AggregationNode, {
    type AggregationNodeType
  } from './nodes/AggregationNode.svelte';
  import BarChartNode, {
    type BarChartNodeType
  } from './nodes/BarChartNode.svelte';
  import LineChartNode, {
    type LineChartNodeType
  } from './nodes/LineChartNode.svelte';
  import FilterNodeSettings from './settings/FilterNodeSettings.svelte';
  import SelectNodeSettings from './settings/SelectNodeSettings.svelte';
  import SortNodeSettings from './settings/SortNodeSettings.svelte';
  import LimitNodeSettings from './settings/LimitNodeSettings.svelte';
  import AggregationNodeSettings from './settings/AggregationNodeSettings.svelte';
  import BarChartNodeSettings from './settings/BarChartNodeSettings.svelte';
  import LineChartNodeSettings from './settings/LineChartNodeSettings.svelte';
  import BarChart from './views/BarChart.svelte';
  import LineChart from './views/LineChart.svelte';
  import PieChartNode, {
    type PieChartNodeType
  } from './nodes/PieChartNode.svelte';
  import PieChartNodeSettings from './settings/PieChartNodeSettings.svelte';
  import PieChart from './views/PieChart.svelte';
  import TableView from './views/TableView.svelte';
  import FitViewHelper from './FitViewHelper.svelte';
  import ConnectorDialog from './ConnectorDialog.svelte';
  import TableDetailsDialog from './TableDetailsDialog.svelte';
  import NodeListItem from './NodeListItem.svelte';
  import {
    Funnel,
    Columns3,
    ArrowUpDown,
    ArrowUpToLine,
    Sigma,
    ChartColumn,
    ChartLine,
    ChartPie,
    EllipsisVertical
  } from 'lucide-svelte';

  type NodeType =
    | BuiltInNode
    | TableNodeType
    | FilterNodeType
    | SelectNodeType
    | SortNodeType
    | LimitNodeType
    | AggregationNodeType
    | BarChartNodeType
    | LineChartNodeType
    | PieChartNodeType;

  const nodeTypes: NodeTypes = {
    table: TableNode,
    filter: FilterNode,
    select: SelectNode,
    sort: SortNode,
    limit: LimitNode,
    aggregation: AggregationNode,
    barChart: BarChartNode,
    lineChart: LineChartNode,
    pieChart: PieChartNode
  };

  const defaultEdgeOptions = {
    markerEnd: { type: MarkerType.ArrowClosed }
  };

  type TableInfo = {
    name: string;
    row_count: number;
  };

  type PreviewResult = {
    columns: Array<{ name: string; type: string }>;
    rows: Array<Record<string, any>>;
    row_count: number;
    error?: string;
  };

  let nodes = $state.raw<NodeType[]>([]);
  let edges = $state.raw<Edge[]>([]);
  let tables = $state<TableInfo[]>([]);
  let selectedNodeId = $state<string | null>(null);
  let previewData = $state<PreviewResult | null>(null);
  let isLoadingPreview = $state(false);
  let isImportDialogOpen = $state(false);
  let isTableDetailsDialogOpen = $state(false);
  let selectedTableForDetails = $state<string>('');
  let nodeIdCounter = 0;

  let currentPage = $state(1);
  let pageSize = $state(100);
  let totalRows = $state(0);
  let totalPages = $derived(Math.ceil(totalRows / pageSize));

  let selectedNode = $derived(
    nodes.find((n) => n.id === selectedNodeId) || null
  );

  let fitViewFn: (() => Promise<boolean>) | null = null;

  function handleFitViewReady(fitView: () => Promise<boolean>) {
    fitViewFn = fitView;
  }

  function isValidConnection(connection: any) {
    const sourceNode = nodes.find((n) => n.id === connection.source);
    const targetNode = nodes.find((n) => n.id === connection.target);

    if (!sourceNode || !targetNode) return false;

    const sourceKind = (sourceNode as any).kind;
    const targetKind = (targetNode as any).kind;

    if (
      sourceKind === 'source' &&
      (targetKind === 'translator' || targetKind === 'chart')
    ) {
      return true;
    }

    if (
      sourceKind === 'translator' &&
      (targetKind === 'translator' || targetKind === 'chart')
    ) {
      return true;
    }

    return false;
  }

  async function loadTables() {
    try {
      const result = await invoke<string>('tables');
      const data = JSON.parse(result);
      tables = data.tables;
    } catch (error) {
      console.error('Failed to load tables:', error);
    }
  }

  function handleImportComplete() {
    loadTables();
    isImportDialogOpen = false;
  }

  function openTableDetails(tableName: string) {
    selectedTableForDetails = tableName;
    isTableDetailsDialogOpen = true;
  }

  function handleTableDetailsAddToEditor(tableName: string) {
    addTableNode(tableName);
  }

  function handleTableDeleteComplete() {
    loadTables();
    isTableDetailsDialogOpen = false;
  }

  function addTableNode(tableName: string) {
    const newNode = {
      id: `table-${++nodeIdCounter}`,
      type: 'table',
      kind: 'source',
      position: { x: 100, y: 100 + nodeIdCounter * 50 },
      data: { table_name: tableName }
    } as TableNodeType;
    nodes = [...nodes, newNode];
    requestAnimationFrame(() => fitViewFn?.());
  }

  const NODE_DEFAULTS: Record<
    string,
    {
      kind: 'source' | 'translator' | 'chart';
      baseY: number;
      data: any;
    }
  > = {
    filter: { kind: 'translator', baseY: 100, data: { conditions: [] } },
    select: { kind: 'translator', baseY: 100, data: { columns: [] } },
    sort: { kind: 'translator', baseY: 100, data: { order: [] } },
    limit: { kind: 'translator', baseY: 100, data: { limit: 10 } },
    aggregation: {
      kind: 'translator',
      baseY: 100,
      data: { dimensions: [], metrics: [] }
    },
    barChart: { kind: 'chart', baseY: 200, data: { xAxis: '', yAxis: '' } },
    lineChart: { kind: 'chart', baseY: 300, data: { xAxis: '', yAxis: '' } },
    pieChart: {
      kind: 'chart',
      baseY: 400,
      data: { categoryColumn: '', valueColumn: '' }
    }
  };

  const translatorNodes = [
    { icon: Funnel, label: 'Filter', type: 'filter' },
    { icon: Columns3, label: 'Select', type: 'select' },
    { icon: ArrowUpDown, label: 'Sort', type: 'sort' },
    { icon: ArrowUpToLine, label: 'Limit', type: 'limit' },
    { icon: Sigma, label: 'Aggregation', type: 'aggregation' }
  ];

  const chartNodes = [
    { icon: ChartColumn, label: 'Bar Chart', type: 'barChart' },
    { icon: ChartLine, label: 'Line Chart', type: 'lineChart' },
    { icon: ChartPie, label: 'Pie Chart', type: 'pieChart' }
  ];

  const settingsComponents: Record<
    string,
    {
      component: any;
      updateFn: (field: string, value: any) => void;
    }
  > = {
    filter: {
      component: FilterNodeSettings,
      updateFn: (field, value) => updateNodeData('filter', field, value)
    },
    select: {
      component: SelectNodeSettings,
      updateFn: (field, value) => updateNodeData('select', field, value)
    },
    sort: {
      component: SortNodeSettings,
      updateFn: (field, value) => updateNodeData('sort', field, value)
    },
    limit: {
      component: LimitNodeSettings,
      updateFn: (field, value) => updateNodeData('limit', field, value)
    },
    aggregation: {
      component: AggregationNodeSettings,
      updateFn: (field, value) => updateNodeData('aggregation', field, value)
    },
    barChart: {
      component: BarChartNodeSettings,
      updateFn: (field, value) => updateNodeData('barChart', field, value)
    },
    lineChart: {
      component: LineChartNodeSettings,
      updateFn: (field, value) => updateNodeData('lineChart', field, value)
    },
    pieChart: {
      component: PieChartNodeSettings,
      updateFn: (field, value) => updateNodeData('pieChart', field, value)
    }
  };

  let settingsConfig = $derived(
    selectedNode ? settingsComponents[selectedNode.type] : null
  );

  type ChartConfig = {
    component: any;
    getProps: (data: any, rows: any[]) => any;
    isConfigured: (data: any) => boolean;
    configMessage: string;
  };

  const chartConfigs: Record<string, ChartConfig> = {
    barChart: {
      component: BarChart,
      getProps: (data, rows) => ({
        data: rows,
        xAxis: data.xAxis,
        yAxis: data.yAxis
      }),
      isConfigured: (data) => !!(data.xAxis && data.yAxis),
      configMessage: 'Configure X and Y axes in the settings panel'
    },
    lineChart: {
      component: LineChart,
      getProps: (data, rows) => ({
        data: rows,
        xAxis: data.xAxis,
        yAxis: data.yAxis
      }),
      isConfigured: (data) => !!(data.xAxis && data.yAxis),
      configMessage: 'Configure X and Y axes in the settings panel'
    },
    pieChart: {
      component: PieChart,
      getProps: (data, rows) => ({
        data: rows,
        categoryColumn: data.categoryColumn,
        valueColumn: data.valueColumn
      }),
      isConfigured: (data) => !!(data.categoryColumn && data.valueColumn),
      configMessage:
        'Configure category and value columns in the settings panel'
    }
  };

  let chartConfig = $derived(
    selectedNode && chartConfigs[selectedNode.type]
      ? chartConfigs[selectedNode.type]
      : null
  );

  function addNode(type: string) {
    const config = NODE_DEFAULTS[type];
    if (!config) return;

    const newNode = {
      id: `${type}-${++nodeIdCounter}`,
      type,
      kind: config.kind,
      position: { x: 400, y: config.baseY + nodeIdCounter * 50 },
      data: structuredClone(config.data)
    } as NodeType;
    nodes = [...nodes, newNode];
    requestAnimationFrame(() => fitViewFn?.());
  }

  async function updatePreview() {
    if (!selectedNodeId) {
      previewData = null;
      return;
    }

    isLoadingPreview = true;

    try {
      let queryNodeId = selectedNodeId;
      const currentNode = nodes.find((n) => n.id === selectedNodeId);
      if ((currentNode as any)?.kind === 'chart') {
        const sourceEdge = edges.find((e) => e.target === selectedNodeId);
        if (sourceEdge) {
          queryNodeId = sourceEdge.source;
        }
      }

      const nodeGraph = {
        selected_node_id: queryNodeId,
        nodes: nodes.map((n) => ({
          id: n.id,
          type: n.type,
          data: n.data
        })),
        edges: edges.map((e) => ({
          source: e.source,
          target: e.target
        }))
      };

      const [countResult, queryResult] = await Promise.all([
        invoke<number>('get_query_row_count', {
          nodeGraph: JSON.stringify(nodeGraph)
        }),
        invoke<string>('run_query', {
          nodeGraph: JSON.stringify(nodeGraph),
          page: currentPage,
          pageSize: pageSize
        })
      ]);

      totalRows = countResult;
      const parsedData = JSON.parse(queryResult);
      previewData = parsedData;
    } catch (error: any) {
      console.error('Failed to run query:', error);
      previewData = {
        columns: [],
        rows: [],
        row_count: 0,
        error: error.toString()
      };
    } finally {
      isLoadingPreview = false;
    }
  }

  function handlePageChange(page: number) {
    currentPage = page;
    updatePreview();
  }

  function onNodeClick({
    node
  }: {
    event: MouseEvent | TouchEvent;
    node: NodeType;
  }) {
    selectedNodeId = node.id;
    currentPage = 1;
    updatePreview();
  }

  function onConnect(connection: any) {
    edges = [...edges, connection];
    currentPage = 1;
    updatePreview();
  }

  function onDelete() {
    if (selectedNodeId && !nodes.find((n) => n.id === selectedNodeId)) {
      selectedNodeId = null;
      previewData = null;
      currentPage = 1;
    }
  }

  function updateNodeData(nodeType: string, field: string, value: any) {
    if (!selectedNodeId) return;

    nodes = nodes.map((n) => {
      if (n.id === selectedNodeId && n.type === nodeType) {
        return {
          ...n,
          data: { ...n.data, [field]: value }
        } as NodeType;
      }
      return n;
    });

    currentPage = 1;
    updatePreview();
  }

  onMount(() => {
    loadTables();
  });
</script>

<div class="h-screen flex">
  <div class="w-64 bg-gray-50 border-r border-gray-200 flex flex-col">
    <div class="p-4 overflow-y-auto">
      <h2 class="text-base font-semibold text-gray-700 mb-3">Tables</h2>
      <div class="space-y-2 mb-6">
        {#each tables as table}
          <div
            class="flex items-center justify-between px-3 py-2 text-sm bg-white border border-gray-300 rounded"
          >
            <span class="text-gray-700">{table.name}</span>
            <div class="flex items-center gap-1">
              <button
                onclick={() => addTableNode(table.name)}
                class="px-2 py-1 text-xs bg-primary text-primary-foreground rounded hover:bg-primary/90"
                title="Add table node"
              >
                +
              </button>
              <button
                onclick={() => openTableDetails(table.name)}
                class="p-1 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded"
                title="Table details"
              >
                <EllipsisVertical class="w-4 h-4" />
              </button>
            </div>
          </div>
        {/each}
        <button
          onclick={() => (isImportDialogOpen = true)}
          class="w-full mt-2 px-3 py-2 text-sm bg-primary text-primary-foreground rounded hover:bg-primary/90"
          title="Import data"
        >
          + Add Data Source
        </button>
      </div>

      <h2 class="text-base font-semibold text-gray-700 mb-3">Translator</h2>
      <div class="space-y-2 mb-6">
        {#each translatorNodes as node}
          <NodeListItem
            icon={node.icon}
            label={node.label}
            onAdd={() => addNode(node.type)}
          />
        {/each}
      </div>

      <h2 class="text-base font-semibold text-gray-700 mb-3">Chart</h2>
      <div class="space-y-2 mb-6">
        {#each chartNodes as node}
          <NodeListItem
            icon={node.icon}
            label={node.label}
            onAdd={() => addNode(node.type)}
          />
        {/each}
      </div>
    </div>
  </div>

  <div class="flex-1 flex flex-col">
    <div class="flex-[3] min-h-0 flex border-b border-gray-200">
      <div class="flex-1 bg-white p-4 overflow-auto border-r border-gray-200">
        {#if isLoadingPreview}
          <div class="text-gray-500">Loading...</div>
        {:else if previewData?.error}
          <div class="text-red-600 text-sm">{previewData.error}</div>
        {:else if previewData}
          {#if chartConfig && selectedNode}
            {#if chartConfig.isConfigured(selectedNode.data)}
              {@const ChartComponent = chartConfig.component}
              <div class="h-[calc(100%-2.5rem)]">
                <ChartComponent
                  {...chartConfig.getProps(selectedNode.data, previewData.rows)}
                />
              </div>
            {:else}
              <div class="text-gray-500">{chartConfig.configMessage}</div>
            {/if}
          {:else}
            <TableView
              data={previewData.rows}
              columns={previewData.columns}
              rowCount={previewData.row_count}
              {currentPage}
              {totalPages}
              {totalRows}
              onPageChange={handlePageChange}
            />
          {/if}
        {:else}
          <div class="text-gray-500">Select a node to preview data</div>
        {/if}
      </div>

      <div class="w-80 bg-white p-4 overflow-auto">
        {#if selectedNodeId && selectedNode && settingsConfig}
          {@const SettingsComponent = settingsConfig.component}

          <SettingsComponent
            nodeData={selectedNode.data}
            previewColumns={previewData?.columns || []}
            onDataChange={settingsConfig.updateFn}
          />
        {:else}
          <div class="text-gray-500 text-sm">Select a node to configure</div>
        {/if}
      </div>
    </div>

    <div class="flex-[2] min-h-0">
      <SvelteFlow
        bind:nodes
        bind:edges
        {nodeTypes}
        {defaultEdgeOptions}
        {isValidConnection}
        onnodeclick={onNodeClick}
        onconnect={onConnect}
        ondelete={onDelete}
        fitView
      >
        <FitViewHelper onReady={handleFitViewReady} />
        <Background />
        <Controls />
      </SvelteFlow>
    </div>
  </div>
</div>

<ConnectorDialog
  bind:open={isImportDialogOpen}
  existingTableNames={tables.map((t) => t.name)}
  onImportComplete={handleImportComplete}
/>

<TableDetailsDialog
  bind:open={isTableDetailsDialogOpen}
  tableName={selectedTableForDetails}
  onAddToEditor={handleTableDetailsAddToEditor}
  onDeleteComplete={handleTableDeleteComplete}
/>
