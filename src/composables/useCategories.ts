// 分类状态 + CRUD。
//
// design.md §6.2：模块级单例 ref，不用 Pinia。
// 变更操作成功后本地同步 categories.value，避免每次重新 list。
//
// selectedCategoryId 的语义（spec §9.3）：
//   null  → 全部 → 全局检索
//   数字  → 只在该分类内检索

import { ref } from 'vue'
import * as api from '@/api/categories'
import type { Category } from '@/types/models'

const categories = ref<Category[]>([])
const selectedCategoryId = ref<number | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

/** 启动时调用：从后端拉取全部分类。 */
async function load() {
  loading.value = true
  error.value = null
  try {
    categories.value = await api.listCategories()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

/** 新建分类。成功后追加到 categories（保持服务端返回的顺序）。 */
async function create(name: string, description?: string): Promise<Category> {
  const created = await api.createCategory(name, description)
  categories.value = await api.listCategories()
  return created
}

/** 更新分类。成功后本地替换对应项。 */
async function update(id: number, name: string, description?: string): Promise<Category> {
  const updated = await api.updateCategory(id, name, description)
  const idx = categories.value.findIndex(c => c.id === id)
  if (idx >= 0) categories.value[idx] = updated
  return updated
}

/**
 * 删除分类。
 * 成功后从 categories 移除；若当前选中的就是它，回退到「全部」(null)。
 * 抛出错误（如 CategoryNotEmpty）由调用方处理。
 */
async function remove(id: number): Promise<void> {
  await api.deleteCategory(id)
  categories.value = categories.value.filter(c => c.id !== id)
  if (selectedCategoryId.value === id) {
    selectedCategoryId.value = null
  }
}

function selectCategory(id: number | null) {
  selectedCategoryId.value = id
}

export function useCategories() {
  return {
    categories,
    selectedCategoryId,
    loading,
    error,
    load,
    create,
    update,
    remove,
    selectCategory,
  }
}
