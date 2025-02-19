
use tauri::{
    menu::{Menu, MenuEvent, MenuItem}, tray::TrayIconBuilder, AppHandle, Emitter, Manager
};

use crate::{flute::{Flute, LizCommand}, tools::{db::DataTable, rhythm::Rhythm, utils::get_app_config_folder}};
use std::{fs::DirBuilder, sync::Mutex};
use std::io;

/// Setup the tray, including its configuration and Menu.
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let persist_i = MenuItem::with_id(app, "persist", "Save", true, None::<&str>)?;
    let reload_i = MenuItem::with_id(app, "reload", "Reload", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &persist_i, &reload_i, &quit_i])?;

    // Create and build the tray icon
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app: &AppHandle, event: MenuEvent| handle_menu_events(app, &event))
        .show_menu_on_left_click(false)
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;
    Ok(())
}

fn handle_menu_events(app: &AppHandle, event: &MenuEvent) {
    match event.id.as_ref() {
        "show" => {
            println!("show menu item was clicked");
            if let Some(win) = app.get_webview_window("main") {
                if let Err(e) = win.show() {
                    println!("Failed to show the app, err: {}", e);
                }
                if let Err(e) = win.set_focus() {
                    println!("Failed to focus the app, err: {}", e);
                }
            } else {
                println!("handle_menu_events: Failed to get the app window");
            }
        }
        "persist" => {
            println!("Persist data into music_sheet.lock");
            match app.state::<Mutex<Flute>>().lock() {
                Ok(flute) => {
                    if let Err(e) = flute.persist() {
                        eprintln!("Failed to persist: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to lock Flute because: {}", e);
                },
            }
        }
        "reload" => {
            println!("Reload data from sheets");
            match app.state::<Mutex<Flute>>().lock() {
                Ok(mut flute) => {
                    let response = flute.play(&LizCommand {
                        action: "reload".to_string(),
                        args: vec![]
                    });
                    println!("Reload response: {:?}", response);
                },
                Err(e) => {
                    eprintln!("Failed to lock Flute because: {}", e);
                },
            }
            let _ = app.emit("fetch-again", "");
        }
        "quit" => {
            println!("quit menu item was clicked");
            app.exit(0);
        }
        _ => {
            println!("menu item {:?} not handled", event.id);
        }
    }
}

/// Create Liz folder if not exist.
pub fn create_liz_folder() -> io::Result<()> {
    let liz_folder = get_app_config_folder();

    if !liz_folder.exists() {
        // Create the 'liz' folder if it does not exist
        DirBuilder::new().recursive(true).create(&liz_folder)?;
        println!("Created 'liz' folder at {:?}", liz_folder);
    } else {
        println!("'liz' folder already exists at {:?}", liz_folder);
    }

    Ok(())
}


pub fn create_flute() -> Result<Flute, Box<dyn std::error::Error>> {
    let rhythm: Rhythm = Rhythm::read_rhythm()?;
    let music_sheet_path = &rhythm.music_sheet_path;
    let mut flute: Flute = Flute {
        music_sheet : DataTable::import_from_json(music_sheet_path)
                .unwrap_or_else(|_| {
                    eprintln!("Failed to load music sheet from {}", music_sheet_path);
                    DataTable::new()  // Return a default instance if loading fails
                }),
        rhythm : rhythm
    };
    flute.calibrate();
    Ok(flute)
}