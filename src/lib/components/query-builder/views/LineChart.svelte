<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as echarts from 'echarts';

  type Props = {
    data: Array<Record<string, any>>;
    xAxis: string;
    yAxis: string;
  };

  let { data, xAxis, yAxis }: Props = $props();

  let chartContainer: HTMLDivElement;
  let chartInstance: echarts.ECharts | null = null;
  let resizeHandler: (() => void) | null = null;

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
    if (chartInstance && data && xAxis && yAxis) {
      const xAxisData = data.map((row) => row[xAxis]);
      const yAxisData = data.map((row) => row[yAxis]);

      const option: echarts.EChartsOption = {
        tooltip: {
          trigger: 'axis',
          axisPointer: {
            type: 'cross'
          }
        },
        grid: {
          left: '10%',
          right: '10%',
          bottom: '15%',
          top: '10%',
          containLabel: true
        },
        xAxis: {
          type: 'category',
          data: xAxisData,
          name: xAxis,
          nameLocation: 'middle',
          nameGap: 30,
          boundaryGap: false
        },
        yAxis: {
          type: 'value',
          name: yAxis,
          nameLocation: 'middle',
          nameGap: 50
        },
        series: [
          {
            type: 'line',
            data: yAxisData,
            smooth: true,
            lineStyle: {
              color: '#a855f7',
              width: 2
            },
            itemStyle: {
              color: '#a855f7'
            },
            areaStyle: {
              color: {
                type: 'linear',
                x: 0,
                y: 0,
                x2: 0,
                y2: 1,
                colorStops: [
                  { offset: 0, color: 'rgba(168, 85, 247, 0.3)' },
                  { offset: 1, color: 'rgba(168, 85, 247, 0.05)' }
                ]
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
