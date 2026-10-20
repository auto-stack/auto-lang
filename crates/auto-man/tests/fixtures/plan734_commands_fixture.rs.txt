use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Echo {
    pub ok: bool,
    pub n: i32,
    pub tag: String,
}

// Tauri Commands

#[tauri::command]
pub fn echo(n: i32, tag: String) -> Echo {
    api::echo(n, tag)
}

// Command Registration
use tauri::Manager;

/// Register all API commands with the Tauri app
pub fn register_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        echo,
    ])
}
