// 窗口内键盘快捷键（spec §11）。
//
// 全部监听 document keydown。编辑器打开时，除 Esc 外其他快捷键暂停
// （典型 modal 行为），避免与输入冲突。
//
// 不含全局呼出快捷键——那个由后端 global-shortcut 插件注册（设置里录制）。
//
// Esc 的优先级：编辑器打开时 → 关闭编辑器；否则 → 隐藏窗口。

import { useSnippets } from '@/composables/useSnippets'
import { useCategories } from '@/composables/useCategories'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { Category } from '@/types/models'

let bound = false

/** ↑/↓ 切换 selectedSnippetId（在 filteredSnippets 上）。 */
function moveSnippet(dir: 1 | -1) {
  const { filteredSnippets, selectedSnippetId, selectSnippet } = useSnippets()
  const list = filteredSnippets.value
  if (list.length === 0) return
  const idx = list.findIndex(s => s.id === selectedSnippetId.value)
  const next = dir > 0 ? Math.min(idx + 1, list.length - 1) : Math.max(idx - 1, 0)
  selectSnippet(list[next]?.id ?? null)
}

/** Ctrl+↑/↓ 切换分类（全部 → 收藏 → 真实分类）。 */
function moveCategory(dir: 1 | -1) {
  const { categories, selectedCategoryId, selectCategory } = useCategories()
  // 列表顺序：'all' → 'favorites' → 真实分类按 id
  const ordered: (number | 'all' | 'favorites')[] = [
    'all',
    'favorites',
    ...categories.value.map((c: Category) => c.id),
  ]
  const idx = ordered.indexOf(selectedCategoryId.value)
  const next = Math.min(Math.max(idx + dir, 0), ordered.length - 1)
  if (next !== idx && next >= 0) {
    selectCategory(ordered[next])
  }
}

function onKeydown(e: KeyboardEvent) {
  const { isEditing, cancelEdit, startCreate } = useSnippets()

  // Esc：编辑器打开时关编辑器，否则隐藏窗口。
  if (e.key === 'Escape') {
    if (isEditing.value) {
      e.preventDefault()
      cancelEdit()
    } else {
      e.preventDefault()
      void getCurrentWindow().hide()
    }
    return
  }

  // 编辑器打开时，其他快捷键暂停。
  if (isEditing.value) return

  // 在 input/textarea/select 里时不触发功能快捷键（让用户正常输入）。
  const tag = (e.target as HTMLElement)?.tagName?.toLowerCase()
  const isTyping = tag === 'input' || tag === 'textarea' || tag === 'select'
  if (isTyping) return

  const ctrl = e.ctrlKey || e.metaKey

  // Ctrl+N：新增片段
  if (ctrl && e.key.toLowerCase() === 'n' && !e.shiftKey) {
    e.preventDefault()
    startCreate()
    return
  }
  // Ctrl+E：编辑当前片段
  if (ctrl && e.key.toLowerCase() === 'e') {
    e.preventDefault()
    startEditCurrent()
    return
  }
  // Ctrl+D：删除当前片段（不弹确认，直接走详情页的删除流程）
  // 这里简化：直接调 remove，后续可加确认。
  if (ctrl && e.key.toLowerCase() === 'd') {
    e.preventDefault()
    deleteCurrent()
    return
  }
  // Enter：复制当前选中片段
  if (e.key === 'Enter') {
    e.preventDefault()
    copyCurrent()
    return
  }
  // ↑/↓：切换片段
  if (!ctrl && e.key === 'ArrowDown') {
    e.preventDefault()
    moveSnippet(1)
    return
  }
  if (!ctrl && e.key === 'ArrowUp') {
    e.preventDefault()
    moveSnippet(-1)
    return
  }
  // Ctrl+↑/↓：切换分类
  if (ctrl && e.key === 'ArrowDown') {
    e.preventDefault()
    moveCategory(1)
    return
  }
  if (ctrl && e.key === 'ArrowUp') {
    e.preventDefault()
    moveCategory(-1)
    return
  }
}

function startEditCurrent() {
  const { selectedSnippet, startEdit } = useSnippets()
  if (selectedSnippet.value) startEdit(selectedSnippet.value)
}

function deleteCurrent() {
  const { selectedSnippet, remove } = useSnippets()
  if (selectedSnippet.value) {
    // 简单确认；后续可替换为 ModalDialog 确认。
    if (window.confirm(`确定删除「${selectedSnippet.value.title}」？`)) {
      void remove(selectedSnippet.value.id)
    }
  }
}

function copyCurrent() {
  const { selectedSnippet, copySnippet } = useSnippets()
  if (selectedSnippet.value) {
    void copySnippet(selectedSnippet.value)
  }
}

/** 注册全局 keydown 监听。幂等。 */
export function bindShortcuts() {
  if (bound) return
  bound = true
  window.addEventListener('keydown', onKeydown)
}

/** 注销（测试用，正常不调）。 */
export function unbindShortcuts() {
  bound = false
  window.removeEventListener('keydown', onKeydown)
}
