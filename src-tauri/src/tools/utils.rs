use std::env;
use std::path::{Path, PathBuf};

/// Get the config dir of Liz
/// If the environment variable "LIZ_DATA_DIR" set, use its value
/// Or use the default value: where is the /liz folder under the system-specific config dir:
/// - Windows: ~APPDATA/liz
/// - Linux: $HOME/.config/liz
/// - Mac: $HOME/Library/Application Support/liz (Not tested yet)
pub fn get_app_config_folder() -> PathBuf {
    match env::var("LIZ_DATA_DIR") {
        Ok(s) => Path::new(&s).to_path_buf(),
        Err(_e) => {
            let path: PathBuf = Path::new(&get_system_config_folder()).join("liz");
            eprintln!("Env variable LIZ_DATA_DIR not set, use default instead: {}", path.to_str().expect("Failed to convert path to str"));
            path
        },
    }
}

#[cfg(target_os = "linux")]
fn get_system_config_folder() -> String {
    // On Linux, we typically use ~/.config
    let home_dir = env::var("HOME").expect("Failed to get HOME directory");
    format!("{}/.config", home_dir)
}

#[cfg(target_os = "windows")]
fn get_system_config_folder() -> String {
    // On Windows, we typically use %APPDATA% (AppData\Roaming)
    env::var("APPDATA").expect("Failed to get APPDATA directory")
}

#[cfg(target_os = "macos")]
fn get_system_config_folder() -> String {
    // On macOS, it's typically ~/Library/Application Support
    let home_dir = env::var("HOME").expect("Failed to get HOME directory");
    format!("{}/Library/Application Support", home_dir)
}