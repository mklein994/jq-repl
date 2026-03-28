use serde::{Deserialize, Serialize};
use std::path::Path;

/// The mutable fzf state that the menu saves on open and restores on close.
#[derive(Debug, Serialize, Deserialize)]
pub struct MenuState {
    pub prompt: String,
    pub query: String,
    /// The fully-assembled jq preview command, with the query embedded (i.e. frozen).
    pub preview: String,
    pub preview_window: String,
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

/// Build the fzf action string for opening the menu.
///
/// - Saves state to `state_path` (already written by the caller before this is called).
/// - Loads menu items from `menu_path` via `reload(cat ...)`.
/// - Clears the query so the user can type to filter the menu.
/// - Freezes the preview by embedding the current query into the command.
/// - Changes the prompt to `[menu]> `.
#[must_use]
pub fn open_actions(state: &MenuState, menu_path: &Path) -> String {
    let menu_path = crate::bash_quote(menu_path);
    [
        "change-prompt([menu]> )".to_string(),
        format!("change-preview({})", state.preview),
        format!("change-preview-window({})", state.preview_window),
        format!("reload(cat {menu_path})"),
        "change-query()".to_string(),
    ]
    .join("+")
}

/// Build the fzf action string for closing the menu.
///
/// Restores the prompt, preview command, preview window layout, list contents, and query.
#[must_use]
pub fn close_actions(state: &MenuState) -> String {
    [
        format!("change-prompt({})", state.prompt),
        format!("change-preview({})", state.preview),
        format!("change-preview-window({})", state.preview_window),
        "reload()".to_string(),
        format!("change-query({})", state.query),
    ]
    .join("+")
}
