// 片段状态 + CRUD + 加载。
//
// design.md §6.2：模块级单例 ref，不用 Pinia。
// 加载策略：watch useCategories().selectedCategoryId，
//   'all'        → list_snippets（全部）
//   'favorites'  → list_snippets_favorites（已收藏，跨原分类）
//   数字          → list_snippets_by_category（指定分类）
// 选中片段若不在新列表里则清空。

import { ref, computed, watch } from 'vue'
import * as api from '@/api/snippets'
import type {
  CreateSnippetPayload,
  UpdateSnippetPayload,
  Snippet,
} from '@/types/models'
import { useCategories } from '@/composables/useCategories'

const snippets = ref<Snippet[]>([])
const selectedSnippetId = ref<number | null>(null)
const selectedSnippet = ref<Snippet | null>(null)
const searchQuery = ref('')
const isEditing = ref(false)

/**
 * 搜索过滤：在当前加载的 snippets 中按 searchQuery 做大小写不敏感子串匹配，
 * 字段包括 title / content / category_name。空查询返回全部（后端已按 §9.4 排序）。
 */
const filteredSnippets = computed<Snippet[]>(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return snippets.value
  return snippets.value.filter(s => {
    const hay = [s.title, s.content, s.category_name ?? '']
      .join('\n')
      .toLowerCase()
    return hay.includes(q)
  })
})
/** 正在编辑的片段；null = 新增模式。 */
const editingSnippet = ref<Snippet | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

let watching = false

/** 加载当前选中分类（或全部/收藏）下的片段。 */
async function load() {
  const { selectedCategoryId } = useCategories()
  loading.value = true
  error.value = null
  try {
    const selection = selectedCategoryId.value
    if (selection === 'all') {
      snippets.value = await api.listSnippets()
    } else if (selection === 'favorites') {
      snippets.value = await api.listSnippetsFavorites()
    } else {
      // 数字 = 真实分类 id
      snippets.value = await api.listSnippetsByCategory(selection)
    }
    // 选中片段若已不在当前列表，清空选中。
    if (
      selectedSnippetId.value !== null &&
      !snippets.value.some(s => s.id === selectedSnippetId.value)
    ) {
      selectSnippet(null)
    } else {
      syncSelectedSnippet()
    }
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

/** 把 selectedSnippet 与 selectedSnippetId 同步。 */
function syncSelectedSnippet() {
  if (selectedSnippetId.value === null) {
    selectedSnippet.value = null
    return
  }
  selectedSnippet.value =
    snippets.value.find(s => s.id === selectedSnippetId.value) ?? null
}

function selectSnippet(id: number | null) {
  selectedSnippetId.value = id
  syncSelectedSnippet()
}

async function create(payload: CreateSnippetPayload): Promise<Snippet> {
  const created = await api.createSnippet(payload)
  // 重新加载以保持排序（§9.4 由后端保证）。
  await load()
  selectSnippet(created.id)
  return created
}

async function update(payload: UpdateSnippetPayload): Promise<Snippet> {
  const updated = await api.updateSnippet(payload)
  await load()
  return updated
}

async function remove(id: number): Promise<void> {
  await api.deleteSnippet(id)
  if (selectedSnippetId.value === id) selectSnippet(null)
  await load()
}

/** 复制流程：写剪贴板 + 标记使用 + 本地同步 used_count/last_used_at。 */
async function copySnippet(snippet: Snippet): Promise<void> {
  await navigator.clipboard.writeText(snippet.content)
  const updated = await api.markSnippetUsed(snippet.id)
  // 本地直接替换，避免整表 reload。
  const idx = snippets.value.findIndex(s => s.id === snippet.id)
  if (idx >= 0) snippets.value[idx] = updated
  if (selectedSnippetId.value === snippet.id) selectedSnippet.value = updated
}

/**
 * 切换片段的收藏状态。
 * 走 update_snippet，其他字段保持原值。成功后：
 *   - 若当前在「收藏」视图下取消收藏 → 整表 reload（让该片段消失）
 *   - 其他情况本地替换即可
 */
async function toggleFavorite(snippet: Snippet): Promise<void> {
  const newFav = snippet.favorite === 0 ? 1 : 0
  const updated = await api.updateSnippet({
    id: snippet.id,
    category_id: snippet.category_id,
    title: snippet.title,
    content: snippet.content,
    favorite: newFav,
  })
  const { selectedCategoryId } = useCategories()
  if (selectedCategoryId.value === 'favorites' && newFav === 0) {
    // 在收藏视图下取消收藏 → 该片段应消失。
    await load()
  } else {
    const idx = snippets.value.findIndex(s => s.id === snippet.id)
    if (idx >= 0) snippets.value[idx] = updated
    if (selectedSnippetId.value === snippet.id) selectedSnippet.value = updated
  }
}

/** 进入新增模式。editingSnippet = null。 */
function startCreate() {
  editingSnippet.value = null
  isEditing.value = true
}

/** 进入编辑模式。 */
function startEdit(snippet: Snippet) {
  editingSnippet.value = snippet
  isEditing.value = true
}

function cancelEdit() {
  isEditing.value = false
  editingSnippet.value = null
}

/** 结束编辑（保存成功后调用）。 */
function endEdit() {
  isEditing.value = false
  editingSnippet.value = null
}

/**
 * 注册「分类切换 → 自动重载片段」的 watch。
 * 幂等：重复调用不会重复注册。App 启动时调用一次。
 */
function bindCategoryWatcher() {
  if (watching) return
  watching = true
  const { selectedCategoryId } = useCategories()
  watch(selectedCategoryId, () => {
    void load()
  })
}

export function useSnippets() {
  return {
    // 状态
    snippets,
    filteredSnippets,
    selectedSnippetId,
    selectedSnippet,
    searchQuery,
    isEditing,
    editingSnippet,
    loading,
    error,
    // 动作
    load,
    selectSnippet,
    create,
    update,
    remove,
    copySnippet,
    toggleFavorite,
    startCreate,
    startEdit,
    cancelEdit,
    endEdit,
    bindCategoryWatcher,
  }
}
