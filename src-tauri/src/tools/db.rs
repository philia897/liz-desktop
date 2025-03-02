use serde::de;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::Read;

use super::utils::{generate_id, id_to_string, string_to_id};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Shortcut {
    #[serde(serialize_with = "serialize_id", deserialize_with = "deserialize_id")]
    pub id: u128, // UUID

    pub hit_number: i64,     // How many time the shortcut is hit
    pub shortcut: String,    // Shortcut string
    pub application: String, // Application using this shortcut
    pub description: String, // Shortcut description, shall not be too long
    pub comment: String,     // Extra info or explanation for the shortcut
}

fn serialize_id<S>(id: &u128, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Serialize `id` as a string
    serializer.serialize_str(&id_to_string(*id))
}

fn deserialize_id<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Deserialize `id` from a string
    let s: String = String::deserialize(deserializer)?;

    string_to_id(&s).map_err(de::Error::custom)
}

impl Shortcut {
    // Example method to update shortcut values
    pub fn update(&mut self, new_sc: &Shortcut) {
        self.hit_number = new_sc.hit_number;
        self.shortcut = new_sc.shortcut.clone();
        self.application = new_sc.application.clone();
        self.description = new_sc.description.clone();
        self.comment = new_sc.comment.clone();
    }
}

impl Default for Shortcut {
    fn default() -> Self {
        Self {
            id: generate_id(),
            hit_number: 0,
            shortcut: "".to_string(),
            application: "None".to_string(),
            description: "None".to_string(),
            comment: "".to_string(),
        }
    }
}

impl Shortcut {
    pub fn to_json_string(&self) -> String {
        let json_string = serde_json::to_string(self).unwrap();
        json_string
    }

    pub fn from_json_string(json_str: &str) -> Result<Self, Box<dyn Error>> {
        let shortcut: Shortcut = serde_json::from_str(json_str)?;
        Ok(shortcut)
    }

    pub fn format_output(&self, fmt: &str) -> String {
        let mut formatted_str: String = fmt.to_string();

        // Replace all possible attributes
        formatted_str = formatted_str.replace("#id", &self.id.to_string());
        formatted_str = formatted_str.replace("#hit_number", &self.hit_number.to_string());
        formatted_str = formatted_str.replace("#shortcut", &self.shortcut);
        formatted_str = formatted_str.replace("#application", &self.application);
        formatted_str = formatted_str.replace("#description", &self.description);
        formatted_str = formatted_str.replace("#comment", &self.comment);

        formatted_str
    }

    /// Parse the shortcut string to keycodes that can be used for execution
    pub fn parse_to_keycode(
        &self,
        key_event_codes: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        let rst: String = convert_shortcut_to_key_presses(&self.shortcut, key_event_codes);
        Ok(rst)
    }

    /// Remove duplicates by considering all attributes except hit_number, or the id is the same
    pub fn remove_duplicates(shortcuts: &Vec<Shortcut>) -> Vec<Shortcut> {
        let mut seen = HashSet::new();
        let mut seen_id = HashSet::new();
        let mut unique_shortcuts = Vec::new();

        for shortcut in shortcuts {
            // Create a tuple of all fields that should be used for uniqueness check
            let unique_key = (
                shortcut.shortcut.clone(),
                shortcut.application.clone(),
                shortcut.description.clone(),
                shortcut.comment.clone(),
            );

            // If the unique key AND id is not already in the set, add it to the result
            if !seen.contains(&unique_key) && !seen_id.contains(&shortcut.id) {
                unique_shortcuts.push(shortcut.clone());
                seen_id.insert(shortcut.id);
                seen.insert(unique_key);
            }
        }

        unique_shortcuts
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MusicSheetDBTable {
    deleted: Vec<Shortcut>,
    data: Vec<Shortcut>,
}

impl MusicSheetDBTable {
    pub fn new() -> Self {
        Self {
            deleted: Vec::new(),
            data: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct MusicSheetDB {
    t: MusicSheetDBTable,
    pub keymap: HashMap<String, String>,
}

impl MusicSheetDB {
    /**
     * Add a list of Shortcuts into the data.
     * safe_check (default true) to remove duplicate shortcuts, which means the content or the id is the same.
     */
    pub fn add_shortcuts(&mut self, shortcuts: Vec<Shortcut>, safe_check: Option<bool>) {
        self.t.data.extend(shortcuts);
        let safe_check = safe_check.unwrap_or(true);
        if safe_check {
            self.remove_data_duplicates();
        }
    }

    // Remove duplicates in shortcuts by considering all attributes except hit_number, or the id is the same
    pub fn remove_data_duplicates(&mut self) {
        self.t.data = Shortcut::remove_duplicates(&self.t.data);
    }

    /// Retrieves one shortcut by its id from either "data" or "deleted" list, default mode to be "data"
    pub fn retrieve(&self, id: u128, mode: Option<&str>) -> Option<&Shortcut> {
        let mode = mode.unwrap_or("data"); // Default to "data" if mode is None
        match mode {
            "data" => self.t.data.iter().find(|&shortcut| shortcut.id == id),
            "deleted" => self.t.deleted.iter().find(|&shortcut| shortcut.id == id),
            _ => None, // Return None if an invalid mode is provided
        }
    }

    /// Retrieve all data
    pub fn retrieve_all(&self) -> &Vec<Shortcut> {
        &self.t.data
    }

    /// Delete a list of shortcuts by id, and move the deleted shortcuts to deleted
    pub fn delete_shortcuts(&mut self, ids: Vec<u128>) {
        let mut deleted_shortcuts: Vec<Shortcut> = Vec::new();

        // Collect the shortcuts with the specified IDs to move them to deleted
        self.t.data.retain(|shortcut| {
            if ids.contains(&shortcut.id) {
                deleted_shortcuts.push(shortcut.clone());
                false
            } else {
                true
            }
        });

        self.t.deleted.extend(deleted_shortcuts);
    }

    /// Get the shortcuts that were deleted
    pub fn retrieve_deleted(&self) -> &Vec<Shortcut> {
        &self.t.deleted
    }

    pub fn clear_deleted(&mut self) {
        self.t.deleted.clear();
    }

    /// If the id is not found in data, then return false and do nothing.
    pub fn update_shortcuts(&mut self, new_shortcuts: Vec<Shortcut>) -> Vec<Shortcut> {
        let mut unmatched: Vec<Shortcut> = Vec::new();
        let mut modified_shortcuts: Vec<Shortcut> = Vec::new();

        for new_sc in new_shortcuts {
            if let Some(shortcut) = self.t.data.iter_mut().find(|s| s.id == new_sc.id) {
                modified_shortcuts.push(shortcut.clone());
                // *shortcut = new_sc;  // replace the entire object
                shortcut.update(&new_sc);
            } else {
                unmatched.push(new_sc);
            }
        }
        self.t.deleted.extend(modified_shortcuts);

        unmatched
    }
}

impl MusicSheetDB {
    /// Initialize an empty table
    pub fn new() -> Self {
        Self {
            t: MusicSheetDBTable::new(),
            keymap: HashMap::new(),
        }
    }

    /// Import from JSON file
    pub fn import_from_json(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let t: MusicSheetDBTable = serde_json::from_reader(file)?;
        Ok(Self {
            t,
            keymap: HashMap::new(),
        })
    }

    /// Export to JSON file
    pub fn export_to_json(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let _ = std::fs::remove_file(file_path);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)?;
        serde_json::to_writer(file, &self.t)?;
        Ok(())
    }

    pub fn read_keymap(&self, keymap_path: &str) {
        // Attempt to open the file
        let mut file = match File::open(keymap_path) {
            Ok(f) => f,
            Err(e) => {
                eprint!("Warning: Keymap file does not exist: {}\n", e);
                return; // Return an empty map in case of error
            }
        };

        // Read the contents of the file
        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents) {
            eprint!("Error reading keymap file: {}\n", e);
            return; // Return an empty map in case of error
        }

        // Parse the contents as JSON
        match serde_json::from_str(&contents) {
            Ok(key_event_codes) => key_event_codes,
            Err(e) => {
                eprint!("Error parsing keymap JSON: {}\n", e);
            }
        }
    }

    /// Function to increase hit_number for a given row index
    pub fn hit_num_up(&mut self, id: u128) -> Result<(), String> {
        if let Some(sc) = self.t.data.iter_mut().find(|shortcut| shortcut.id == id) {
            sc.hit_number += 1; // Increment the hit_number
            Ok(())
        } else {
            Err(format!("ID {} not found", id)) // Return an error if the index is invalid
        }
    }

    /// Sort by a specific column name, support: id, hit_number, application, description
    pub fn sort_by_column(&mut self, column: &str, ascending: bool) {
        // Decide on the comparison function based on the column, done once
        let comparator: Box<dyn Fn(&Shortcut, &Shortcut) -> std::cmp::Ordering> = match column {
            "id" => Box::new(|a, b| a.id.cmp(&b.id)),
            "hit_number" => Box::new(|a, b| a.hit_number.cmp(&b.hit_number)),
            "application" => Box::new(|a, b| a.application.cmp(&b.application)),
            "description" => Box::new(|a, b| a.description.cmp(&b.description)),
            _ => Box::new(|_, _| std::cmp::Ordering::Equal), // Handle unknown column names
        };

        // Now sort the data using the pre-selected comparator
        self.t.data.sort_by(|a, b| {
            let ordering = comparator(a, b);
            if ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSheet {
    data: Vec<Shortcut>,
}

impl UserSheet {
    // Initialize an empty table
    pub fn new(shortcuts: Vec<Shortcut>) -> Self {
        Self { data: shortcuts }
    }

    pub fn import_from(path: &str) -> Result<Self, Box<dyn Error>> {
        let metadata = fs::metadata(path)?;

        if metadata.is_file() {
            UserSheet::import_from_json(path)
        } else if metadata.is_dir() {
            UserSheet::import_from_json_dir(path)
        } else {
            Err(format!("{} is neither a file nor a directory.", path).into())
        }
    }

    /// Import from JSON file
    fn import_from_json(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let data: Vec<Shortcut> = serde_json::from_reader(file)?;
        Ok(Self { data: data })
    }

    /// Import all JSON files from a directory
    fn import_from_json_dir(dir_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut all_data: Vec<Shortcut> = Vec::new();

        // Iterate over all entries in the directory
        for entry in fs::read_dir(dir_path)? {
            let entry: fs::DirEntry = entry?;
            let path: std::path::PathBuf = entry.path();

            // Check if the entry is a file and ends with .json
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file = File::open(&path)?;

                // Deserialize the JSON content into UserDataRow
                let data: Vec<Shortcut> = serde_json::from_reader(file)?;

                // Extend the result vector with the new data
                all_data.extend(data);
            }
        }

        Ok(Self { data: all_data })
    }

    /// Export to JSON file
    pub fn export_to_json(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let _ = std::fs::remove_file(file_path);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)?;
        serde_json::to_writer(file, &self.data)?;
        Ok(())
    }

    pub fn transform_to_db(&self, db: &mut MusicSheetDB) {
        db.add_shortcuts(self.data.clone(), None);
    }
}

/**
 * Convert shortcut string to key presses, using the keymap to map key to keycode
 * For example:
 * meta+pageup tab 123!@# tab ABC  
 * => 126.1 104.1 104.0 126.0 15.1 15.0 [STR]+ 123!@#[STR] 15.1 15.0 [STR]+ ABC[STR]
 * Where keycode of meta is 126, pageup (104), tab (15)
 * type 123!@ means directly type these characters "123!@".
 * Note: "ctrl + c" will be consider press "ctrl", then "+" then "c", as they are splited by space.
 */
fn convert_shortcut_to_key_presses(
    shortcut: &str,
    key_event_codes: &HashMap<String, String>,
) -> String {
    let mut result = Vec::new();

    // Split by marker [STR] to different blocks
    let ss: Vec<&str> = shortcut.split("[STR]").collect();

    for s in ss {
        if s.is_empty() {
            continue;
        }
        if s.starts_with("+") {
            // Typing the string
            let type_str: &str = &s[2..];
            result.push(format!("[STR]+ {}[STR]", type_str.trim()));
        } else {
            // Split the input by spaces
            let parts: Vec<&str> = s.split_whitespace().collect();

            for part in parts {
                if part.is_empty() {
                    continue;
                }
                if part.contains('+') && part != "+" {
                    // Execute shortcut like ctrl+c, ctrl+v
                    let keys: Vec<&str> = part.split('+').collect();
                    for key in &keys {
                        // Press
                        let key: String = key.trim().to_lowercase();
                        if let Some(event_code) = key_event_codes.get(&key) {
                            result.push(format!("{}.1", event_code));
                        } else {
                            result.push(format!("{}.1", key));
                        }
                    }
                    for key in keys.iter().rev() {
                        // Release
                        let key: String = key.trim().to_lowercase();
                        if let Some(event_code) = key_event_codes.get(&key) {
                            result.push(format!("{}.0", event_code));
                        } else {
                            result.push(format!("{}.0", key));
                        }
                    }
                } else {
                    // Not a shortcut, either one single key or a string to type
                    let key = part.trim().to_lowercase();
                    if let Some(event_code) = key_event_codes.get(&key) {
                        // Press one key
                        result.push(format!("{}.1", event_code));
                        result.push(format!("{}.0", event_code));
                    } else if key.len() == 1 {
                        // Press one character
                        let k = part.trim();
                        result.push(format!("{}.1", k));
                        result.push(format!("{}.0", k));
                    } else {
                        //  Type the string
                        result.push(format!("[STR]+ {}[STR]", part.trim()));
                    }
                }
            }
        }
    }

    result.join(" ")
}

//  TEST

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_shortcut_to_key_presses() {
        let mut key_event_codes = HashMap::new();
        key_event_codes.insert("meta".to_string(), "126".to_string());
        key_event_codes.insert("pageup".to_string(), "104".to_string());
        key_event_codes.insert("tab".to_string(), "15".to_string());

        // Test 1: Basic conversion with keys mapped to keycodes
        let shortcut = "Meta+S Tab";
        let expected = Some("126.1 s.1 s.0 126.0 15.1 15.0".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 2: Test with characters (e.g., numbers or symbols)
        let shortcut = "123!@# tab ABC";
        let expected = Some("[STR]+ 123!@#[STR] 15.1 15.0 [STR]+ ABC[STR]".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 3: Test with more complex shortcuts (e.g., multiple key combinations)
        let shortcut = "meta+pageup tab 123!@# meta+tab";
        let expected = Some(
            "126.1 104.1 104.0 126.0 15.1 15.0 [STR]+ 123!@#[STR] 126.1 15.1 15.0 126.0"
                .to_string(),
        );
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 4: Test with unrecognized keys (e.g., no mapping for 'enter')
        let shortcut = "enter tab";
        let expected = Some("[STR]+ enter[STR] 15.1 15.0".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 5: Test with additional '+' combinations
        let shortcut = "meta+tab+pageup";
        let expected = Some("126.1 15.1 104.1 104.0 15.0 126.0".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 6: Test empty input
        let shortcut = "";
        let expected = Some("".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 7: Test plus with space
        let shortcut = "a + b + c";
        let expected = Some("a.1 a.0 +.1 +.0 b.1 b.0 +.1 +.0 c.1 c.0".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 8: Test [STR]
        let shortcut = "meta+pageup tab [STR]+ 123! @# [STR] meta+tab";
        let expected = Some(
            "126.1 104.1 104.0 126.0 15.1 15.0 [STR]+ 123! @#[STR] 126.1 15.1 15.0 126.0"
                .to_string(),
        );
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);
    }
}
