// 导入 / 导出 (备份) API。
//
// 命令名 / 参数：见 src-tauri/src/commands/backup.rs。
// 文件选择走 @tauri-apps/plugin-dialog 的 save/open 对话框，
// 选定后把绝对路径传给 Rust 命令；空（用户取消）时返回 null。

import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'

export interface ExportResult {
  path: string
  categories: number
  snippets: number
}

export interface ImportResult {
  created_categories: number
  reused_categories: number
  created_snippets: number
  skipped_snippets: number
}

/** 弹保存对话框 → 调 export_data。用户取消返回 null。 */
export async function exportData(): Promise<ExportResult | null> {
  const path = await save({
    title: '导出 SnipDock 数据',
    defaultPath: 'snipdock-backup.json',
    filters: [{ name: 'JSON', extensions: ['json'] }],
  })
  if (!path) return null
  return invoke<ExportResult>('export_data', { path })
}

/** 弹打开对话框 → 调 import_data。用户取消返回 null。 */
export async function importData(): Promise<ImportResult | null> {
  const path = await open({
    title: '导入 SnipDock 数据',
    multiple: false,
    filters: [{ name: 'JSON', extensions: ['json'] }],
  })
  if (!path || typeof path !== 'string') return null
  return invoke<ImportResult>('import_data', { path })
}
