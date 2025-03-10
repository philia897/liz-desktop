use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use super::utils::get_app_config_folder;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Rhythm {
    pub liz_path: String, // The config path from
    // pub user_sheets_path: String, // Path for all the shortcut sheets
    pub music_sheet_path: String, // Path for the lock file for Bluebird
    pub keymap_path: String,      // Can be used to customize key mapping
    pub interval_ms: u64,         // interval of each shortcut block. No need to set it normally.
    pub trigger_shortcut: String, // The shortcut to activate Liz
    pub shortcut_print_fmt: String, // The format to show one shortcut
    pub language: String,    // The Application Language
}

impl Default for Rhythm {
    fn default() -> Self {
        // Get the home directory and construct the rhythm path
        let liz_path: String = get_app_config_folder()
            .to_str()
            .expect("Failed to convert path to str")
            .to_string();
        // let user_sheets_path: String = format!("{}/sheets", liz_path);
        let music_sheet_path: String = format!("{}/music_sheet.lock", liz_path);
        let keymap_path: String = format!("");
        let trigger_shortcut: String = "Ctrl+Alt+L".to_string();
        let shortcut_print_fmt: String =
            "<b>#description</b> | #application | #shortcut".to_string();

        Self {
            liz_path,
            // user_sheets_path,
            music_sheet_path,
            keymap_path,
            interval_ms: 100,
            trigger_shortcut,
            shortcut_print_fmt,
            language: format!("en"),
        }
    }
}

/// Get the default rhythm config file path
fn get_rhythm_path() -> PathBuf {
    get_app_config_folder().join("rhythm.toml")
}

/// Function to parse JSON into Rhythm struct
pub fn parse_rhythm(json_str: &str) -> Result<Rhythm, Box<dyn std::error::Error>> {
    let rhythm: Rhythm = serde_json::from_str(json_str)?; // Deserialize JSON
    Ok(rhythm)
}

impl Rhythm {
    pub fn to_string_list(&self) -> Vec<String> {
        vec![
            json!({"name": "language", "value": self.language, "hint": "The Application Language (Support zh, en)"}).to_string(),
            json!({"name": "liz_path", "value": self.liz_path, "hint": "The path of data dir"}).to_string(),
            // json!({"name": "user_sheets_path", "value": self.user_sheets_path, "hint": "Path for all the shortcut sheets"}).to_string(),
            json!({"name": "music_sheet_path", "value": self.music_sheet_path, "hint": "Path for the lock file for Bluebird"}).to_string(),
            json!({"name": "keymap_path", "value": self.keymap_path, "hint": "Can be used to customize key mapping"}).to_string(),
            json!({"name": "interval_ms", "value": self.interval_ms, "hint": "Interval of each shortcut block. No need to set it normally."}).to_string(),
            json!({"name": "trigger_shortcut", "value": self.trigger_shortcut, "hint": "The shortcut to activate Liz"}).to_string(),
            json!({"name": "shortcut_print_fmt", "value": self.shortcut_print_fmt, "hint": "The format to show one shortcut"}).to_string(),
        ]
    }

    pub fn save_rhythm(&self, path: Option<PathBuf>) -> Result<String, Box<dyn std::error::Error>> {
        let rhythm_path: PathBuf = path.unwrap_or_else(get_rhythm_path);

        // Serialize the Rhythm struct into a TOML string
        let content = toml::to_string_pretty(self)?;

        // Create the parent directory if it doesn’t exist
        if let Some(parent) = rhythm_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let rhythm_path_str = rhythm_path.to_string_lossy().to_string();

        // Write the serialized data to the file
        // Open or create the file using OpenOptions
        let mut file = OpenOptions::new()
            .write(true) // Enable writing
            .create(true) // Create if it doesn't exist
            .truncate(true) // Clear existing content before writing
            .open(rhythm_path)?;
        file.write_all(content.as_bytes())?;

        println!("Saved Rhythm settings to {}.", rhythm_path_str);

        Ok(rhythm_path_str)
    }

    pub fn read_rhythm(
        rhythm_path_str: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let rhythm_path: PathBuf = match rhythm_path_str {
            Some(rhythm_path_str) => PathBuf::from(rhythm_path_str),
            None => get_rhythm_path(),
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
}
