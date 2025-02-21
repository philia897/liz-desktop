<div align="center">
  <img src="assets/icon_1024.png" width="100"/>
</div>

# liz-desktop

A Rust-based shortcut helper to remember, customize and autorun shortcuts or commands. Developed via Tauri 2.0.

- Windows ☑️
- Linux ☑️
- Mac ✘

![demo](./assets/demo.gif)

## Features

- **Fuzzy search:** Search by description, application name or shortcut keys.
- **Auto-execution:** Use [enigo](https://github.com/enigo-rs/enigo) to simulate execution of the selected shortcut.
- **Shortcut/Typing:** Liz supports:
    - Shortcut: `ctrl+c` 
    - Typing a string: `Liz and the Blue Bird` 
    - Hybrid: `esc [STR]+ Liz and the Blue Bird`
- **Dark/Light mode:** Following the system
- **Dynamic rank:** rank the shortcuts according to the frequency. The most frequently used shortcuts will be on the top.
- **Customization:** Support adding/importing/sharing/managing the shortcuts via json files.

> You can see an example of **sheet** [here](./data/sheets/examples.json), which denotes the json file that defines a bunch of shortcuts. In the example it shows how to add different types of shortcut commands. In the `data/sheets` you can find other sheets I created and feel free to have a try.

## Usage

### Installation

Please check the [release](https://github.com/philia897/liz-desktop/releases) and download the packages to install.

In the first run, Liz will create its data dir `liz_path` automatically with the default config path, which will be:

- **Windows:** `%APPDATA%\liz`, such as: `C:\Users\<YourUsername>\AppData\Roaming\liz`
- **Linux:** `$HOME/.config/liz`, such as: `/home/<YourUsername>/.config/liz`

> It can also be customized by setting the environment variable: `LIZ_DATA_DIR`

### Quick Start

After installation and start the application, you should see an blank list of shortcuts.

Following the [example](./data/sheets/examples.json) to create your own `sheet` or copy the examples [here](./data/sheets/). Put the sheets under `<liz_path>/sheets`. If the folder does not exists, create it.

Then right click the tray and click `reload` to load the sheets in `<liz_path>/sheets`. click `show` to see the loaded shortcut list.

> Liz will not reload the sheets stored in `<liz_path>/sheets` until user click `reload`.

> The tray manu also have `persist`, which will persist the data to a .lock file immediately. Liz will auto persist when the program exits.

### Configuration

Liz has these configuration options:

- **`liz_path:`**  
  _Path of Liz data directory_  
  This is the main directory where Liz stores its data. By default, it's set to the application’s config folder, explained above.

- **`user_sheets_path:`**  
  _Path for all the shortcut sheets_  
  This is the directory where user-defined shortcut sheets are stored. The default path is `<liz_path>/sheets`.

- **`music_sheet_path:`**  
  _Path for the lock file for Bluebird_  
  The file used as the lock file for Liz, performs as a database. The default path is `<liz_path>/music_sheet.lock`.

- **`keymap_path:`**  
  _Path to the keymap file_  
  The path to the keymap configuration file. This file stores the customized key mappings for the application. The default path is `<liz_path>/keymap_builtin.json`.

- **`interval_ms:`**  
  _Interval for each shortcut block (in milliseconds)_  
  This is the time interval (in milliseconds) for each shortcut block. Normally, you don't need to change this. The default value is **100 milliseconds**.

- **`trigger_shortcut:`**  
  _Shortcut key to trigger a specific action_  
  This is the default keyboard shortcut used to trigger a specific action in Liz. By default, it is set to `Ctrl+Alt+L`.

You can control the Liz configuration via any of the following ways:

- write a `rhythm.toml` file following this [example](./data/rhythm.toml) and use it by `liz -c /path/to/your/rhythm.toml`.

- write a `rhythm.toml` file and put it under default `<liz_path>/rhythm.toml`. Liz will automatically use it.


> According to the [doc of enigo](https://github.com/enigo-rs/enigo#), For linux users you'd better to install these tools for X11 support:
> 
> Debian-based: `apt install libxdo-dev`
>
> Arch: `pacman -S xdotool`
>
> Fedora: `dnf install libX11-devel libxdo-devel`
>
> Gentoo: `emerge -a xdotool`






## Future plan

- Add a control panel for app settings and shortcut management.
- Add Arch support (Already created a repo but have not fully test the PKGBUILD)
- Add Mac support (It theoretically works, but I have not tested it yet. No Mac equipment)
- Using tauri plugins to remember window position and size.
- Using tauri plugin for logging.
- Try SQLite instead of the json lock file for data persistence.
- Find way to reduce the memory cost, (maybe provide a solution to use external tools like [Rofi](https://github.com/davatorium/rofi/))
- ...

## Credits & License

- Thanks to the wonderful projects [Tauri 2.0](https://tauri.app/) and [Enigo](https://github.com/enigo-rs/enigo).
- License: [GPL-3](./LICENSE)

