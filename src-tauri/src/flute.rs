use serde::{Deserialize, Serialize};

use crate::tools::{db::{ShortcutDB, UserSheet}, exec::execute_shortcut_enigo, rhythm::Rhythm};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum StateCode {
    OK,
    FAIL,
    BUG,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LizCommand {
    pub action: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlueBirdResponse {
    pub code: StateCode,
    pub results: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Flute {
    pub music_sheet : ShortcutDB,
    pub rhythm : Rhythm
}

impl Flute {

    pub fn calibrate(&mut self) -> &mut Self {
        self.update_rank();
        self
    }

    fn update_rank(&mut self) {
        self.music_sheet.sort_by_column("formatted", true);
        self.music_sheet.sort_by_column("hit_number", false);
    }

    pub fn play(&mut self, cmd: &LizCommand) -> BlueBirdResponse {
        match cmd.action.as_str() {
            "get_shortcuts" => self.command_get_shortcuts(cmd),
            "reload" => self.command_reload(cmd),
            "execute" => self.command_execute(cmd),
            "persist" => self.command_persist(cmd),
            "info" => self.command_info(cmd),
            _ => self.command_default(cmd),
        }
    }
    
    fn command_get_shortcuts(&self, _cmd: &LizCommand) -> BlueBirdResponse {
        BlueBirdResponse {
            code : StateCode::OK,
            results : self.music_sheet.get_formatted_vec()
        }
    }
    
    fn command_reload(&mut self, cmd: &LizCommand) -> BlueBirdResponse {
        let user_data_path: &String;
        if cmd.args.is_empty() {
            user_data_path = &self.rhythm.user_sheets_path;
        } else {
            user_data_path = &cmd.args[0];
        }
        match UserSheet::import_from(&user_data_path) {
            Ok(user_data) => {
                self.music_sheet = user_data.transform_to_db(&self.music_sheet, &self.rhythm.keymap_path);
                BlueBirdResponse {
                    code : StateCode::OK,
                    results : vec!["Reload Done".to_string()]
                }
            }
            Err(e) => {
                eprintln!("Failure: failed to import user data from: {}, error: {}", user_data_path, e);
                BlueBirdResponse {
                    code : StateCode::FAIL,
                    results : vec!["Failure:".to_string(), "Failed to import:".to_string(), user_data_path.to_string()]
                }
            }
        }
    }
    
    fn command_execute(&mut self, cmd: &LizCommand) -> BlueBirdResponse {
        if cmd.args.is_empty() {
            eprintln!("BUG: Empty args, expect one index on args[0]");
            return BlueBirdResponse {
                code : StateCode::BUG,
                results : vec!["BUG:".to_string(), "Empty args:".to_string(), "Expect one index".to_string()]
            }
        }
        match cmd.args[0].parse::<usize>() {
            Ok(idx) => {
                let keycode = self.music_sheet.get_value(idx, "keycode");
                if keycode.is_none() {
                    eprintln!("BUG: No keycode found on index {}", cmd.args[0]);
                    return BlueBirdResponse {
                        code : StateCode::BUG,
                        results : vec!["BUG:".to_string(), "No keycode found on index:".to_string(), cmd.args[0].clone()]
                    }
                }
                println!("Execute: {}: {}", idx, self.music_sheet.get_value(idx, "formatted").unwrap() );
                if let Err(e) = execute_shortcut_enigo(&keycode.unwrap(), self.rhythm.interval_ms) {
                    eprintln!("Enigo Failure: Fail to execute shortcut: {:?}", e);
                    return BlueBirdResponse {
                        code : StateCode::FAIL,
                        results : vec!["Failure:".to_string(), format!("{}", e)]
                    }
                }
                let _ = self.music_sheet.hit_num_up(idx);
                self.update_rank();
                return BlueBirdResponse {
                    code : StateCode::OK,
                    results : vec![]
                }
            },
            Err(_e) => {
                eprintln!("BUG: Parsing this index error: {}", cmd.args[0]);
                return BlueBirdResponse {
                    code : StateCode::BUG,
                    results : vec!["BUG:".to_string(), "Parsing this index error:".to_string(), cmd.args[0].clone()]
                }
            },
        }
    }

    pub fn persist(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.music_sheet.export_to_json(&self.rhythm.music_sheet_path)
    }
    
    fn command_persist(&self, _cmd: &LizCommand) -> BlueBirdResponse {
        match self.persist() {
            Ok(()) => {
                BlueBirdResponse{
                    code : StateCode::OK,
                    results : vec![]
                }
            },
            Err(e) => {
                eprintln!("BUG: Failed to persist music_sheet, error: {}", e);
                BlueBirdResponse{
                    code : StateCode::BUG,
                    results : vec!["BUG:".to_string(), "Failed to persist music_sheet".to_string()]
                }
            }
        }
    }

    fn command_info(&self, _cmd: &LizCommand) -> BlueBirdResponse {
        let r: &Rhythm = &self.rhythm;
        BlueBirdResponse{
            code : StateCode::OK,
            results : r.to_pretty_vec()
        }
    }

    fn command_default(&self, cmd: &LizCommand) -> BlueBirdResponse {
        BlueBirdResponse {
            code : StateCode::FAIL,
            results : vec![cmd.action.to_string(), "Invalid".to_string()]
        }
    }
}

// Implement the Drop trait for Flute
// impl Drop for Flute {
//     fn drop(&mut self) {
//         // Attempt to save the music_sheet when the Flute instance is dropped
//         let file_path: &String = &self.rhythm.music_sheet_path;
//         if let Err(e) = self.music_sheet.export_to_json(file_path) {
//             eprintln!("Failed to save music sheet in Drop: {}", e);
//         } else {
//             println!("Music sheet saved successfully in Drop.");
//         }
//     }
// }
