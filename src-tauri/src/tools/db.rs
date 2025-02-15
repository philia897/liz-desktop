use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::error::Error;
use serde::{Deserialize, Serialize};

/**
 * The DataRow maintained by Bluebird, which is locked and can not be modified by user.
 */
#[derive(Debug, Serialize, Deserialize)]
struct DataRow {
    hit_number: i64,
    comment: String,
    keycode: String,
    formatted: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataTable {
    data: Vec<DataRow>,
}

impl DataTable {
    /// Initialize an empty table
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Import from JSON file
    pub fn import_from_json(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let data: Vec<DataRow> = serde_json::from_reader(file)?;
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
    fn add_row(&mut self, new_row: DataRow) {
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
        self.data.sort_by(|a, b| {
            match column {
                "hit_number" => {
                    if ascending {
                        a.hit_number.cmp(&b.hit_number)
                    } else {
                        b.hit_number.cmp(&a.hit_number)
                    }
                }
                "comment" => {
                    if ascending {
                        a.comment.cmp(&b.comment)
                    } else {
                        b.comment.cmp(&a.comment)
                    }
                }
                "keycode" => {
                    if ascending {
                        a.keycode.cmp(&b.keycode)
                    } else {
                        b.keycode.cmp(&a.keycode)
                    }
                }
                "formatted" => {
                    if ascending {
                        a.formatted.cmp(&b.formatted)
                    } else {
                        b.formatted.cmp(&a.formatted)
                    }
                }
                _ => std::cmp::Ordering::Equal,
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
pub struct UserDataTable {
    data: Vec<UserDataRow>,
}

impl UserDataTable {
    // Initialize an empty table
    // pub fn new() -> Self {
    //     Self { data: Vec::new() }
    // }

    pub fn import_from(path: &str) -> Result<Self, Box<dyn Error>> {
        let metadata = fs::metadata(path)?;

        if metadata.is_file() {
            UserDataTable::import_from_json(path)
        } else if metadata.is_dir() {
            UserDataTable::import_from_json_dir(path)
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

    pub fn transform_to_data_table(&self, old_table : &DataTable, keymap_path: &str) -> Result<DataTable, Box<dyn Error>> {
        // get a HashMap of <formatted, hit_number>
        let map_fh: HashMap<String, i64> = old_table.data.iter().map(|row| (row.formatted.clone(), row.hit_number)).collect();

        // Read the key event codes from file
        let key_event_codes: HashMap<String, String> = self.read_keymap(keymap_path);

        let mut new_table = DataTable::new();
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
            new_table.add_row(DataRow {
                hit_number : *map_fh.get(&formatted).unwrap_or(&0),
                comment : row.comment.clone(),
                keycode : keycode,
                formatted : formatted
            });
        }

        Ok(new_table)
    }

}

/**
 * Convert shortcut string to key presses, using the keymap to map key to keycode
 * For example:
 * meta+pageup tab 123!@# tab ABC  
 * => 126.1 104.1 104.0 126.0 15.1 15.0 [STR]+ 123!@#[STR] 15.1 15.0 [STR]+ ABC[STR]
 * Where keycode of meta is 126, pageup (104), tab (15)
 * type 123!@ means directly type these characters "123!@".
 */
fn convert_shortcut_to_key_presses(shortcut: &str, key_event_codes: &HashMap<String, String>) -> Option<String> {
    let mut result = Vec::new();

    let ss: Vec<&str> = shortcut.split("[STR]").collect();
    
    for s in ss {
        if s.is_empty() {
            continue;
        }
        if s.starts_with("+") {
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
                    let keys: Vec<&str> = part.split('+').collect();
                    for key in &keys {
                        let key: String = key.trim().to_lowercase();
                        if let Some(event_code) = key_event_codes.get(&key) {
                            result.push(format!("{}.1", event_code));
                        } else {
                            result.push(format!("{}.1", key));
                        }
                    }
                    for key in keys.iter().rev() {
                        let key: String = key.trim().to_lowercase();
                        if let Some(event_code) = key_event_codes.get(&key) {
                            result.push(format!("{}.0", event_code));
                        } else {
                            result.push(format!("{}.0", key));
                        }
                    }
                } else {
                    let key = part.trim().to_lowercase();
                    if let Some(event_code) = key_event_codes.get(&key) {
                        result.push(format!("{}.1", event_code));
                        result.push(format!("{}.0", event_code));
                    } else {
                        result.push(format!("[STR]+ {}[STR]", part.trim()));
                    }
                }
            }
        }
    }

    Some(result.join(" "))
}

