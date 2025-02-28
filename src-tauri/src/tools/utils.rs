use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Get the default config dir of Liz
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


pub fn generate_id() -> u128 {
    // Generate a new UUID (Version 4 for random UUID)
    let uuid = Uuid::new_v4();
    let id: u128 = uuid.as_u128();
    id
}

// Convert String to u128 id
pub fn string_to_id(s: &str) -> Result<u128, Box<dyn Error>> {
    let id: u128 = match Uuid::parse_str(s) {
        Ok(uuid) => {
            uuid.as_u128()
        },
        Err(e) => {
            eprintln!("Failed to parse string using UUID lib, error: {}\nTry to use str::parse instead...", e);
            s.parse::<u128>()?
        },
    } ;
    
    Ok(id)
}

// Convert u128 id to String
pub fn id_to_string(n: u128) -> String {
    let uuid = Uuid::from_u128(n);
    uuid.to_string()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_string_converting() {
        let id: u128 = generate_id();
        
        let id_str = id_to_string(id);
        println!("id_to_string: {} -> {}", id, id_str);

        let id2 = string_to_id(&id_str).unwrap();
        println!("string_to_id: {} -> {}", id_str, id2);

        assert_eq!(id, id2);

        let id_str = id.to_string();
        println!("id_to_string 2: {} -> {}", id, id_str);

        let id2 = id_str.parse::<u128>().unwrap();
        println!("string_to_id 2: {} -> {}", id_str, id2);

        assert_eq!(id, id2);

        let id2 = string_to_id(&id_str).unwrap_or(0);
        println!("string_to_id 3: {} -> {}", id_str, id2);

        assert_eq!(id, id2)

    }

}