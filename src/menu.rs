use crate::Config;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tabwriter::TabWriter;

/// The mutable fzf state that the menu saves on open and restores on close.
#[derive(Debug, Serialize, Deserialize)]
pub struct MenuState {
    pub key_bindings: String,
    pub prompt: String,
    pub query: String,
    pub preview: String,
    /// The fully-assembled jq preview command, with the query embedded (i.e. frozen).
    pub frozen_preview: String,
    pub transform_bin: String,
    pub reset_key: String,
    pub menu_height: usize,
}

impl MenuState {
    pub fn save(&self, path: &Path) -> Result<(), crate::Error> {
        info!("saving menu state");
        let contents = serde_json::to_string(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, crate::Error> {
        info!("loading menu state");
        let contents = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    }
}

pub fn write_menu_contents(path: &Path, config: &Config) -> Result<(), Error> {
    info!("writing menu contents");
    let menu = config
        .lens
        .iter()
        .map(|(key, lens)| {
            (
                key.as_str(),
                "lens",
                lens.key.as_str(),
                lens.command.as_str(),
            )
        })
        .chain(config.external.iter().map(|(key, external)| {
            (
                key.as_str(),
                "external",
                external.key.as_str(),
                external.command.as_str(),
            )
        }))
        .map(|x| <[_; _]>::from(x).join("\t"));

    let keys = config
        .lens
        .values()
        .map(|value| format!("lens:{}", value.key))
        .chain(
            config
                .external
                .values()
                .map(|value| format!("external:{}", value.key)),
        )
        .collect::<Vec<_>>();

    let mut bytes = vec![];
    let mut writer = TabWriter::new(&mut bytes);
    write!(&mut writer, "{}", menu.collect::<Vec<_>>().join("\n"))?;
    writer.flush()?;

    let mut file = File::create(path)?;
    for (key, line) in keys.iter().zip(String::from_utf8(bytes)?.lines()) {
        writeln!(&mut file, "{key}\t{line}")?;
    }

    Ok(())
}

/// Build the fzf action string for opening the menu
pub fn open_actions(state: &MenuState, menu_path: &Path) -> Result<String, Error> {
    info!("open_actions");
    let menu_path = crate::bash_quote(menu_path);
    Ok([
        "change-prompt([menu]> )".to_string(),
        format!("unbind({})", state.key_bindings),
        format!("change-preview({})", state.frozen_preview),
        format!("change-preview-window({})", state.menu_height),
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
    info!("close_actions");

    [
        format!("rebind({})", state.key_bindings),
        "reload()".to_string(),
        "disable-search".to_string(),
        format!("change-prompt({})", state.prompt),
        format!("change-query({})", state.query),
        format!("change-preview({})", state.preview),
        "change-preview-window()".to_string(),
        format!("trigger({})", state.reset_key),
    ]
    .join("+")
}

pub fn accept_actions(state: &MenuState, action: &str) -> Result<String, Error> {
    info!("accept_actions: {action:?}");

    let (kind, keybinding) = action
        .split_once(':')
        .ok_or_else(|| Error::UnknownActionKind(action.to_string()))?;
    let base_prompt = match kind {
        "lens" => Ok(state.prompt.clone()),
        "external" => {
            let mut prompt: crate::Prompt = state.prompt.parse().unwrap();
            prompt.transform(None, Some(None));
            Ok(prompt.to_string())
        }
        _ => Err(Error::UnknownActionKind(action.to_string())),
    }?;

    Ok([
        "reload()".to_string(),
        "disable-search".to_string(),
        "change-preview-window()".to_string(),
        format!("rebind({})", state.key_bindings),
        format!("change-prompt({base_prompt})"),
        format!("change-query({})", state.query),
        format!("trigger({keybinding})"),
        if kind == "external" {
            format!("trigger({})", state.reset_key)
        } else {
            String::new()
        },
    ]
    .into_iter()
    .filter(|x| !x.is_empty())
    .collect::<Vec<_>>()
    .join("+"))
}

pub fn calculate_menu_height(target_menu_lines: usize) -> Result<usize, Error> {
    let fzf_lines = std::env::var("FZF_LINES")?.parse::<usize>()?;
    let fzf_preview_lines = std::env::var("FZF_PREVIEW_LINES")?.parse::<usize>()?;
    let minimum_list_area_padding = 2; // This is apparently hard-coded by fzf

    let menu_height =
        fzf_lines - (fzf_lines - fzf_preview_lines) + minimum_list_area_padding - target_menu_lines;
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

    #[error("unknown action: {0:?}")]
    UnknownActionKind(String),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a representative [`MenuState`] for use in tests.
    ///
    /// `prompt` is the prompt that was active when the menu was opened — it reflects whatever lens
    /// (if any) was active at that point.
    fn make_state(prompt: &str) -> MenuState {
        MenuState {
            key_bindings: "alt-g,ctrl-space,alt-G,alt-h,alt-c,alt-C,tab,alt-L,alt-e,alt-E,alt-l,\
                           alt-v,alt-j,alt-J"
                .to_string(),
            prompt: prompt.to_string(),
            query: ".foo".to_string(),
            preview: "gojq --raw-output -n -C {q} './tests/foo bar.json'".to_string(),
            frozen_preview: "gojq --raw-output -n -C '.foo' './tests/foo bar.json'".to_string(),
            transform_bin: "_jq-repl-transform".to_string(),
            reset_key: "alt-G".to_string(),
            menu_height: 10,
        }
    }

    // --- close_actions ---

    /// Cancelling the menu (esc/ctrl-g) should restore the prompt exactly as it was when the menu
    /// was opened, then trigger the reset-lens key so the preview is consistent with the prompt.
    #[test]
    fn close_restores_prompt_when_no_lens_was_active() {
        let state = make_state("-n> ");
        let actions = close_actions(&state);
        assert!(
            actions.contains("change-prompt(-n> )"),
            "expected base prompt to be restored; got: {actions}"
        );
    }

    #[test]
    fn close_restores_prompt_when_lens_was_active() {
        let state = make_state("-n gron> ");
        let actions = close_actions(&state);
        assert!(
            actions.contains("change-prompt(-n gron> )"),
            "expected lens prompt to be restored; got: {actions}"
        );
    }

    #[test]
    fn close_triggers_reset_key_to_sync_preview() {
        let state = make_state("-n> ");
        let actions = close_actions(&state);
        assert!(
            actions.contains("trigger(alt-G)"),
            "expected reset_key trigger to sync preview; got: {actions}"
        );
    }

    // --- accept_actions: lens ---

    /// Accepting a lens from the menu should restore the base prompt temporarily, then let the
    /// triggered lens binding (via _jq-repl-transform) set the final prompt. The base prompt
    /// restoration step means the transform sees a clean slate.
    #[test]
    fn accept_lens_triggers_the_lens_key() {
        let state = make_state("-n> ");
        let actions = accept_actions(&state, "lens:ctrl-space").unwrap();
        assert!(
            actions.contains("trigger(ctrl-space)"),
            "expected the lens keybinding to be triggered; got: {actions}"
        );
    }

    #[test]
    fn accept_lens_restores_base_prompt_before_trigger() {
        let state = make_state("-n> ");
        let actions = accept_actions(&state, "lens:ctrl-space").unwrap();
        assert!(
            actions.contains("change-prompt(-n> )"),
            "expected base prompt before lens trigger; got: {actions}"
        );
    }

    // --- accept_actions: external ---

    /// This was the bug: when a lens was active when the menu was opened, accepting an *external*
    /// tool (e.g. bat) would restore `state.prompt` (e.g. `-n gron> `). But the external's
    /// execute binding doesn't go through _jq-repl-transform, so nothing corrected the prompt
    /// afterward. The prompt ended up showing the lens as active even though it wasn't.
    ///
    /// The fix: accepting an external strips the lens program from the saved prompt, so the prompt
    /// correctly reflects that no lens is active after the external closes.
    #[test]
    fn accept_external_restores_base_prompt_not_lens_prompt() {
        // Simulate: gron lens was active when the menu was opened for the second time.
        let state = make_state("-n gron> ");
        let actions = accept_actions(&state, "external:alt-L").unwrap();

        assert!(
            actions.contains("change-prompt(-n> )"),
            "expected base prompt (no lens) when accepting an external; got: {actions}"
        );
        assert!(
            !actions.contains("change-prompt(-n gron> )"),
            "should not restore stale lens prompt when accepting an external; got: {actions}"
        );
    }

    #[test]
    fn accept_external_triggers_the_external_key() {
        let state = make_state("-n> ");
        let actions = accept_actions(&state, "external:alt-L").unwrap();
        assert!(
            actions.contains("trigger(alt-L)"),
            "expected the external keybinding to be triggered; got: {actions}"
        );
    }
}
