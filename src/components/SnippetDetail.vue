<script setup lang="ts">
// 右栏（只读态）：展示选中片段的全部字段 + 操作行。
//
// 复制按钮：navigator.clipboard + markSnippetUsed（design.md §6.4 偏差方案）。
// 编辑/删除：切换 useSnippets 的 isEditing 或调 remove。
// 删除用 ModalDialog 确认。

import { ref } from 'vue'
import { useSnippets } from '@/composables/useSnippets'
import { useCategories } from '@/composables/useCategories'
import { parseAppError } from '@/types/models'
import ModalDialog from './ModalDialog.vue'

const {
  selectedSnippet,
  copySnippet,
  startEdit,
  remove,
  toggleFavorite,
} = useSnippets()
const { categories } = useCategories()

const copied = ref(false)
const errorMsg = ref('')
const confirmDelete = ref(false)

/** 分类名：优先用 snippet 自带的 category_name（join 出来的），否则本地查表。 */
function categoryName(id: number, fromJoin?: string): string {
  if (fromJoin) return fromJoin
  return categories.value.find(c => c.id === id)?.name ?? ''
}

/** RFC3339 截断到分钟显示。 */
function fmtTime(ts?: string): string {
  if (!ts) return '—'
  return ts.replace('T', ' ').slice(0, 16)
}

async function onCopy() {
  if (!selectedSnippet.value) return
  errorMsg.value = ''
  try {
    await copySnippet(selectedSnippet.value)
    copied.value = true
    setTimeout(() => { copied.value = false }, 1500)
  } catch (e) {
    errorMsg.value = parseAppError(e)?.message ?? String(e)
  }
}

async function onToggleFavorite() {
  if (!selectedSnippet.value) return
  errorMsg.value = ''
  try {
    await toggleFavorite(selectedSnippet.value)
  } catch (e) {
    errorMsg.value = parseAppError(e)?.message ?? String(e)
  }
}

async function onConfirmDelete() {
  if (!selectedSnippet.value) return
  errorMsg.value = ''
  try {
    await remove(selectedSnippet.value.id)
  } catch (e) {
    errorMsg.value = parseAppError(e)?.message ?? String(e)
  }
}
</script>

<template>
  <div class="detail-panel">
    <template v-if="selectedSnippet">
      <div class="detail-scroll">
        <h2 class="d-title">
          <span
            class="star-toggle"
            :class="{ starred: selectedSnippet.favorite !== 0 }"
            title="切换收藏"
            @click="onToggleFavorite"
          >{{ selectedSnippet.favorite !== 0 ? '★' : '☆' }}</span>
          {{ selectedSnippet.title }}
        </h2>

        <dl class="meta">
          <div class="meta-row">
            <dt>分类</dt>
            <dd>{{ categoryName(selectedSnippet.category_id, selectedSnippet.category_name) }}</dd>
          </div>
          <div class="meta-row">
            <dt>使用</dt>
            <dd>{{ selectedSnippet.used_count }} 次 · 最近 {{ fmtTime(selectedSnippet.last_used_at) }}</dd>
          </div>
          <div class="meta-row">
            <dt>更新</dt>
            <dd>{{ fmtTime(selectedSnippet.updated_at) }}</dd>
          </div>
        </dl>

        <div class="content-label">内容</div>
        <pre class="content">{{ selectedSnippet.content }}</pre>
      </div>

      <div v-if="errorMsg" class="err-msg">{{ errorMsg }}</div>

      <div class="actions">
        <button class="btn primary" @click="onCopy">
          {{ copied ? '已复制 ✓' : '复制 (Enter)' }}
        </button>
        <button class="btn" @click="startEdit(selectedSnippet)">编辑 (Ctrl+E)</button>
        <button class="btn danger" @click="confirmDelete = true">删除 (Ctrl+D)</button>
      </div>
    </template>

    <div v-else class="empty">
      <p>未选中片段</p>
      <p class="hint">在左侧列表点选一条，或点 + 新增</p>
    </div>

    <ModalDialog
      :model-value="confirmDelete"
      title="删除片段"
      :message="`确定删除「${selectedSnippet?.title ?? ''}」吗？此操作不可撤销。`"
      confirm-text="删除"
      :danger="true"
      @update:model-value="(v: boolean) => { if (!v) confirmDelete = false }"
      @submit="onConfirmDelete"
    />
  </div>
</template>

<style scoped>
.detail-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-2);
}
.detail-scroll {
  flex: 1 1 auto;
  overflow: auto;
  padding: var(--space-4);
}
.d-title {
  margin: 0 0 var(--space-3);
  font-size: 16px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
.star {
  color: var(--accent);
  font-size: 14px;
}
.star-toggle {
  cursor: pointer;
  color: var(--fg-2);
  font-size: 16px;
  transition: color 0.15s;
}
.star-toggle:hover {
  color: #e6c43a;
}
.star-toggle.starred {
  color: #e6c43a;
}
.meta {
  margin: 0 0 var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.meta-row {
  display: flex;
  gap: var(--space-3);
  font-size: 12px;
}
.meta-row dt {
  flex: 0 0 48px;
  color: var(--fg-2);
  margin: 0;
}
.meta-row dd {
  margin: 0;
  color: var(--fg-1);
}
code {
  font-family: var(--mono);
  background: var(--bg-3);
  padding: 1px 6px;
  border-radius: var(--radius-1);
  font-size: 11px;
}
.content-label {
  font-size: 12px;
  color: var(--fg-2);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: var(--space-2);
}
.content {
  margin: 0;
  padding: var(--space-3);
  background: var(--bg-1);
  border: 1px solid var(--border);
  border-radius: var(--radius-2);
  font-family: var(--mono);
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
  overflow: auto;
  max-height: calc(100vh - 360px);
}
.err-msg {
  flex: 0 0 auto;
  padding: var(--space-2) var(--space-4);
  color: var(--danger);
  font-size: 12px;
  border-top: 1px solid var(--border);
}
.actions {
  flex: 0 0 auto;
  display: flex;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--border);
}
.btn {
  padding: var(--space-2) var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-2);
  background: var(--bg-3);
  color: var(--fg-1);
  cursor: pointer;
  font: inherit;
}
.btn:hover {
  border-color: var(--fg-2);
}
.btn.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.btn.primary:hover {
  filter: brightness(1.1);
}
.btn.danger {
  color: var(--danger);
}
.btn.danger:hover {
  background: var(--danger);
  color: #fff;
  border-color: var(--danger);
}
.empty {
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  color: var(--fg-2);
  font-size: 13px;
}
.empty .hint {
  font-size: 12px;
  margin: 0;
}
</style>
