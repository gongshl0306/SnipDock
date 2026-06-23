// 快速搜索窗口的状态与逻辑（两栏版）。
//
// 左栏：分类（全部 / ★ 收藏 / 真实分类）。右栏：当前选中分类的片段。
// 顶部搜索框过滤右栏的片段。排序由后端保证（used_count DESC → updated_at DESC）。
//
// 复用 useSnippets 的 copySnippet（写剪贴板 + mark_used）。
// 分类列表从 useCategories 拉（独立 webview，各自一份 singleton）。

import { ref, computed } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useSnippets } from '@/composables/useSnippets'
import { useCategories, type CategorySelection } from '@/composables/useCategories'
import * as snippetApi from '@/api/snippets'
import type { Snippet } from '@/types/models'

const { copySnippet } = useSnippets()
const { categories, load: loadCategories } = useCategories()

const selectedCategoryId = ref<CategorySelection>('favorites')
const snippets = ref<Snippet[]>([])
const searchQuery = ref('')
const selectedIndex = ref(0)
const copied = ref(false)
const loaded = ref(false)

/** 左栏分类列表（含虚拟项）。 */
const categoryList = computed(() => [
  { id: 'all' as const, name: '全部' },
  { id: 'favorites' as const, name: '收藏' },
  ...categories.value.map(c => ({ id: c.id, name: c.name })),
])

/** 右栏：按搜索词过滤当前分类的片段。 */
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

/** 加载指定分类下的片段。 */
async function loadSnippetsFor(selection: CategorySelection) {
  if (selection === 'all') {
    snippets.value = await snippetApi.listSnippets()
  } else if (selection === 'favorites') {
    snippets.value = await snippetApi.listSnippetsFavorites()
  } else {
    snippets.value = await snippetApi.listSnippetsByCategory(selection)
  }
}

/** 初始化：加载分类 + 默认选中分类的片段。 */
async function init() {
  if (loaded.value) return
  loaded.value = true
  await loadCategories()
  await loadSnippetsFor(selectedCategoryId.value)
}

/** 切换左栏分类，重新拉右栏片段。 */
async function selectCategory(selection: CategorySelection) {
  selectedCategoryId.value = selection
  searchQuery.value = ''
  selectedIndex.value = 0
  await loadSnippetsFor(selection)
}

function moveSelection(dir: 1 | -1) {
  const len = filteredSnippets.value.length
  if (len === 0) return
  selectedIndex.value = (selectedIndex.value + dir + len) % len
}

/** 复制并关闭窗口。 */
async function copyAndClose(snippet: Snippet): Promise<void> {
  await copySnippet(snippet)
  copied.value = true
  setTimeout(async () => {
    copied.value = false
    searchQuery.value = ''
    selectedIndex.value = 0
    await getCurrentWindow().hide()
  }, 600)
}

/** 隐藏窗口。 */
async function hide() {
  await getCurrentWindow().hide()
}

/** 重置状态（窗口再次显示时调）。 */
function reset() {
  searchQuery.value = ''
  selectedIndex.value = 0
  copied.value = false
}

export function useQuickSearch() {
  return {
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
  }
}
