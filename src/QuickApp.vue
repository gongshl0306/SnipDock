<script setup lang="ts">
// 快速搜索窗口的根组件（#quick hash 时挂载）。
//
// 启动即加载全部片段（quick 窗口预创建但 hidden，组件在首次创建时就 mount，
// 所以 load 在 onMounted 调；之后窗口 show/hide 不重建组件，状态常驻）。
//
// 交互：输入过滤、↑↓ 切换、Enter 复制并关闭、Esc 关闭、鼠标点击复制。
// 失焦隐藏由 Rust 端处理；窗口再次 show 时 reset 状态（监听 visibility 事件）。

import { onMounted, ref, nextTick } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useQuickSearch } from '@/composables/useQuickSearch'

const {
  searchQuery,
  filteredSnippets,
  selectedIndex,
  copied,
  load,
  moveSelection,
  copyAndClose,
  hide,
  reset,
} = useQuickSearch()

const inputRef = ref<HTMLInputElement | null>(null)

onMounted(async () => {
  await load()
  await nextTick()
  inputRef.value?.focus()

  // 窗口再次显示时重置状态 + 聚焦搜索框。
  await listen('tauri://focus', async () => {
    reset()
    await nextTick()
    inputRef.value?.focus()
  })
})

function onKeydown(e: KeyboardEvent) {
  switch (e.key) {
    case 'Enter': {
      e.preventDefault()
      const s = filteredSnippets.value[selectedIndex.value]
      if (s) void copyAndClose(s)
      break
    }
    case 'ArrowDown':
      e.preventDefault()
      moveSelection(1)
      break
    case 'ArrowUp':
      e.preventDefault()
      moveSelection(-1)
      break
    case 'Escape':
      e.preventDefault()
      void hide()
      break
  }
}

function preview(content: string): string {
  const firstLine = content.split('\n').map(l => l.trim()).find(l => l !== '') ?? ''
  return firstLine.length > 60 ? firstLine.slice(0, 60) + '…' : firstLine
}

async function onClickItem(index: number) {
  selectedIndex.value = index
  const s = filteredSnippets.value[index]
  if (s) await copyAndClose(s)
}
</script>

<template>
  <div class="quick-shell">
    <div v-if="copied" class="copied-overlay">
      <span>已复制 ✓</span>
    </div>

    <div class="search-row">
      <input
        ref="inputRef"
        v-model="searchQuery"
        class="search-input"
        type="text"
        placeholder="搜索片段…（↑↓ 选择，Enter 复制，Esc 关闭）"
        @keydown="onKeydown"
      />
    </div>

    <ul v-if="filteredSnippets.length > 0" class="result-list">
      <li
        v-for="(s, i) in filteredSnippets"
        :key="s.id"
        class="result-item"
        :class="{ active: i === selectedIndex }"
        @mouseenter="selectedIndex = i"
        @click="onClickItem(i)"
      >
        <div class="item-head">
          <span v-if="s.favorite" class="star">★</span>
          <span class="item-title">{{ s.title }}</span>
          <span v-if="s.category_name" class="item-cat">{{ s.category_name }}</span>
        </div>
        <div class="item-preview">{{ preview(s.content) }}</div>
      </li>
    </ul>
    <div v-else class="empty">
      <span v-if="searchQuery.trim()">无匹配片段</span>
      <span v-else>暂无片段，先在主窗口添加</span>
    </div>
  </div>
</template>

<style scoped>
.quick-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-1);
  color: var(--fg-1);
  overflow: hidden;
  border-radius: var(--radius-3);
  position: relative;
}
.copied-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-1);
  z-index: 10;
  font-size: 18px;
  color: var(--accent);
}
.search-row {
  flex: 0 0 auto;
  padding: var(--space-3);
  border-bottom: 1px solid var(--border);
}
.search-input {
  width: 100%;
  padding: var(--space-3) var(--space-4);
  background: var(--bg-3);
  color: var(--fg-1);
  border: 1px solid var(--border);
  border-radius: var(--radius-2);
  font: inherit;
  outline: none;
}
.search-input:focus {
  border-color: var(--accent);
}
.result-list {
  flex: 1 1 auto;
  list-style: none;
  margin: 0;
  padding: var(--space-1) 0;
  overflow: auto;
}
.result-item {
  padding: var(--space-2) var(--space-4);
  cursor: pointer;
  border-left: 2px solid transparent;
}
.result-item:hover {
  background: var(--bg-2);
}
.result-item.active {
  background: var(--bg-3);
  border-left-color: var(--accent);
}
.item-head {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
.star {
  color: var(--accent);
  font-size: 11px;
}
.item-title {
  flex: 1 1 auto;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.item-cat {
  flex: 0 0 auto;
  font-size: 11px;
  color: var(--fg-2);
  background: var(--bg-1);
  padding: 1px 6px;
  border-radius: var(--radius-1);
}
.item-preview {
  margin-top: 2px;
  font-size: 12px;
  color: var(--fg-2);
  font-family: var(--mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.empty {
  flex: 1 1 auto;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fg-2);
  font-size: 13px;
}
</style>
