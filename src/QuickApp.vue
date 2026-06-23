<script setup lang="ts">
// 快速搜索窗口（两栏）：左栏分类 / 右栏片段。
// 600x420 无边框置顶窗口。搜索框跨两栏。呼出时默认选中「收藏」。

import { onMounted, ref, nextTick } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { StarFilled, CircleCheckFilled } from '@element-plus/icons-vue'
import { useQuickSearch } from '@/composables/useQuickSearch'
import Highlight from '@/components/Highlight.vue'

const {
  categoryList,
  selectedCategoryId,
  searchQuery,
  filteredSnippets,
  selectedIndex,
  copied,
  init,
  selectCategory,
  moveSelection,
  copyAndClose,
  hide,
  reset,
} = useQuickSearch()

const inputRef = ref<HTMLInputElement | null>(null)

onMounted(async () => {
  await init()
  await nextTick()
  inputRef.value?.focus()

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
  return firstLine.length > 50 ? firstLine.slice(0, 50) + '…' : firstLine
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
      <CircleCheckFilled class="icon copied-icon" />
      <span>已复制</span>
    </div>

    <div class="search-row">
      <input
        ref="inputRef"
        v-model="searchQuery"
        class="search-input"
        type="text"
        placeholder="搜索…"
        @keydown="onKeydown"
      />
    </div>

    <div class="cols">
      <ul class="cat-col">
        <li
          v-for="cat in categoryList"
          :key="cat.id"
          class="cat-item"
          :class="{
            active: selectedCategoryId === cat.id,
            fav: cat.id === 'favorites',
          }"
          @click="selectCategory(cat.id)"
        >
          <StarFilled v-if="cat.id === 'favorites'" class="icon" />
          {{ cat.name }}
        </li>
      </ul>

      <ul v-if="filteredSnippets.length > 0" class="snip-col">
        <li
          v-for="(s, i) in filteredSnippets"
          :key="s.id"
          class="snip-item"
          :class="{ active: i === selectedIndex }"
          @mouseenter="selectedIndex = i"
          @click="onClickItem(i)"
        >
          <div class="item-title">
            <Highlight :text="s.title" :query="searchQuery" />
          </div>
          <div class="item-preview">
            <Highlight :text="preview(s.content)" :query="searchQuery" />
          </div>
        </li>
      </ul>
      <div v-else class="empty">
        <span>暂无片段</span>
      </div>
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
  gap: var(--space-2);
  background: var(--bg-1);
  z-index: 10;
  font-size: 18px;
  color: var(--accent);
}
.copied-icon {
  font-size: 24px;
  color: #22c55e;
}
.search-row {
  flex: 0 0 auto;
  padding: var(--space-2) var(--space-3);
  border-bottom: 1px solid var(--border);
}
.search-input {
  width: 100%;
  padding: var(--space-2) var(--space-3);
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
.cols {
  flex: 1 1 auto;
  display: grid;
  grid-template-columns: 30% 70%;
  min-height: 0;
}
.cat-col {
  list-style: none;
  margin: 0;
  padding: var(--space-1) 0;
  overflow: auto;
  border-right: 1px solid var(--border);
  background: var(--bg-2);
}
.cat-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: var(--space-2) var(--space-3);
  font-size: 12px;
  cursor: pointer;
  border-left: 2px solid transparent;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.cat-item:hover { background: var(--bg-3); }
.cat-item.active {
  background: var(--bg-3);
  border-left-color: var(--accent);
  color: var(--accent);
}
.cat-item.fav { color: #e6c43a; }
.cat-item.fav.active { color: #f6d24c; }
.snip-col {
  list-style: none;
  margin: 0;
  padding: var(--space-1) 0;
  overflow: auto;
}
.snip-item {
  padding: var(--space-2) var(--space-3);
  cursor: pointer;
  border-left: 2px solid transparent;
}
.snip-item:hover { background: var(--bg-2); }
.snip-item.active {
  background: var(--bg-3);
  border-left-color: var(--accent);
}
.item-title {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.item-preview {
  margin-top: 2px;
  font-size: 11px;
  color: var(--fg-2);
  font-family: var(--mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fg-2);
  font-size: 12px;
}
</style>
