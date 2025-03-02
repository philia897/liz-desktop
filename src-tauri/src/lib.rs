use std::{process::exit, sync::Mutex};

use clap::Parser;
use setup::create_flute;
use tauri::{AppHandle, Emitter, Manager, RunEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

mod flute;
mod setup;
mod tools;
use flute::{BlueBirdResponse, Flute, LizCommand, StateCode};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, value_name = "PATH")]
    config: Option<String>,
}

fn execute_cmd(cmd: LizCommand, app: &AppHandle) -> BlueBirdResponse {
    match app.state::<Mutex<Flute>>().lock() {
        Ok(mut flute) => flute.play(&cmd),
        Err(e) => {
            eprintln!("Failed to lock Flute because: {}", e);
            BlueBirdResponse {
                code: StateCode::BUG,
                results: vec!["Failed to lock Flute".to_string(), format!("{}", e)],
            }
        }
    }
}

#[tauri::command]
fn send_command(cmd: LizCommand, app: AppHandle) -> BlueBirdResponse {
    match cmd.action.as_str() {
        "reload" | "create_shortcuts" | "update_shortcuts" | "delete_shortcuts"
        | "import_shortcuts" => {
            let resp: BlueBirdResponse = execute_cmd(cmd, &app);
            let _ = app.emit("fetch-again", "");
            resp
        }
        _ => execute_cmd(cmd, &app),
    }
}


// Now right-click, emit one signal and let frontend to revoke this command to create the config panel
// Refer to this https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html
// TODO: Find a better way to handle this.
#[tauri::command]
async fn create_window(app: tauri::AppHandle) {
    let path = std::path::PathBuf::from("config.html");
    let _webview_window = tauri::webview::WebviewWindowBuilder::new(&app, "config", tauri::WebviewUrl::App(path))
                    .decorations(false)
                    .transparent(true)
                    .center()
                    .inner_size(800.0, 600.0)
                    .min_inner_size(500.0, 200.0)
                    .build().unwrap();
}

fn cleanup(app: &AppHandle) {
    match app.state::<Mutex<Flute>>().lock() {
        Ok(mut flute) => {
            flute.music_sheet.clear_deleted(); // TODO: Deleted shall be more wisely handled.
            let file_path: &String = &flute.rhythm.music_sheet_path;
            if let Err(e) = flute.music_sheet.export_to_json(file_path) {
                eprintln!("Failed to save music sheet in Drop: {}", e);
            } else {
                println!("Music sheet saved successfully.");
            }
        }
        Err(e) => {
            eprintln!("Failed to lock Flute because: {}", e);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Parse the arguments
    let args = Args::parse();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        // .plugin(tauri_plugin_global_shortcut::Builder::new().build())
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
            let trigger_shortcut: String;
            match create_flute(args.config) {
                Ok(flute) => {
                    trigger_shortcut = flute.rhythm.trigger_shortcut.clone();
                    let _ = app.manage(Mutex::new(flute));
                }
                Err(e) => {
                    eprintln!("Failed to get flute: {}", e);
                    exit(1);
                }
            }
            let _ = setup::setup_tray(app);
            if let Err(e) = setup::register_trigger_shortcut(app, trigger_shortcut.as_str()) {
                eprintln!("Failed to register trigger shortcut: {}", e);
                app.dialog()
                    .message(format!{"Failed to register trigger shortcut: {}\nPlease use another one!", trigger_shortcut})
                    .kind(MessageDialogKind::Info)
                    .title("Information")
                    .buttons(MessageDialogButtons::OkCustom("OK".to_owned()))
                    .show(|_r| { });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send_command, create_window])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, e| match e {
            RunEvent::ExitRequested { .. } => {
                cleanup(app_handle);
            }
            _ => {}
        });
}
