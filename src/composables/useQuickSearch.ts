// 快速搜索窗口的状态与逻辑。
//
// 与主窗口的 useSnippets 独立（不同 webview，模块级单例各自一份）。
// quick 窗口只在「全部」语义下搜全部片段，不需要 selectedCategoryId。
//
// 搜索：spec §8.1 的 4 字段匹配（title/content/description/category_name），
// 大小写不敏感、子串包含。结果沿用后端 §9.4 排序（load 时已排好）。
//
// 复制流程：copySnippet（写剪贴板 + mark_snippet_used）→ 显示短暂 toast →
// 调 getCurrentWindow().hide() 隐藏窗口 + 清空搜索框。
// 失焦隐藏由 Rust 端 on_window_event 处理；复制后主动 hide 避免依赖失焦。

import { ref, computed } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useSnippets } from '@/composables/useSnippets'
import type { Snippet } from '@/types/models'

const { snippets, load, copySnippet } = useSnippets()

const searchQuery = ref('')
const selectedIndex = ref(0)
const copied = ref(false)

/** 按搜索词过滤；空词时返回全部（后端已排序）。 */
const filteredSnippets = computed<Snippet[]>(() => {
  const q = searchQuery.value.trim().toLowerCase()
  if (!q) return snippets.value
  return snippets.value.filter(s => {
    const hay = [
      s.title,
      s.content,
      s.description,
      s.category_name ?? '',
    ]
      .join('\n')
      .toLowerCase()
    return hay.includes(q)
  })
})

/** 选中索引在过滤结果范围内 clamp。 */
function clampSelected() {
  const len = filteredSnippets.value.length
  if (len === 0) {
    selectedIndex.value = 0
    return
  }
  if (selectedIndex.value >= len) selectedIndex.value = len - 1
  if (selectedIndex.value < 0) selectedIndex.value = 0
}

/** 当前选中的片段（可能为 null）。 */
const selectedSnippet = computed<Snippet | null>(() => {
  clampSelected()
  return filteredSnippets.value[selectedIndex.value] ?? null
})

/** ↑/↓ 切换选中。dir = +1 下移、-1 上移。 */
function moveSelection(dir: 1 | -1) {
  const len = filteredSnippets.value.length
  if (len === 0) return
  selectedIndex.value = (selectedIndex.value + dir + len) % len
}

/** 复制并关闭窗口。 */
async function copyAndClose(snippet: Snippet): Promise<void> {
  await copySnippet(snippet)
  copied.value = true
  // 短暂显示「已复制」后隐藏；150ms 让用户看到反馈。
  setTimeout(async () => {
    copied.value = false
    searchQuery.value = ''
    selectedIndex.value = 0
    await getCurrentWindow().hide()
  }, 600)
}

/** 隐藏窗口（Esc / 失焦兜底）。 */
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
    snippets,
    searchQuery,
    filteredSnippets,
    selectedIndex,
    selectedSnippet,
    copied,
    load,
    moveSelection,
    copyAndClose,
    hide,
    reset,
  }
}
