<div align="center">
  <img src="assets/icon_1024.png" width="100"/>
  <h2>Remember and Execute shortcuts for you</h2>
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
- **Shortcut manager:** Has a builtin pretty config panel for managing shortcuts
- **Import/Export:** Support importing/exporting the shortcuts via json/txt files.

> You can see an example of **sheet** [here](./data/sheets/examples.json), which denotes the json file that defines a bunch of shortcuts. In the example it shows how to add different types of shortcut commands. In the `data/sheets` you can find other sheets I created and feel free to have a try.

## Usage

### Installation

Please check the [release](https://github.com/philia897/liz-desktop/releases) and download the packages to install.

> Arch [AUR](https://aur.archlinux.org/packages/liz-desktop-bin): `paru -S liz-desktop-bin`

### Quick Start

 - Download shortcut sheet examples [here](./data/sheets/).
 - Open the config panel via tray menu `Config`. Right click the table to `Import` the downloaded json sheets.
 - Click tray menu `Show` to activate Liz and enjoy.

> You can use a `trigger_shortcut` to `Show` liz as well, the shortcut is `Ctrl+Alt+L` by default.

> The tray menu also have `Persist`, which will persist the data to a .lock file immediately. Liz will auto persist when the program exits.

> Tray menu option `Reload` means reload Liz main view if shortcuts are not added to Liz correctly.

### Configuration

You can control the Liz configuration via any of the following ways:

- (Recommand) Use the builtin Config panel: Click the top-left button and choose `Settings`.

- write your own `rhythm.toml` file following this [example](./data/rhythm.toml) and use it by `liz -c /path/to/your/rhythm.toml`.

- write a `rhythm.toml` file and put it under default `<liz_path>/rhythm.toml`. Liz will automatically use it.

> According to the [doc of enigo](https://github.com/enigo-rs/enigo#), For Linux users you'd better to install these tools for X11 support:
> 
> Debian-based: `apt install libxdo-dev`
>
> Arch: `pacman -S xdotool`
>
> Fedora: `dnf install libX11-devel libxdo-devel`
>
> Gentoo: `emerge -a xdotool`


## TroubleShooting

> Please create an issue if encounter any bug or error

In the first run, Liz will create its data dir `liz_path` automatically with the default config path, which will be:

- **Windows:** `%APPDATA%\liz`, such as: `C:\Users\<YourUsername>\AppData\Roaming\liz`
- **Linux:** `$HOME/.config/liz`, such as: `/home/<YourUsername>/.config/liz`

> It can also be customized by setting the environment variable `LIZ_DATA_DIR`.

To reset settings to default, simply delete the file `<liz_path>/rhythm.toml`, or clear the values in the config panel.

## Future plan

- Add Mac support (It theoretically works, but I have not tested it yet. No Mac equipment)
- Using tauri plugins to remember window position and size.
- Using tauri plugin for logging.
- Try SQLite instead of the json lock file for data persistence.
- Find way to reduce the memory cost, (maybe provide a solution to use external tools like [Rofi](https://github.com/davatorium/rofi/))
- ...

## Credits & License

- Thanks to the wonderful projects [Tauri 2.0](https://tauri.app/) and [Enigo](https://github.com/enigo-rs/enigo).
- License: [GPL-3](./LICENSE)

