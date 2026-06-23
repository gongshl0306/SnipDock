// 设置相关的 Tauri commands。
//
// get_settings：返回当前设置（内存中的单例）。
// set_toggle_shortcut：重新注册全局呼出快捷键。
//   1. 注销当前已注册的（若有）
//   2. 空 accelerator → 仅清除（不注册新键），写回配置
//   3. 非空 → 注册（格式错误 / 已被系统占用会抛错误给前端）
//   4. 写回配置文件（持久化）
//
// 错误以 AppError::Shortcut(message) 返回中文信息给前端展示。

use std::sync::Mutex;

use tauri::State;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::error::AppError;
use crate::settings::{self, AppSettings};
use crate::tray;

/// 返回当前设置（从内存单例克隆）。
#[tauri::command]
pub fn get_settings(
    state: State<'_, Mutex<AppSettings>>,
) -> Result<AppSettings, AppError> {
    Ok(state.lock().expect("settings mutex poisoned").clone())
}

/**
 * 设置全局呼出快捷键。
 *
 * - accelerator 非空：先注销旧的，再注册新的。注册失败（格式错误 /
 *   被其他程序占用）返回 AppError，旧快捷键状态不变。
 * - accelerator 空（""）：注销当前的，不注册新的（=关闭快捷键）。
 *
 * 成功后写回配置文件。
 */
#[tauri::command]
pub fn set_toggle_shortcut(
    accelerator: String,
    app: tauri::AppHandle,
    state: State<'_, Mutex<AppSettings>>,
) -> Result<(), AppError> {
    let new_accel = accelerator.trim();
    let gs = app.global_shortcut();

    // 先尝试注销当前已注册的（如果有）。
    {
        let current = state.lock().expect("settings mutex poisoned");
        if let Some(old) = &current.toggle_shortcut {
            // unregister 接收 Shortcut；解析失败说明存的值非法，忽略即可。
            if let Ok(old_sc) = old.parse::<tauri_plugin_global_shortcut::Shortcut>() {
                let _ = gs.unregister(old_sc);
            }
        }
    }

    // 空字符串 = 清除快捷键。
    let next_value: Option<String> = if new_accel.is_empty() {
        None
    } else {
        // 注册新快捷键；格式错误会被 Tauri 抛出，转成中文错误。
        // on_shortcut 接收能解析为 Shortcut 的类型；先解析 &str 得到 Shortcut。
        let shortcut: tauri_plugin_global_shortcut::Shortcut = new_accel
            .try_into()
            .map_err(|_| AppError::Shortcut(format!("快捷键格式无效：{new_accel}")))?;
        gs.on_shortcut(shortcut, move |app, _s, _e| {
            tray::toggle_main_window(app);
        })
        .map_err(|e| AppError::Shortcut(format!("注册快捷键失败（可能已被占用）：{e}")))?;
        Some(new_accel.to_string())
    };
    // 更新内存 + 写盘。
    let mut current = state.lock().expect("settings mutex poisoned");
    current.toggle_shortcut = next_value.clone();
    settings::save(&app, &current);
    Ok(())
}
