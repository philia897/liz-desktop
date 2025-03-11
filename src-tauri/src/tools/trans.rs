use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// #[derive(Debug)]
pub struct TranslationCache {
    pub data: HashMap<String, String>,
}

impl TranslationCache {
    pub fn load(lang: &str, resource_dir: &PathBuf) -> Self {
        Self { data: load_translations(lang, resource_dir) }
    }

    pub fn reload(&mut self, lang: &str, resource_dir: &PathBuf) {
        let data = load_translations(lang, resource_dir);
        self.data.extend(data);
    }
}

/// Load translations, starting with English as the base, then overwriting with the given language.
fn load_translations(lang: &str, resource_dir: &PathBuf) -> HashMap<String, String> {
    let mut translations: HashMap<String, String> = HashMap::new();

    // Load English first as a base
    if let Ok(content) = fs::read_to_string(resource_dir.join("locales/en.json")) {
        if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
            translations.extend(map);
        }
    }

    // Load the target language and overwrite existing values
    if lang != "en" {
        let file_path = format!("locales/{}.json", lang);
        if let Ok(content) = fs::read_to_string(resource_dir.join(file_path)) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                translations.extend(map);
            }
        }
    }

    translations
}
