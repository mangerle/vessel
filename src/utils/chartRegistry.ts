/**
 * ECharts 组件注册中心
 *
 * 全局只注册一次。原先 Containers.vue / Compose.vue / ContainerDetail.vue
 * 各自调用 use([...])，不仅模板重复，也导致同一组件被重复 use 多次（无副作用但无意义）。
 * 这里收口为一次性副作用模块：在任意 view 顶部 import 即可。
 */
import { use } from 'echarts/core'
import { LineChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

let registered = false

/** 幂等注册：多次调用仅第一次真正执行 use()。 */
export function ensureChartsRegistered(): void {
  if (registered) return
  registered = true
  use([
    CanvasRenderer,
    LineChart,
    GridComponent,
    TooltipComponent,
    LegendComponent,
    TitleComponent
  ])
}

// 模块加载即注册（Vite import 顺序保证 view 渲染前完成）
ensureChartsRegistered()
