use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use std::collections::HashMap;

use enigo::{
    Direction::{Press, Release},
    Enigo, InputError, Key, Keyboard, Settings,
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
        "f1" => Some(Key::F1),
        "f2" => Some(Key::F2),
        "f3" => Some(Key::F3),
        "f4" => Some(Key::F4),
        "f5" => Some(Key::F5),
        "f6" => Some(Key::F6),
        "f7" => Some(Key::F7),
        "f8" => Some(Key::F8),
        "f9" => Some(Key::F9),
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
            let key =
                string_to_key(key_str).ok_or_else(|| format!("Unknown key: '{}'", key_str))?;
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
    let mut enigo: Enigo = Enigo::new(&Settings::default())?;

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


/**
 * Convert shortcut string to key presses, using the keymap to map key to keycode
 * For example:
 * meta+pageup tab 123!@# tab ABC  
 * => 126.1 104.1 104.0 126.0 15.1 15.0 [STR]+ 123!@#[STR] 15.1 15.0 [STR]+ ABC[STR]
 * Where keycode of meta is 126, pageup (104), tab (15)
 * type 123!@ means directly type these characters "123!@".
 * Note: "ctrl + c" will be consider press "ctrl", then "+" then "c", as they are splited by space.
 */
pub fn convert_shortcut_to_keycode(
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
                    } else if string_to_key(&key).is_some() {
                        // Press one key
                        result.push(format!("{}.1", key));
                        result.push(format!("{}.0", key));
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
    fn test_convert_shortcut_to_keycode() {
        let mut key_event_codes = HashMap::new();
        key_event_codes.insert("meta".to_string(), "126".to_string());
        key_event_codes.insert("pageup".to_string(), "104".to_string());
        key_event_codes.insert("tab".to_string(), "15".to_string());

        // Test 1: Basic conversion with keys mapped to keycodes
        let shortcut = "Meta+S Tab";
        let expected = Some("126.1 s.1 s.0 126.0 15.1 15.0".to_string());
        let result = convert_shortcut_to_keycode(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 2: Test with characters (e.g., numbers or symbols)
        let shortcut = "123!@# tab ABC";
        let expected = Some("[STR]+ 123!@#[STR] 15.1 15.0 [STR]+ ABC[STR]".to_string());
        let result = convert_shortcut_to_keycode(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 3: Test with more complex shortcuts (e.g., multiple key combinations)
        let shortcut = "meta+pageup tab 123!@# meta+tab";
        let expected = Some(
            "126.1 104.1 104.0 126.0 15.1 15.0 [STR]+ 123!@#[STR] 126.1 15.1 15.0 126.0"
                .to_string(),
        );
        let result = convert_shortcut_to_keycode(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 4: Test with unrecognized keys (e.g., no mapping for 'enter')
        let shortcut = "enter tab";
        let expected = Some("[STR]+ enter[STR] 15.1 15.0".to_string());
        let result = convert_shortcut_to_keycode(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 5: Test with additional '+' combinations
        let shortcut = "meta+tab+pageup";
        let expected = Some("126.1 15.1 104.1 104.0 15.0 126.0".to_string());
        let result = convert_shortcut_to_keycode(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 6: Test empty input
        let shortcut = "";
        let expected = Some("".to_string());
        let result = convert_shortcut_to_keycode(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 7: Test plus with space
        let shortcut = "a + b + c";
        let expected = Some("a.1 a.0 +.1 +.0 b.1 b.0 +.1 +.0 c.1 c.0".to_string());
        let result = convert_shortcut_to_keycode(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);

        // Test 8: Test [STR]
        let shortcut = "meta+pageup tab [STR]+ 123! @# [STR] meta+tab";
        let expected = Some(
            "126.1 104.1 104.0 126.0 15.1 15.0 [STR]+ 123! @#[STR] 126.1 15.1 15.0 126.0"
                .to_string(),
        );
        let result = convert_shortcut_to_keycode(shortcut, &key_event_codes);
        assert_eq!(Some(result), expected);
    }
}