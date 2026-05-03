# macOS 风格 UI 重构 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans
> 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 将现有的 Docker Manager UI 重构为 macOS 风格的“三栏浮动卡片”布局，实现简洁、精致且优雅的视觉体验。

**架构：** 采用三栏式设计：左侧导航、中间项目列表卡片、右侧详情工作区卡片。通过自定义 CSS 变量和 Naive UI 主题定制实现 Apple
审美。

**技术栈：** Vue 3, Naive UI, Tailwind CSS (如果已配置) 或 原生 CSS, ECharts, Xterm.js, Tauri.

---

## 文件结构预定义

为了实现模块化和清晰的职责划分，我们将创建/修改以下文件：

### 1. 全局样式与配置

- `src/assets/styles/theme.css`: 定义全局 CSS 变量（颜色、圆角、阴影）。
- `src/theme.ts`: Naive UI 主题覆盖配置。

### 2. 通用组件 (Base Components)

- `src/components/common/SegmentedControl.vue`: macOS 风格的滑动分段选择器。
- `src/components/common/FloatingCard.vue`: 带有预设阴影和圆角的卡片容器。

### 3. Compose 模块重构 (Feature Components)

- `src/views/Compose.vue`: 组合主容器。
- `src/components/compose/ComposeProjectList.vue`: 中间栏项目列表。
- `src/components/compose/ContainerDetail.vue`: 右侧详情区。
- `src/components/compose/StatsDashboard.vue`: 仪表盘子组件。
- `src/components/compose/TerminalWindow.vue`: 终端子组件。

---

## 任务列表

### 任务 1：全局样式基础建设

**文件：**

- 创建：`src/assets/styles/theme.css`
- 修改：`src/main.ts`

- [ ] **步骤 1：定义全局 CSS 变量**
  在 `src/assets/styles/theme.css` 中定义 Apple 风格的变量：

```css
:root {
    --macos-bg-light: #F5F5F7;
    --macos-card-bg-light: #FFFFFF;
    --macos-accent-blue: #007AFF;
    --macos-success-green: #28CD41;
    --macos-border-color: rgba(0, 0, 0, 0.1);
    --macos-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
    --macos-radius: 12px;
}

body {
    background-color: var(--macos-bg-light);
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
}
```

- [ ] **步骤 2：在 main.ts 中引入样式**

```typescript
import './assets/styles/theme.css'
```

- [ ] **步骤 3：Commit**

```bash
git add src/assets/styles/theme.css src/main.ts
git commit -m "style: init macos theme variables"
```

### 任务 2：创建分段选择器组件 (Segmented Control)

**文件：**

- 创建：`src/components/common/SegmentedControl.vue`

- [ ] **步骤 1：实现 SegmentedControl.vue**
  这是一个具有滑动动画效果的选项卡切换器。

```vue

<template>
  <div class="segmented-control">
    <div
        class="selection-pill"
        :style="{ width: pillWidth + 'px', transform: `translateX(${pillOffset}px)` }"
    ></div>
    <div
        v-for="(option, index) in options"
        :key="option.value"
        class="option-item"
        :class="{ active: modelValue === option.value }"
        @click="select(option.value, index)"
        ref="optionRefs"
    >
      {{ option.label }}
    </div>
  </div>
</template>

<script setup lang="ts">
  import {ref, onMounted, watch} from 'vue'

  const props = defineProps<{
    options: { label: string, value: string }[],
    modelValue: string
  }>()

  const emit = defineEmits(['update:modelValue'])
  const optionRefs = ref<HTMLElement[]>([])
  const pillWidth = ref(0)
  const pillOffset = ref(0)

  const updatePill = (index: number) => {
    const el = optionRefs.value[index]
    if (el) {
      pillWidth.value = el.offsetWidth
      pillOffset.value = el.offsetLeft
    }
  }

  const select = (value: string, index: number) => {
    emit('update:modelValue', value)
    updatePill(index)
  }

  onMounted(() => {
    const index = props.options.findIndex(o => o.value === props.modelValue)
    updatePill(index >= 0 ? index : 0)
  })
</script>

<style scoped>
  .segmented-control {
    display: flex;
    background: rgba(0, 0, 0, 0.05);
    border-radius: 8px;
    padding: 2px;
    position: relative;
    width: fit-content;
  }

  .selection-pill {
    position: absolute;
    height: calc(100% - 4px);
    background: white;
    border-radius: 6px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .option-item {
    padding: 4px 12px;
    font-size: 13px;
    color: #333;
    cursor: pointer;
    position: relative;
    z-index: 1;
    transition: color 0.3s;
  }

  .option-item.active {
    font-weight: 500;
  }
</style>
```

- [ ] **步骤 2：Commit**

```bash
git add src/components/common/SegmentedControl.vue
git commit -m "feat: add SegmentedControl component"
```

### 任务 3：重构 Compose 项目列表组件

**文件：**

- 创建：`src/components/compose/ComposeProjectList.vue`

- [ ] **步骤 1：从原 Compose.vue 抽取列表逻辑**
  包含搜索框和容器列表项，列表项左侧带有状态指示条。

```vue

<template>
  <div class="project-list-card">
    <div class="header">
      <n-input placeholder="搜索容器..." size="small" round clearable/>
    </div>
    <div class="list-content">
      <div
          v-for="item in items"
          :key="item.id"
          class="list-item"
          :class="{ active: selectedId === item.id }"
          @click="$emit('select', item.id)"
      >
        <div class="status-bar" :style="{ backgroundColor: item.isRunning ? '#28CD41' : '#8E8E93' }"></div>
        <div class="info">
          <div class="name">{{ item.name }}</div>
          <div class="image">{{ item.image }}</div>
        </div>
      </div>
    </div>
  </div>
</template>
```

- [ ] **步骤 2：Commit**

```bash
git add src/components/compose/ComposeProjectList.vue
git commit -m "feat: add ComposeProjectList component"
```

### 任务 4：重构容器详情组件

**文件：**

- 创建：`src/components/compose/ContainerDetail.vue`

- [ ] **步骤 1：实现详情卡片布局**
  包含 Header（操作按钮）和 Body（分段切换内容）。

```vue

<template>
  <div class="detail-card">
    <div class="toolbar">
      <div class="title-section">
        <h2 class="name">{{ container?.name }}</h2>
        <span class="id">{{ container?.id?.substring(0, 12) }}</span>
      </div>
      <div class="actions">
        <n-button-group round size="small">
          <n-button @click="$emit('restart')">重启</n-button>
          <n-button @click="$emit('stop')">停止</n-button>
        </n-button-group>
      </div>
    </div>
    <div class="nav-section">
      <SegmentedControl v-model="activeTab" :options="tabOptions"/>
    </div>
    <div class="content-section">
      <!-- 根据 activeTab 显示不同内容 -->
    </div>
  </div>
</template>
```

- [ ] **步骤 2：Commit**

```bash
git add src/components/compose/ContainerDetail.vue
git commit -m "feat: add ContainerDetail component"
```

### 任务 5：组装与整体润色

**文件：**

- 修改：`src/views/Compose.vue`
- 修改：`src/layout/MainLayout.vue`

- [ ] **步骤 1：重写 Compose.vue 主容器**
  使用 `display: flex` 布局三栏，并应用卡片样式。

```vue

<template>
  <div class="compose-view">
    <ComposeProjectList class="list-col" @select="handleSelect"/>
    <ContainerDetail class="detail-col" :container="selectedContainer"/>
  </div>
</template>

<style scoped>
  .compose-view {
    display: flex;
    gap: 16px;
    height: 100%;
  }

  .list-col {
    width: 320px;
    background: white;
    border-radius: 12px;
    box-shadow: var(--macos-shadow);
  }

  .detail-col {
    flex: 1;
    background: white;
    border-radius: 12px;
    box-shadow: var(--macos-shadow);
  }
</style>
```

- [ ] **步骤 2：调整 MainLayout.vue 去除冗余背景和边框**

- [ ] **步骤 3：验证所有功能（日志、终端、监控）在重构后依然正常**

- [ ] **步骤 4：Final Commit**

```bash
git add src/views/Compose.vue src/layout/MainLayout.vue
git commit -m "feat: complete UI redesign for Compose view"
```
