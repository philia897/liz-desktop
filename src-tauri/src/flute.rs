use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::tools::{
    db::{MusicSheetDB, Shortcut, UserSheet},
    exec::execute_shortcut_enigo,
    rhythm::{parse_rhythm, Rhythm},
    utils::{generate_id, id_to_string, string_to_id},
};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum StateCode {
    OK,
    FAIL,
    BUG,
}

// Implement Display for StateCode to allow it to be printed
impl fmt::Display for StateCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_str = match *self {
            StateCode::OK => "OK",
            StateCode::FAIL => "FAIL",
            StateCode::BUG => "BUG",
        };
        write!(f, "{}", state_str)
    }
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

impl BlueBirdResponse {
    pub fn new() -> BlueBirdResponse {
        Self {
            code: StateCode::OK,
            results: vec![],
        }
    }
}

#[derive(Debug)]
pub struct FluteExecuteError {
    msg: String,
    code: StateCode,
}

impl FluteExecuteError {
    // Constructor to create a new FluteExecuteError
    pub fn new(msg: &str, code: StateCode) -> Self {
        FluteExecuteError {
            msg: msg.to_string(),
            code,
        }
    }

    // Method to get the error message
    pub fn message(&self) -> &str {
        &self.msg
    }

    // Method to get the state code
    pub fn code(&self) -> &StateCode {
        &self.code
    }
}

// Implement the Display trait to format the error message
impl fmt::Display for FluteExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

// Implement the Error trait for FluteExecuteError
impl Error for FluteExecuteError {}

#[derive(Debug)]
pub struct Flute {
    pub music_sheet: MusicSheetDB,
    pub rhythm: Rhythm,
}

impl Flute {
    pub fn calibrate(&mut self) -> &mut Self {
        self.update_rank();
        self
    }

    fn update_rank(&mut self) {
        self.music_sheet.sort_by_column("application", true);
        self.music_sheet.sort_by_column("hit_number", false);
    }

    pub fn play(&mut self, cmd: &LizCommand) -> BlueBirdResponse {
        match cmd.action.as_str() {
            "get_shortcuts" => self.command_get_shortcuts(cmd),
            // "reload" => self.command_reload(cmd),
            "execute" => self.command_execute(cmd),
            "persist" => self.command_persist(cmd),
            "info" => self.command_info(cmd),
            "get_shortcut_details" => self.command_get_shortcut_details(cmd),
            "new_id" => self.command_new_id(cmd),
            "create_shortcuts" => self.command_create_shortcuts(cmd),
            "update_shortcuts" => self.command_update_shortcuts(cmd),
            "delete_shortcuts" => self.command_delete_shortcuts(cmd),
            "get_deleted_shortcut_details" => self.command_get_deleted_shortcut_details(cmd),
            "export_shortcuts" => self.command_export_shortcuts(cmd),
            "import_shortcuts" => self.command_import_shortcuts(cmd),
            "update_rhythm" => self.command_update_rhythm(cmd),
            _ => self.command_default(cmd),
        }
    }

    fn _get_sc_by_id(&self, id_str: &str) -> Result<Shortcut, Box<dyn Error>> {
        let id: u128 = string_to_id(id_str)?;
        let r: &Shortcut = self
            .music_sheet
            .retrieve(id, None)
            .ok_or("Id does not exist".to_string())?;
        Ok(r.clone())
    }

    fn command_export_shortcuts(&self, cmd: &LizCommand) -> BlueBirdResponse {
        fn split_vec(vec: &Vec<String>) -> Option<(String, Vec<String>)> {
            let (first, rest) = vec.split_first()?; // Get first element and the rest
            Some((first.clone(), rest.to_vec())) // Clone to return owned values
        }
        if let Some((file_path, id_list)) = split_vec(&cmd.args) {
            let sc_to_export: Result<Vec<Shortcut>, _> = id_list
                .iter()
                .map(|id_str| self._get_sc_by_id(&id_str))
                .collect();
            match sc_to_export {
                Ok(sc_to_export) => {
                    println!("Export to {}", file_path);
                    let sheet = UserSheet::new(sc_to_export);
                    match sheet.export_to_json(&file_path) {
                        Ok(_) => BlueBirdResponse::new(),
                        Err(e) => {
                            let err_str = format!("Failed to export to {}: {}", file_path, e);
                            eprintln!("Export Shortcuts: {}", err_str);
                            BlueBirdResponse {
                                code: StateCode::BUG,
                                results: vec![err_str],
                            }
                        }
                    }
                }
                Err(e) => {
                    let err_str = format!("Failed to parse id: {}", e);
                    eprintln!("Export Shortcuts: {}", err_str);
                    BlueBirdResponse {
                        code: StateCode::BUG,
                        results: vec![err_str],
                    }
                }
            }
        } else {
            BlueBirdResponse {
                code: StateCode::BUG,
                results: vec!["File path is not given".to_string()],
            }
        }
    }

    fn command_import_shortcuts(&mut self, cmd: &LizCommand) -> BlueBirdResponse {
        if cmd.args.is_empty() {
            eprintln!("BUG: Empty args, expect one file_path");
            return BlueBirdResponse {
                code: StateCode::BUG,
                results: vec!["Empty args, expect one shortcut id".to_string()],
            };
        }
        println!("Import from {:?}", cmd.args);
        let mut failed_paths: Vec<String> = Vec::new();
        for file in cmd.args.iter() {
            match UserSheet::import_from(file) {
                Ok(sheet) => {
                    sheet.transform_to_db(&mut self.music_sheet);
                }
                Err(e) => {
                    let err_str = format!("Failed to import file {}: {}", file, e);
                    eprintln!("Export Shortcuts: {}", err_str);
                    failed_paths.push(file.clone());
                }
            }
        }
        if failed_paths.is_empty() {
            BlueBirdResponse::new()
        } else {
            BlueBirdResponse {
                code: StateCode::FAIL,
                results: failed_paths,
            }
        }
    }

    fn command_get_shortcuts(&self, _cmd: &LizCommand) -> BlueBirdResponse {
        let fmt = &self.rhythm.shortcut_print_fmt;
        let shortcuts = self.music_sheet.retrieve_all();
        let sc_vec: Vec<String> = shortcuts
            .into_iter()
            .map(|sc| {
                // Create a JSON string
                let json = serde_json::json!({
                    "id": id_to_string(sc.id),  // Convert id to string
                    "sc": sc.format_output(fmt)
                });
                // Serialize it into a JSON string
                serde_json::to_string(&json).unwrap() // Use unwrap or handle errors properly
            })
            .collect();
        BlueBirdResponse {
            code: StateCode::OK,
            results: sc_vec,
        }
    }

    fn command_get_shortcut_details(&self, _cmd: &LizCommand) -> BlueBirdResponse {
        let shortcuts = self.music_sheet.retrieve_all();
        let sc_vec: Vec<String> = shortcuts
            .into_iter()
            .map(|sc| sc.to_json_string())
            .collect();
        BlueBirdResponse {
            code: StateCode::OK,
            results: sc_vec,
        }
    }

    fn command_get_deleted_shortcut_details(&self, _cmd: &LizCommand) -> BlueBirdResponse {
        let shortcuts = self.music_sheet.retrieve_deleted();
        let sc_vec: Vec<String> = shortcuts
            .into_iter()
            .map(|sc| sc.to_json_string())
            .collect();
        BlueBirdResponse {
            code: StateCode::OK,
            results: sc_vec,
        }
    }

    fn command_new_id(&self, _cmd: &LizCommand) -> BlueBirdResponse {
        BlueBirdResponse {
            code: StateCode::OK,
            results: vec![id_to_string(generate_id())],
        }
    }

    fn _args_to_shortcut_vec(&self, cmd: &LizCommand) -> Result<Vec<Shortcut>, String> {
        let shortcuts: Result<Vec<Shortcut>, _> = cmd
            .args
            .iter()
            .map(|sc_str| Shortcut::from_json_string(&sc_str))
            .collect();
        match shortcuts {
            Ok(shortcuts) => Ok(shortcuts),
            Err(e) => {
                let err_str = format!("Failed to parse shortcut: {}", e);
                Err(err_str)
            }
        }
    }

    fn command_create_shortcuts(&mut self, cmd: &LizCommand) -> BlueBirdResponse {
        match self._args_to_shortcut_vec(cmd) {
            Ok(shortcuts) => {
                self.music_sheet.add_shortcuts(shortcuts, None);
                BlueBirdResponse::new()
            }
            Err(e) => {
                eprintln!("Create Shortcuts: {}", e);
                BlueBirdResponse {
                    code: StateCode::BUG,
                    results: vec![e],
                }
            }
        }
    }

    fn command_update_shortcuts(&mut self, cmd: &LizCommand) -> BlueBirdResponse {
        match self._args_to_shortcut_vec(cmd) {
            Ok(shortcuts) => {
                let unmatched: Vec<Shortcut> = self.music_sheet.update_shortcuts(shortcuts);
                let unmatched: Vec<String> =
                    unmatched.iter().map(|sc| sc.to_json_string()).collect();
                if unmatched.is_empty() {
                    BlueBirdResponse {
                        code: StateCode::OK,
                        results: unmatched,
                    }
                } else {
                    eprintln!("Unmatched: {:?}", unmatched);
                    BlueBirdResponse {
                        code: StateCode::FAIL,
                        results: unmatched,
                    }
                }
            }
            Err(e) => {
                eprintln!("Update Shortcuts: {}", e);
                BlueBirdResponse {
                    code: StateCode::BUG,
                    results: vec![e],
                }
            }
        }
    }

    fn command_delete_shortcuts(&mut self, cmd: &LizCommand) -> BlueBirdResponse {
        let id_to_delete: Result<Vec<u128>, _> = cmd
            .args
            .iter()
            .map(|id_str| string_to_id(&id_str))
            .collect();
        match id_to_delete {
            Ok(id_to_delete) => {
                self.music_sheet.delete_shortcuts(id_to_delete);
                BlueBirdResponse::new()
            }
            Err(e) => {
                let err_str = format!("Failed to parse id: {}", e);
                eprintln!("Delete Shortcuts: {}", err_str);
                BlueBirdResponse {
                    code: StateCode::BUG,
                    results: vec![err_str],
                }
            }
        }
    }

    fn command_update_rhythm(&mut self, cmd: &LizCommand) -> BlueBirdResponse {
        if cmd.args.is_empty() {
            return BlueBirdResponse {
                code: StateCode::BUG,
                results: vec![format!("Settings is missing")]
            }
        }

        let new_rhythm = parse_rhythm(&cmd.args[0]);
        match new_rhythm {
            Ok(new_rhythm) => {
                let saved_path = new_rhythm.save_rhythm(None); // Save to the default path
                self.rhythm = new_rhythm;
                match saved_path {
                    Ok(saved_path) => BlueBirdResponse {
                        code: StateCode::OK,
                        results: vec![saved_path]
                    },
                    Err(e) => {
                        let err_msg = format!("Failed to save rhythm to {}\nError: {}", &cmd.args[0], e);
                        BlueBirdResponse {
                        code: StateCode::FAIL,
                        results: vec![err_msg]
                        }
                    }
                }
                
            },
            Err(e) => {
                let err_msg = format!("Failed to parse rhythm: {}\nError: {}", cmd.args[0], e);
                eprint!("{}", err_msg);
                BlueBirdResponse {
                    code: StateCode::BUG,
                    results: vec![err_msg]
                }
            },
        }
    }

    // fn command_reload(&mut self, cmd: &LizCommand) -> BlueBirdResponse {
    //     let user_data_path: &String;
    //     if cmd.args.is_empty() {
    //         user_data_path = &self.rhythm.user_sheets_path;
    //     } else {
    //         user_data_path = &cmd.args[0];
    //     }
    //     match UserSheet::import_from(&user_data_path) {
    //         Ok(user_data) => {
    //             user_data.transform_to_db(&mut self.music_sheet);
    //             BlueBirdResponse::new()
    //         }
    //         Err(e) => {
    //             eprintln!(
    //                 "Failure: failed to import user data from: {}, error: {}",
    //                 user_data_path, e
    //             );
    //             BlueBirdResponse {
    //                 code: StateCode::FAIL,
    //                 results: vec![
    //                     "Failure:".to_string(),
    //                     "Failed to import:".to_string(),
    //                     user_data_path.to_string(),
    //                 ],
    //             }
    //         }
    //     }
    // }

    /// Execute the shortcut of given id
    fn _execute(&mut self, id_str: &str) -> Result<(), FluteExecuteError> {
        match string_to_id(id_str) {
            Ok(id) => {
                let sc = self.music_sheet.retrieve(id, None);
                if sc.is_none() {
                    return Err(FluteExecuteError::new(
                        &format!("No keycode found for id {}", id_str),
                        StateCode::BUG,
                    ));
                }
                let sc = sc.unwrap();
                match sc.parse_to_keycode(&self.music_sheet.keymap) {
                    Err(e) => {
                        let err_str = format!("Failed to parse {}: {}", sc.shortcut, e);
                        Err(FluteExecuteError::new(&err_str, StateCode::BUG))
                    }
                    Ok(keycode) => {
                        println!("Execute: {}: {}", id_str, keycode);
                        if let Err(e) = execute_shortcut_enigo(&keycode, self.rhythm.interval_ms) {
                            let err_str =
                                format!("Enigo fails to execute shortcut {}: {}", sc.shortcut, e);
                            return Err(FluteExecuteError::new(&err_str, StateCode::FAIL));
                        }
                        let _ = self.music_sheet.hit_num_up(id);
                        self.update_rank();
                        Ok(())
                    }
                }
            }
            Err(e) => {
                let err_str = format!("BUG: Failed to parse ID {}: {}", id_str, e);
                Err(FluteExecuteError::new(&err_str, StateCode::BUG))
            }
        }
    }

    fn command_execute(&mut self, cmd: &LizCommand) -> BlueBirdResponse {
        if cmd.args.is_empty() {
            eprintln!("BUG: Empty args, expect one index on args[0]");
            return BlueBirdResponse {
                code: StateCode::BUG,
                results: vec!["Empty args, expect one shortcut id".to_string()],
            };
        }
        match self._execute(cmd.args[0].as_str()) {
            Ok(_) => BlueBirdResponse::new(),
            Err(e) => {
                eprint!("Execute: {}", e);
                BlueBirdResponse {
                    results: vec![e.message().to_string()],
                    code: e.code,
                }
            }
        }
    }

    pub fn persist(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.music_sheet
            .export_to_json(&self.rhythm.music_sheet_path)
    }

    fn command_persist(&self, _cmd: &LizCommand) -> BlueBirdResponse {
        match self.persist() {
            Ok(()) => BlueBirdResponse::new(),
            Err(e) => {
                eprintln!("BUG: Failed to persist music_sheet, error: {}", e);
                BlueBirdResponse {
                    code: StateCode::BUG,
                    results: vec!["Failed to persist music_sheet".to_string()],
                }
            }
        }
    }

    fn command_info(&self, _cmd: &LizCommand) -> BlueBirdResponse {
        let r: &Rhythm = &self.rhythm;
        BlueBirdResponse {
            code: StateCode::OK,
            results: r.to_string_list(),
        }
    }

    fn command_default(&self, cmd: &LizCommand) -> BlueBirdResponse {
        eprint!("Invalid Cmd: {:#?}", cmd);
        BlueBirdResponse {
            code: StateCode::BUG,
            results: vec![format!("Invalid Liz Cmd: {}", cmd.action)],
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
