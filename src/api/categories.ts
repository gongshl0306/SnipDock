// 分类相关 Tauri 命令的薄封装。
//
// 命令名严格对应 target.md §7.1：list_categories / create_category /
// update_category / delete_category。
// 参数用 camelCase（Tauri 默认会把 JS 传来的 camelCase 键映射到
// Rust 命令参数）。错误以 throw 形式抛出，由调用方 catch。

import { invoke } from '@tauri-apps/api/core'
import type { Category } from '@/types/models'

export function listCategories(): Promise<Category[]> {
  return invoke<Category[]>('list_categories')
}

export function createCategory(
  name: string,
  description?: string,
): Promise<Category> {
  return invoke<Category>('create_category', { name, description: description ?? null })
}

export function updateCategory(
  id: number,
  name: string,
  description?: string,
): Promise<Category> {
  return invoke<Category>('update_category', { id, name, description: description ?? null })
}

export function deleteCategory(id: number): Promise<void> {
  return invoke<void>('delete_category', { id })
}
