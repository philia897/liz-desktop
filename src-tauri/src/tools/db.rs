use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::error::Error;
use serde::{Deserialize, Serialize};

/**
 * The DataRow maintained by Bluebird, which is locked and can not be modified by user.
 */
#[derive(Debug, Serialize, Deserialize)]
struct Shortcut {
    hit_number: i64,
    comment: String,
    keycode: String,
    formatted: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShortcutDB {
    data: Vec<Shortcut>,
}

impl ShortcutDB {
    /// Initialize an empty table
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Import from JSON file
    pub fn import_from_json(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let data: Vec<Shortcut> = serde_json::from_reader(file)?;
        Ok(Self{data: data})
    }

    /// Export to JSON file
    pub fn export_to_json(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let _ = std::fs::remove_file(file_path);
        let file = OpenOptions::new().write(true).create(true).open(file_path)?;
        serde_json::to_writer(file, &self.data)?;
        Ok(())
    }

    /// Get a value by row and column name
    pub fn get_value(&self, row: usize, column: &str) -> Option<String> {
        self.data.get(row).and_then(|r| match column {
            "hit_number" => Some(r.hit_number.to_string()),
            "comment" => Some(r.comment.clone()),
            "keycode" => Some(r.keycode.clone()),
            "formatted" => Some(r.formatted.clone()),
            _ => None,
        })
    }

    /// Get all values in the "formatted" column as a Vec<String>
    pub fn get_formatted_vec(&self) -> Vec<String> {
        self.data.iter().map(|row| row.formatted.clone()).collect()
    }

    /// Method to add a new row to the DataTable
    fn add_row(&mut self, new_row: Shortcut) {
        self.data.push(new_row);
    }

    /// Function to increase hit_number for a given row index
    pub fn hit_num_up(&mut self, row: usize) -> Result<(), String> {
        if row < self.data.len() {
            self.data[row].hit_number += 1; // Increment the hit_number
            Ok(())
        } else {
            Err(format!("Row index {} is out of bounds", row)) // Return an error if the index is invalid
        }
    }

    /// Sort by a specific column name
    pub fn sort_by_column(&mut self, column: &str, ascending: bool) {
        // Decide on the comparison function based on the column, done once
        let comparator: Box<dyn Fn(&Shortcut, &Shortcut) -> std::cmp::Ordering> = match column {
            "hit_number" => Box::new(|a, b| a.hit_number.cmp(&b.hit_number)),
            "comment" => Box::new(|a, b| a.comment.cmp(&b.comment)),
            "keycode" => Box::new(|a, b| a.keycode.cmp(&b.keycode)),
            "formatted" => Box::new(|a, b| a.formatted.cmp(&b.formatted)),
            _ => Box::new(|_, _| std::cmp::Ordering::Equal),  // Handle unknown column names
        };

        // Now sort the data using the pre-selected comparator
        self.data.sort_by(|a, b| {
            let ordering = comparator(a, b);
            if ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
    }

}


/**
 * The data defined by user, need to be transformed to DataRow before using
 */
#[derive(Debug, Serialize, Deserialize)]
struct UserDataRow {
    description: String,
    shortcut: String,
    application: String,
    comment: String,
}

impl UserDataRow {
    /// Method to format the output, used for showing on Liz
    pub fn format_output(&self) -> String {
        format!("<b>{}</b> | {} | {}", self.description, self.application, self.shortcut)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSheet {
    data: Vec<UserDataRow>,
}

impl UserSheet {
    // Initialize an empty table
    // pub fn new() -> Self {
    //     Self { data: Vec::new() }
    // }

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
        let data: Vec<UserDataRow> = serde_json::from_reader(file)?;
        Ok(Self{data: data})
    }

    /// Import all JSON files from a directory
    fn import_from_json_dir(dir_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut all_data: Vec<UserDataRow> = Vec::new();

        // Iterate over all entries in the directory
        for entry in fs::read_dir(dir_path)? {
            let entry: fs::DirEntry = entry?;
            let path: std::path::PathBuf = entry.path();

            // Check if the entry is a file and ends with .json
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let file = File::open(&path)?;
                
                // Deserialize the JSON content into UserDataRow
                let data: Vec<UserDataRow> = serde_json::from_reader(file)?;
                
                // Extend the result vector with the new data
                all_data.extend(data);
            }
        }

        Ok(Self { data: all_data })
    }

    fn read_keymap(&self, keymap_path: &str) -> HashMap<String, String> {
        // Attempt to open the file
        let mut file = match File::open(keymap_path) {
            Ok(f) => f,
            Err(e) => {
                eprint!("Error opening keymap file: {}\n", e);
                return HashMap::new(); // Return an empty map in case of error
            }
        };
    
        // Read the contents of the file
        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents) {
            eprint!("Error reading keymap file: {}\n", e);
            return HashMap::new(); // Return an empty map in case of error
        }
    
        // Parse the contents as JSON
        match serde_json::from_str(&contents) {
            Ok(key_event_codes) => key_event_codes,
            Err(e) => {
                eprint!("Error parsing keymap JSON: {}\n", e);
                HashMap::new() // Return an empty map in case of error
            }
        }
    }

    /// Replace the shortcuts stored in the old music_sheet while maintaining its hit_numbers for repeated shortcuts
    pub fn transform_to_db(&self, old_db : &ShortcutDB, keymap_path: &str) -> ShortcutDB {
        // get a HashMap of <formatted, hit_number>
        let map_fh: HashMap<String, i64> = old_db.data.iter().map(|row| (row.formatted.clone(), row.hit_number)).collect();

        // Read the key event codes from file
        let key_event_codes: HashMap<String, String> = self.read_keymap(keymap_path);

        let mut new_db = ShortcutDB::new();
        for row in &self.data {
            let keycode: String;
            let formatted : String;
            if let Some(code) = convert_shortcut_to_key_presses(&row.shortcut, &key_event_codes) {
                keycode = code;
                formatted = row.format_output();
            } else {
                keycode = "".to_string();
                eprintln!("Transforming error: {:?}", row);
                formatted = UserDataRow {
                    description : row.description.clone(),
                    shortcut : format!("{} | <b>Err!<b>", row.shortcut),
                    application : row.application.clone(),
                    comment : row.comment.clone()
                }.format_output();
            }
            new_db.add_row(Shortcut {
                hit_number : *map_fh.get(&formatted).unwrap_or(&0),
                comment : row.comment.clone(),
                keycode : keycode,
                formatted : formatted
            });
        }

        new_db
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
fn convert_shortcut_to_key_presses(shortcut: &str, key_event_codes: &HashMap<String, String>) -> Option<String> {
    let mut result = Vec::new();

    // Split by marker [STR] to different blocks
    let ss: Vec<&str> = shortcut.split("[STR]").collect();
    
    for s in ss {
        if s.is_empty() {
            continue;
        }
        if s.starts_with("+") {  // Typing the string
            let type_str: &str = &s[2..];
            result.push(format!("[STR]+ {}[STR]", type_str.trim()));
        } else {
            // Split the input by spaces
            let parts: Vec<&str> = s.split_whitespace().collect();

            for part in parts {
                if part.is_empty() {
                    continue;
                }
                if part.contains('+') && part != "+" {   // Execute shortcut like ctrl+c, ctrl+v
                    let keys: Vec<&str> = part.split('+').collect();
                    for key in &keys {     // Press
                        let key: String = key.trim().to_lowercase();
                        if let Some(event_code) = key_event_codes.get(&key) {
                            result.push(format!("{}.1", event_code));
                        } else {
                            result.push(format!("{}.1", key));
                        }
                    }
                    for key in keys.iter().rev() {   // Release
                        let key: String = key.trim().to_lowercase();
                        if let Some(event_code) = key_event_codes.get(&key) {
                            result.push(format!("{}.0", event_code));
                        } else {
                            result.push(format!("{}.0", key));
                        }
                    }
                } else {     // Not a shortcut, either one single key or a string to type
                    let key = part.trim().to_lowercase();
                    if let Some(event_code) = key_event_codes.get(&key) {  // Press one key
                        result.push(format!("{}.1", event_code));
                        result.push(format!("{}.0", event_code));
                    } else if key.len() == 1 {              // Press one character
                        let k = part.trim();
                        result.push(format!("{}.1", k));
                        result.push(format!("{}.0", k));
                    } else {                  //  Type the string
                        result.push(format!("[STR]+ {}[STR]", part.trim()));
                    }
                }
            }
        }
    }

    Some(result.join(" "))
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
        assert_eq!(result, expected);

        // Test 2: Test with characters (e.g., numbers or symbols)
        let shortcut = "123!@# tab ABC";
        let expected = Some("[STR]+ 123!@#[STR] 15.1 15.0 [STR]+ ABC[STR]".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(result, expected);

        // Test 3: Test with more complex shortcuts (e.g., multiple key combinations)
        let shortcut = "meta+pageup tab 123!@# meta+tab";
        let expected = Some("126.1 104.1 104.0 126.0 15.1 15.0 [STR]+ 123!@#[STR] 126.1 15.1 15.0 126.0".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(result, expected);

        // Test 4: Test with unrecognized keys (e.g., no mapping for 'enter')
        let shortcut = "enter tab";
        let expected = Some("[STR]+ enter[STR] 15.1 15.0".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(result, expected);

        // Test 5: Test with additional '+' combinations
        let shortcut = "meta+tab+pageup";
        let expected = Some("126.1 15.1 104.1 104.0 15.0 126.0".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(result, expected);

        // Test 6: Test empty input
        let shortcut = "";
        let expected = Some("".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(result, expected);

        // Test 7: Test plus with space
        let shortcut = "a + b + c";
        let expected = Some("a.1 a.0 +.1 +.0 b.1 b.0 +.1 +.0 c.1 c.0".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(result, expected);

        // Test 8: Test [STR]
        let shortcut = "meta+pageup tab [STR]+ 123! @# [STR] meta+tab";
        let expected = Some("126.1 104.1 104.0 126.0 15.1 15.0 [STR]+ 123! @#[STR] 126.1 15.1 15.0 126.0".to_string());
        let result = convert_shortcut_to_key_presses(shortcut, &key_event_codes);
        assert_eq!(result, expected);

    }
}