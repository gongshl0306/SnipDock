// 设置相关 Tauri 命令的薄封装。
//
// get_settings：返回当前 AppSettings（含 toggle_shortcut）。
// set_toggle_shortcut：重新注册全局呼出快捷键。空串 = 清除。
// 命令参数名必须与 Rust 端一致（accelerator）。

import { invoke } from '@tauri-apps/api/core'
import type { AppSettings } from '@/types/models'

export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('get_settings')
}

export function setToggleShortcut(accelerator: string): Promise<void> {
  return invoke<void>('set_toggle_shortcut', { accelerator })
}
