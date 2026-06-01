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
import { ref, onMounted, nextTick, watch } from 'vue'

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

// 监听外部 modelValue 变化，同步更新 pill 位置
watch(() => props.modelValue, (newVal) => {
  const index = props.options.findIndex(o => o.value === newVal)
  if (index >= 0) updatePill(index)
})

onMounted(() => {
  nextTick(() => {
    const index = props.options.findIndex(o => o.value === props.modelValue)
    updatePill(index >= 0 ? index : 0)
  })
})
</script>

<style scoped>
.segmented-control {
  display: flex;
  background: var(--bg-hover);
  border-radius: 8px;
  padding: 2px;
  position: relative;
  width: fit-content;
}
.selection-pill {
  position: absolute;
  height: calc(100% - 4px);
  background: var(--bg-active);
  border-radius: 6px;
  box-shadow: 0 1px 3px var(--shadow-color);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.option-item {
  padding: 4px 12px;
  font-size: 13px;
  color: var(--text-muted);
  cursor: pointer;
  position: relative;
  z-index: 1;
  transition: color 0.3s;
  user-select: none;
}
.option-item.active {
  font-weight: 500;
  color: var(--text-title);
}
</style>
