use std::{process::exit, sync::Mutex};

use setup::create_flute;
use tauri::{AppHandle, Emitter, Manager, RunEvent};

mod flute;
mod setup;
mod tools;
use flute::{BlueBirdResponse, LizCommand, StateCode, Flute};

fn execute_cmd(cmd: LizCommand, app: &AppHandle) -> BlueBirdResponse {
    match app.state::<Mutex<Flute>>().lock() {
        Ok(mut flute) => {
            flute.play(&cmd)
        },
        Err(e) => {
            eprintln!("Failed to lock Flute because: {}", e);
            BlueBirdResponse {
                code: StateCode::BUG,
                results: vec!["Failed to lock Flute".to_string(), format!("{}", e)]
            }
        },
    }
}

#[tauri::command]
fn send_command(cmd: LizCommand, app: AppHandle) -> BlueBirdResponse {
    match cmd.action.as_str() {
        "reload" => {
            let resp = execute_cmd(cmd, &app);
            let _ = app.emit("fetch-again", "");
            resp
        }
        _ => {execute_cmd(cmd, &app)}
    }
}

fn cleanup(app: &AppHandle) {
    match app.state::<Mutex<Flute>>().lock() {
        Ok(flute) => {
            let file_path: &String = &flute.rhythm.music_sheet_path;
            if let Err(e) = flute.music_sheet.export_to_json(file_path) {
                eprintln!("Failed to save music sheet in Drop: {}", e);
            } else {
                println!("Music sheet saved successfully.");
            }
        },
        Err(e) => {
            eprintln!("Failed to lock Flute because: {}", e);
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = setup::create_liz_folder() {
        eprintln!("Failed to get liz working dir because: {}", e);
        exit(1);
    }

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(win) = app.get_webview_window("main") {
                if let Err(e) = win.show() {
                    println!("Failed to show the app, err: {}", e);
                }
                if let Err(e) = win.set_focus() {
                    println!("Failed to focus the app, err: {}", e);
                }
            }
        }));
    }

    builder
        .setup(|app| {
            match create_flute() {
                Ok(flute) => {
                    let _ = app.manage(Mutex::new(flute));
                },
                Err(e) => {
                    eprintln!("Failed to get flute: {}", e);
                    exit(1);
                }
            }
            let _ = setup::setup_tray(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send_command])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, e| match e {
            RunEvent::ExitRequested { ..} => {
                cleanup(app_handle);
            }
            _ => {}
        });
}
