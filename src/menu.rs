use serde::{Deserialize, Serialize};
use std::path::Path;

/// The mutable fzf state that the menu saves on open and restores on close.
#[derive(Debug, Serialize, Deserialize)]
pub struct MenuState {
    pub key_bindings: String,
    pub prompt: String,
    pub query: String,
    pub preview: String,
    /// The fully-assembled jq preview command, with the query embedded (i.e. frozen).
    pub frozen_preview: String,
    pub preview_window: String,
    pub transform_bin: String,
    pub reset_key: String,
    pub menu_height: usize,
}

impl MenuState {
    pub fn save(&self, path: &Path) -> Result<(), crate::Error> {
        let contents = serde_json::to_string(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, crate::Error> {
        let contents = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    }
}

/// Build the fzf action string for opening the menu
pub fn open_actions(state: &MenuState, menu_path: &Path) -> Result<String, Error> {
    let menu_path = crate::bash_quote(menu_path);
    Ok([
        "change-prompt([menu]> )".to_string(),
        format!("unbind({})", state.key_bindings),
        format!("change-preview({})", state.frozen_preview),
        format!(
            "change-preview-window(up,{},border-bottom)",
            state.menu_height
        ),
        format!("reload(cat {menu_path})"),
        "change-query()".to_string(),
        "enable-search".to_string(),
    ]
    .join("+"))
}

/// Build the fzf action string for closing the menu.
///
/// Restores the prompt, preview command, preview window layout, list contents, and query.
#[must_use]
pub fn close_actions(state: &MenuState) -> String {
    [
        format!("rebind({})", state.key_bindings),
        "reload()".to_string(),
        "disable-search".to_string(),
        format!("change-prompt({})", state.prompt),
        format!("change-query({})", state.query),
        format!("change-preview({})", state.preview),
        format!("change-preview-window({})", state.preview_window),
        format!("trigger({})", state.reset_key),
    ]
    .join("+")
}

pub fn accept_actions(state: &MenuState, action: &str) -> Result<String, Error> {
    let keybinding = action;

    Ok([
        "reload()".to_string(),
        "disable-search".to_string(),
        format!("change-preview-window({})", state.preview_window),
        format!("rebind({})", state.key_bindings),
        format!("change-prompt({})", state.prompt),
        format!("change-query({})", state.query),
        format!("trigger({keybinding})"),
    ]
    .join("+"))
}

pub fn calculate_menu_height(target_menu_lines: usize) -> Result<usize, Error> {
    let fzf_lines = std::env::var("FZF_LINES")?.parse::<usize>()?;
    let fzf_preview_lines = std::env::var("FZF_PREVIEW_LINES")?.parse::<usize>()?;
    let menu_height = fzf_lines - (fzf_lines - fzf_preview_lines) + 2 - target_menu_lines;
    Ok(menu_height)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    VarError(#[from] std::env::VarError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("TODO: {0}")]
    Todo(&'static str),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}
