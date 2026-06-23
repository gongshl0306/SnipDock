// 片段相关 Tauri 命令的薄封装。
//
// 命令名严格对应 target.md §7.2（偏差：search_snippets / copy_snippet 不实现，
// 见 design.md §5.2）。参数用 camelCase，与 Rust 端的 rename_all="camelCase" 对齐。
// 错误以 throw 形式抛出，由调用方 catch。

import { invoke } from '@tauri-apps/api/core'
import type {
  CreateSnippetPayload,
  UpdateSnippetPayload,
  Snippet,
} from '@/types/models'

/** 「全部」视图：返回所有分类下的片段，已按 §9.4 排序。 */
export function listSnippets(): Promise<Snippet[]> {
  return invoke<Snippet[]>('list_snippets')
}

/**
 * 某分类下的片段。
 * 注意：invoke 的 key 必须与 Rust 命令参数名 `category_id` 完全一致
 * （Tauri 2 不自动转 camelCase），所以这里显式写 category_id。
 */
export function listSnippetsByCategory(categoryId: number): Promise<Snippet[]> {
  return invoke<Snippet[]>('list_snippets_by_category', {
    category_id: categoryId,
  })
}

/** 已收藏的片段（跨原分类）。「收藏」虚拟分类用。 */
export function listSnippetsFavorites(): Promise<Snippet[]> {
  return invoke<Snippet[]>('list_snippets_favorites')
}

export function createSnippet(payload: CreateSnippetPayload): Promise<Snippet> {
  return invoke<Snippet>('create_snippet', { payload })
}

export function updateSnippet(payload: UpdateSnippetPayload): Promise<Snippet> {
  return invoke<Snippet>('update_snippet', { payload })
}

export function deleteSnippet(id: number): Promise<void> {
  return invoke<void>('delete_snippet', { id })
}

/** 标记片段被使用：used_count+1, last_used_at=now。复制流程后调用。 */
export function markSnippetUsed(id: number): Promise<Snippet> {
  return invoke<Snippet>('mark_snippet_used', { id })
}
