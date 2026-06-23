// 应用设置：JSON 文件持久化。
//
// 存 app_data_dir/snipdock-settings.json。文件不存在视为默认值，不报错。
// 用 JSON 而非 DB：设置是单例数据，避免加 settings 表和迁移。
//
// 目前只有 toggle_shortcut（全局呼出快捷键）。未来加更多设置项时在此扩展。

use std::fs;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

/// 应用设置。所有字段可选，缺失时用默认值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 全局呼出快捷键，如 "Ctrl+Space"。None/空 = 未注册。
    /// 格式遵循 Tauri accelerator 语法（CommandOr/Ctrl/Alt/Shift + 键）。
    #[serde(default)]
    pub toggle_shortcut: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            toggle_shortcut: None,
        }
    }
}

/// 配置文件路径。
fn settings_path<R: Runtime>(app: &AppHandle<R>) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .expect("failed to resolve app data dir")
        .join("snipdock-settings.json")
}

/// 从磁盘加载；文件不存在或解析失败返回默认值（不报错，保证启动不崩）。
pub fn load<R: Runtime>(app: &AppHandle<R>) -> AppSettings {
    let path = settings_path(app);
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

/// 写入磁盘（序列化失败也返回 Ok，设置变更不应阻断 UI）。
pub fn save<R: Runtime>(app: &AppHandle<R>, settings: &AppSettings) {
    let path = settings_path(app);
    if let Ok(text) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, text);
    }
}
