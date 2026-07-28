//! 配置模块 - 用户配置读写（主题偏好、窗口位置、开机自启等）
//! 配置文件存储于 %APPDATA%/LiteNote/config.json

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// 应用配置结构（与前端 TypeScript 的 Config 类型对应）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    /// 主题: "light" | "dark"
    #[serde(default = "default_theme")]
    pub theme: String,
    /// 是否开机自启
    #[serde(default)]
    pub auto_start: bool,
    /// 侧边栏当前选中: "all" | "favorite" | "trash"
    #[serde(default = "default_sidebar")]
    pub sidebar_active: String,
    /// 最后打开的笔记 id
    #[serde(default)]
    pub last_opened_note_id: Option<i64>,
    /// 窗口宽度
    #[serde(default = "default_width")]
    pub window_width: f64,
    /// 窗口高度
    #[serde(default = "default_height")]
    pub window_height: f64,
    /// 窗口不透明度 (0.3 ~ 1.0)
    #[serde(default = "default_opacity")]
    pub window_opacity: f64,
}

fn default_theme() -> String {
    "light".to_string()
}
fn default_sidebar() -> String {
    "all".to_string()
}
fn default_width() -> f64 {
    1100.0
}
fn default_height() -> f64 {
    720.0
}
fn default_opacity() -> f64 {
    0.9
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            auto_start: false,
            sidebar_active: default_sidebar(),
            last_opened_note_id: None,
            window_width: default_width(),
            window_height: default_height(),
            window_opacity: default_opacity(),
        }
    }
}

impl AppConfig {
    /// 从文件加载配置，失败则返回默认配置
    pub fn load(path: &PathBuf) -> Self {
        match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// 保存配置到文件
    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, text).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// 获取配置文件路径 %APPDATA%/LiteNote/config.json
pub fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    Ok(dir.join("config.json"))
}

/// 获取数据库文件路径 %APPDATA%/LiteNote/data.db
pub fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("data.db"))
}
