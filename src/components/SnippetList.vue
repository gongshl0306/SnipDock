<script setup lang="ts">
// 中栏：片段列表。
//
// 数据来自 useSnippets() 单例；选中片段用 selectedSnippetId 高亮。
// 新增按钮调 startCreate()，右栏切换为编辑器。
// M3 直接显示后端返回的已排序结果（§9.4 排序在 SQL）；M4 会接 filteredSnippets。
// 搜索框 M4 才启用，这里先占位禁用。

import { computed } from 'vue'
import { useSnippets } from '@/composables/useSnippets'

const {
  filteredSnippets,
  selectedSnippetId,
  loading,
  selectSnippet,
  startCreate,
  startEdit,
} = useSnippets()

const isEmpty = computed(() => !loading.value && filteredSnippets.value.length === 0)

/** content 单行预览：首行非空文本，截断到 80 字符。 */
function preview(content: string): string {
  const firstLine = content.split('\n').map(l => l.trim()).find(l => l !== '') ?? ''
  return firstLine.length > 80 ? firstLine.slice(0, 80) + '…' : firstLine
}
</script>

<template>
  <div class="list-panel">
    <div class="toolbar">
      <span class="title">片段</span>
      <button class="add-btn" title="新增片段 (Ctrl+N)" @click="startCreate">+</button>
    </div>

    <div v-if="loading" class="state">加载中…</div>
    <div v-else-if="isEmpty" class="state empty">
      <p>暂无片段</p>
      <button class="link-btn" @click="startCreate">新增第一个片段</button>
    </div>

    <ul v-else class="snip-list">
      <li
        v-for="s in filteredSnippets"
        :key="s.id"
        class="snip-item"
        :class="{ active: selectedSnippetId === s.id }"
        @click="selectSnippet(s.id)"
        @dblclick="startEdit(s)"
      >
        <div class="snip-head">
          <span v-if="s.favorite" class="star" title="已收藏">★</span>
          <span class="snip-title">{{ s.title }}</span>
        </div>
        <div class="snip-preview">{{ preview(s.content) }}</div>
        <div class="snip-meta">
          <span v-if="s.category_name" class="cat-tag">{{ s.category_name }}</span>
          <span v-if="s.used_count > 0" class="used">用过 {{ s.used_count }} 次</span>
        </div>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.list-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-2);
}
.toolbar {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3);
  border-bottom: 1px solid var(--border);
}
.toolbar .title {
  font-size: 12px;
  color: var(--fg-2);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.add-btn {
  width: 22px;
  height: 22px;
  border: none;
  border-radius: var(--radius-1);
  background: transparent;
  color: var(--fg-2);
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
}
.add-btn:hover {
  background: var(--bg-3);
  color: var(--accent);
}
.state {
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  color: var(--fg-2);
  font-size: 13px;
}
.empty .link-btn {
  background: none;
  border: none;
  color: var(--accent);
  cursor: pointer;
  font: inherit;
  padding: 0;
}
.empty .link-btn:hover {
  text-decoration: underline;
}
.snip-list {
  flex: 1 1 auto;
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
.snip-item:hover {
  background: var(--bg-3);
}
.snip-item.active {
  background: var(--bg-3);
  border-left-color: var(--accent);
}
.snip-head {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}
.star {
  color: var(--accent);
  font-size: 12px;
}
.snip-title {
  flex: 1 1 auto;
  font-size: 13px;
  color: var(--fg-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.snip-preview {
  margin-top: 2px;
  font-size: 12px;
  color: var(--fg-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--mono);
}
.snip-meta {
  margin-top: var(--space-1);
  display: flex;
  gap: var(--space-2);
  font-size: 11px;
  color: var(--fg-2);
}
.cat-tag {
  padding: 1px 6px;
  background: var(--bg-1);
  border-radius: var(--radius-1);
}
</style>
