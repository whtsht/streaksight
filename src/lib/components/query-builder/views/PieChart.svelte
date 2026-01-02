<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as echarts from 'echarts';

  type Props = {
    data: Array<Record<string, any>>;
    categoryColumn: string;
    valueColumn: string;
  };

  let { data, categoryColumn, valueColumn }: Props = $props();

  let chartContainer: HTMLDivElement;
  let chartInstance: echarts.ECharts | null = null;
  let resizeHandler: (() => void) | null = null;

  const purplePalette = [
    '#a855f7',
    '#c084fc',
    '#e9d5ff',
    '#9333ea',
    '#7e22ce',
    '#d8b4fe',
    '#f3e8ff',
    '#6b21a8',
    '#581c87'
  ];

  onMount(() => {
    if (chartContainer) {
      chartInstance = echarts.init(chartContainer);
      resizeHandler = () => {
        chartInstance?.resize();
      };
      window.addEventListener('resize', resizeHandler);
    }
  });

  onDestroy(() => {
    if (resizeHandler) {
      window.removeEventListener('resize', resizeHandler);
    }
    chartInstance?.dispose();
  });

  $effect(() => {
    if (chartInstance && data && categoryColumn && valueColumn) {
      const pieData = data.map((row) => ({
        name: row[categoryColumn],
        value: row[valueColumn]
      }));

      const option: echarts.EChartsOption = {
        tooltip: {
          trigger: 'item',
          formatter: '{a} <br/>{b}: {c} ({d}%)'
        },
        legend: {
          orient: 'vertical',
          right: '10%',
          top: 'center'
        },
        series: [
          {
            name: categoryColumn,
            type: 'pie',
            radius: ['40%', '70%'],
            avoidLabelOverlap: false,
            label: {
              formatter: '{b}: {d}%'
            },
            emphasis: {
              itemStyle: {
                shadowBlur: 10,
                shadowOffsetX: 0,
                shadowColor: 'rgba(0, 0, 0, 0.5)'
              }
            },
            data: pieData,
            itemStyle: {
              color: function (params) {
                return purplePalette[params.dataIndex % purplePalette.length];
              }
            }
          }
        ]
      };

      chartInstance.setOption(option);
    }
  });
</script>

<div bind:this={chartContainer} class="w-full h-full"></div>
