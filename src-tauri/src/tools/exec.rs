use std::thread::sleep;
use std::time::Duration;
use std::error::Error;

use enigo::{
    Direction::{Press, Release}, Enigo, InputError, Key, Keyboard, Settings
};


/// Converts a key name (e.g., "ctrl", "u", "enter") to an enigo::Key.
/// Single characters are mapped to `Key::Unicode`.
fn string_to_key(s: &str) -> Option<Key> {
    let key_str = s.to_lowercase();
    match key_str.as_str() {
        "ctrl" | "control" => Some(Key::Control),
        "alt" => Some(Key::Alt),
        "shift" => Some(Key::Shift),
        "win" | "meta" | "cmd" => Some(Key::Meta),
        "enter" | "return" => Some(Key::Return),
        "esc" | "escape" => Some(Key::Escape),
        "space" => Some(Key::Space),
        "tab" => Some(Key::Tab),
        "backspace" => Some(Key::Backspace),
        // Arrow keys
        "up" => Some(Key::UpArrow),
        "down" => Some(Key::DownArrow),
        "left" => Some(Key::LeftArrow),
        "right" => Some(Key::RightArrow),
        // Function keys
        "f1"  => Some(Key::F1),
        "f2"  => Some(Key::F2),
        "f3"  => Some(Key::F3),
        "f4"  => Some(Key::F4),
        "f5"  => Some(Key::F5),
        "f6"  => Some(Key::F6),
        "f7"  => Some(Key::F7),
        "f8"  => Some(Key::F8),
        "f9"  => Some(Key::F9),
        "f10" => Some(Key::F10),
        "f11" => Some(Key::F11),
        "f12" => Some(Key::F12),
        // Special keys
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" => Some(Key::PageUp),
        "pagedown" => Some(Key::PageDown),
        "delete" => Some(Key::Delete),
        "insert" => Some(Key::Insert),
        "capslock" => Some(Key::CapsLock),
        // For single characters
        _ if key_str.chars().count() == 1 => {
            let ch = key_str.chars().next().unwrap();
            Some(Key::Unicode(ch))
        }
        _ => None, // Unknown Key
    }
}


/// Simulate a sequence of keyboard events using Enigo.
/// The sequence format is space-separated tokens like "ctrl.1 u.1 u.0 ctrl.0"
/// where "1" stands for Press and "0" stands for Release.
fn simulate_key_events_enigo(enigo: &mut Enigo, sequence: &str) -> Result<(), Box<dyn Error>> {
    // Split the sequence by whitespace into individual event tokens.
    for token in sequence.split_whitespace() {
        // Use the last dot to separate key from event code.
        if let Some(idx) = token.rfind('.') {
            let key_str = &token[..idx];
            let event_code = &token[idx + 1..];
            if event_code.is_empty() {
                return Err(format!("Invalid token (missing event code): '{}'", token).into());
            }
            let key = string_to_key(key_str)
                .ok_or_else(|| format!("Unknown key: '{}'", key_str))?;
            let direction = match event_code {
                "1" => Press,
                "0" => Release,
                _ => return Err(format!("Unknown event code: '{}'", event_code).into()),
            };
            // Simulate the key event.
            enigo.key(key, direction)?;
        } else {
            return Err(format!("Invalid token format (no '.' found): '{}'", token).into());
        }
    }
    Ok(())
}

/// Simulate tpying a text using Enigo.
fn simulate_text_events_enigo(enigo: &mut Enigo, text: &str) -> Result<(), InputError> {
    enigo.text(text)?;
    Ok(())
}

pub fn execute_shortcut_enigo(shortcut_str: &str, delay_ms: u64) -> Result<(), Box<dyn Error>> {
    // Initialize Enigo with the new Settings.
    let mut enigo: Enigo = match Enigo::new(&Settings::default()) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to initialize Enigo: {:?}", e);
            std::process::exit(1);
        }
    };

    let shortcuts: Vec<&str> = shortcut_str.split("[STR]").collect();

    for shortcut in shortcuts {
        if shortcut.is_empty() {
            continue;
        }

        sleep(Duration::from_millis(delay_ms)); // Sleep for the specified delay

        if shortcut.starts_with("+") {
            let type_str: &str = &shortcut[2..]; // remove the prefix
            simulate_text_events_enigo(&mut enigo, type_str)?;
        } else {
            simulate_key_events_enigo(&mut enigo, shortcut)?;
        }
    }

    Ok(())
}