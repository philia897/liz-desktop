use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use super::utils::get_app_config_folder;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Rhythm {
    pub liz_path: String,         // The config path from
    pub user_sheets_path: String, // Path for all the shortcut sheets
    pub music_sheet_path: String, // Path for the lock file for Bluebird
    pub keymap_path: String,
    pub persist_freq_s: u64, // The interval between two auto-persisting
    pub interval_ms: u64,    // interval of each shortcut block. No need to set it normally.
    pub trigger_shortcut: String,
    pub shortcut_print_fmt: String, // The format to show one shortcut
}

impl Default for Rhythm {
    fn default() -> Self {
        // Get the home directory and construct the rhythm path
        let liz_path: String = get_app_config_folder()
            .to_str()
            .expect("Failed to convert path to str")
            .to_string();
        let user_sheets_path: String = format!("{}/sheets", liz_path);
        let music_sheet_path: String = format!("{}/music_sheet.lock", liz_path);
        let keymap_path: String = format!("{}/keymap_builtin.json", liz_path);
        let trigger_shortcut: String = "Ctrl+Alt+L".to_string();
        let shortcut_print_fmt: String =
            "<b>#description</b> | #application | #shortcut".to_string();

        Self {
            liz_path,
            user_sheets_path,
            music_sheet_path,
            keymap_path,
            persist_freq_s: 3600,
            interval_ms: 100,
            trigger_shortcut,
            shortcut_print_fmt,
        }
    }
}

impl Rhythm {
    pub fn read_rhythm(
        rhythm_path_str: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let rhythm_path: PathBuf = match rhythm_path_str {
            Some(rhythm_path_str) => PathBuf::from(rhythm_path_str),
            None => get_app_config_folder().join("rhythm.toml"),
        };

        if !rhythm_path.exists() {
            eprintln!(
                "Warning: rhythm config file {} not found, using default values.",
                rhythm_path.display()
            );
            return Ok(Rhythm::default());
        }

        let content: String = fs::read_to_string(rhythm_path)?;
        let rhythm: Rhythm = toml::de::from_str(&content).unwrap_or_default();

        Ok(rhythm)
    }

    pub fn to_pretty_vec(&self) -> Vec<String> {
        vec![
            "liz_path".to_string(),
            self.liz_path.clone(),
            "user_sheets_path".to_string(),
            self.user_sheets_path.clone(),
            "music_sheet_path".to_string(),
            self.music_sheet_path.clone(),
            "keymap_path".to_string(),
            self.keymap_path.clone(),
            "persist_freq_s".to_string(),
            self.persist_freq_s.to_string(),
            "interval_ms".to_string(),
            self.interval_ms.to_string(),
            "trigger_shortcut".to_string(),
            self.trigger_shortcut.clone(),
            "shortcut_print_fmt".to_string(),
            self.shortcut_print_fmt.clone(),
        ]
    }
}
