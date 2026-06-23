// 前端领域类型，严格对应 target.md §6。
//
// 字段名、可空性、类型与 Rust 端 CategoryRow / 命令参数保持一致。
// M2 实际只用 Category；Snippet / Payload 为 M3 预埋。

/** 分类。对应 categories 表一行。spec §6.1 */
export interface Category {
  id: number
  name: string
  description: string
  sort_order: number
  created_at: string
  updated_at: string
}

/** 片段。category_name 由全局检索时 join 得到，可空。spec §6.2 */
export interface Snippet {
  id: number
  category_id: number
  category_name?: string
  title: string
  content: string
  description: string
  language: string
  favorite: number
  used_count: number
  created_at: string
  updated_at: string
  last_used_at?: string
}

/** 新建片段的载荷。spec §6.3 */
export interface CreateSnippetPayload {
  category_id: number
  title: string
  content: string
  description?: string
  language?: string
  favorite?: number
}

/** 更新片段的载荷。spec §6.4 */
export interface UpdateSnippetPayload {
  id: number
  category_id: number
  title: string
  content: string
  description?: string
  language?: string
  favorite?: number
}

/** 应用设置。对应后端 settings.rs 的 AppSettings。 */
export interface AppSettings {
  /** 全局呼出快捷键，如 "Ctrl+Space"。null/空 = 未注册。 */
  toggle_shortcut: string | null
}

/**
 * 后端 AppError 序列化后的形状。
 * Rust 端用 `#[serde(tag = "type", content = "message")]`，
 * 因此 Tauri 抛到前端的错误对象长这样（被 invoke 包在 string 里）。
 */
export interface AppErrorPayload {
  type: string
  message: string
}

/** 从 invoke 抛出的未知错误里解析出 AppErrorPayload；失败时返回 null。 */
export function parseAppError(err: unknown): AppErrorPayload | null {
  // Tauri 把 Err 变体序列化成 JSON 字符串抛给前端。
  if (typeof err === 'string') {
    try {
      const parsed = JSON.parse(err)
      if (parsed && typeof parsed.type === 'string') return parsed
    } catch {
      return null
    }
  }
  if (err && typeof err === 'object' && 'type' in err && 'message' in err) {
    return err as AppErrorPayload
  }
  return null
}
