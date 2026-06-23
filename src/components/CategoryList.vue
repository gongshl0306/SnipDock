<script setup lang="ts">
// 左栏：分类列表 + 工具栏。
//
// 「全部」是虚拟项，对应 selectedCategoryId = 'all'（全局检索）。
// 「★ 收藏」是虚拟项，对应 'favorites'（跨原分类的收藏片段）。
// 分类的真实数据来自 useCategories() 单例。
// hover「★ 收藏」不显示 ✎/✕（不可重命名/删除）。
// 新增/重命名用 ModalDialog 的表单模式；删除用确认模式。
// 删除失败（CategoryNotEmpty）在底部 status 里提示。

import { ref } from 'vue'
import { Plus, Edit, Close, StarFilled } from '@element-plus/icons-vue'
import { useCategories } from '@/composables/useCategories'
import { parseAppError } from '@/types/models'
import ModalDialog, { type FieldSchema } from './ModalDialog.vue'

const { categories, selectedCategoryId, selectCategory, create, update, remove } =
  useCategories()

// 模态状态机：当前处于哪个操作。null = 关闭。
type ModalState =
  | { kind: 'create' }
  | { kind: 'rename'; id: number; oldName: string; oldDesc: string }
  | { kind: 'delete'; id: number; name: string }

const modal = ref<ModalState | null>(null)
const statusMsg = ref('')
const statusErr = ref('')

// 各操作对应的模态配置（computed 化避免散落）
function modalTitle(): string {
  if (!modal.value) return ''
  switch (modal.value.kind) {
    case 'create': return '新增分类'
    case 'rename': return '重命名分类'
    case 'delete': return '删除分类'
  }
}

function modalFields(): FieldSchema[] {
  if (modal.value?.kind === 'create') {
    return [
      { name: 'name', label: '名称', placeholder: 'Kubernetes', required: true },
      { name: 'description', label: '描述', placeholder: '可选' },
    ]
  }
  if (modal.value?.kind === 'rename') {
    return [
      { name: 'name', label: '名称', required: true, defaultValue: modal.value.oldName },
      { name: 'description', label: '描述', defaultValue: modal.value.oldDesc },
    ]
  }
  return []
}

function modalMessage(): string {
  if (modal.value?.kind === 'delete') {
    return `确定删除分类「${modal.value.name}」吗？`
  }
  return ''
}

async function onSubmit(values: Record<string, string>) {
  if (!modal.value) return
  statusErr.value = ''
  try {
    if (modal.value.kind === 'create') {
      await create(values.name, values.description)
      statusMsg.value = `已新增分类「${values.name}」`
    } else if (modal.value.kind === 'rename') {
      await update(modal.value.id, values.name, values.description)
      statusMsg.value = `已更新分类「${values.name}」`
    } else if (modal.value.kind === 'delete') {
      await remove(modal.value.id)
      statusMsg.value = '已删除分类'
    }
  } catch (e) {
    const appErr = parseAppError(e)
    if (appErr?.type === 'CategoryNotEmpty') {
      statusErr.value = '该分类下还有片段，请先移除或删除片段'
    } else if (appErr) {
      statusErr.value = appErr.message
    } else {
      statusErr.value = String(e)
    }
  } finally {
    modal.value = null
  }
}

function startCreate() {
  modal.value = { kind: 'create' }
}
function startRename(id: number, name: string, description: string) {
  modal.value = { kind: 'rename', id, oldName: name, oldDesc: description }
}
function startDelete(id: number, name: string) {
  modal.value = { kind: 'delete', id, name }
}
</script>

<template>
  <div class="cat-panel">
    <div class="toolbar">
      <span class="title">分类</span>
      <button class="add-btn" title="新增分类 (Ctrl+Shift+N)" @click="startCreate">
        <Plus class="icon" />
      </button>
    </div>

    <ul class="cat-list">
      <li
        class="cat-item all"
        :class="{ active: selectedCategoryId === 'all' }"
        @click="selectCategory('all')"
      >
        <span class="cat-name">全部</span>
      </li>
      <li
        class="cat-item favorites"
        :class="{ active: selectedCategoryId === 'favorites' }"
        @click="selectCategory('favorites')"
      >
        <span class="cat-name fav"><StarFilled class="icon star-icon" /> 收藏</span>
      </li>
      <li
        v-for="c in categories"
        :key="c.id"
        class="cat-item"
        :class="{ active: selectedCategoryId === c.id }"
        @click="selectCategory(c.id)"
      >
        <span class="cat-name" :title="c.description">{{ c.name }}</span>
        <span class="cat-actions">
          <button
            class="icon-btn"
            title="重命名"
            @click.stop="startRename(c.id, c.name, c.description)"
          ><Edit class="icon" /></button>
          <button
            class="icon-btn danger"
            title="删除"
            @click.stop="startDelete(c.id, c.name)"
          ><Close class="icon" /></button>
        </span>
      </li>
    </ul>

    <div v-if="statusErr" class="status err">{{ statusErr }}</div>
    <div v-else-if="statusMsg" class="status ok">{{ statusMsg }}</div>

    <ModalDialog
      :model-value="modal !== null"
      :title="modalTitle()"
      :fields="modalFields()"
      :message="modalMessage()"
      :danger="modal?.kind === 'delete'"
      :confirm-text="modal?.kind === 'delete' ? '删除' : '保存'"
      @update:model-value="(v: boolean) => { if (!v) modal = null }"
      @submit="onSubmit"
    />
  </div>
</template>

<style scoped>
.cat-panel {
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
.cat-list {
  flex: 1 1 auto;
  list-style: none;
  margin: 0;
  padding: var(--space-1) 0;
  overflow: auto;
}
.cat-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-2) var(--space-3);
  cursor: pointer;
  border-left: 2px solid transparent;
  position: relative;
}
.cat-item:hover {
  background: var(--bg-3);
}
.cat-item.active {
  background: var(--bg-3);
  border-left-color: var(--accent);
  color: var(--accent);
}
.cat-name {
  flex: 1 1 auto;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.cat-name.fav {
  color: #e6c43a;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.star-icon {
  font-size: 13px;
}
.cat-item.favorites.active .cat-name.fav {
  color: #f6d24c;
}
.cat-actions {
  flex: 0 0 auto;
  display: none;
  gap: 2px;
}
.cat-item:hover .cat-actions {
  display: flex;
}
.icon-btn {
  width: 20px;
  height: 20px;
  border: none;
  border-radius: var(--radius-1);
  background: transparent;
  color: var(--fg-2);
  cursor: pointer;
  font-size: 12px;
  line-height: 1;
  padding: 0;
}
.icon-btn:hover {
  background: var(--bg-1);
  color: var(--fg-1);
}
.icon-btn.danger:hover {
  color: var(--danger);
}
.status {
  flex: 0 0 auto;
  padding: var(--space-2) var(--space-3);
  border-top: 1px solid var(--border);
  font-size: 12px;
}
.status.err { color: var(--danger); }
.status.ok { color: var(--fg-2); }
</style>
