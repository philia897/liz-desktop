<div align="center">
  <img src="assets/icon_1024.png" width="100"/>
  <h2>便捷优雅的快捷键助手</h2>
</div>

# liz-desktop

[English](./README.md) [中文](./README_zh.md)

一个基于 Rust 的快捷键助手，用于记忆、自定义和自动运行快捷键或命令。通过 Tauri 2.0 开发。

- Windows ☑️
- Linux ☑️
- Mac ✘

![demo](./assets/demo.gif)

## 功能

- **模糊搜索：** 通过描述、应用程序名称或快捷键进行搜索。
- **自动执行：** 使用 [enigo](https://github.com/enigo-rs/enigo) 模拟执行选定的快捷键。
- **快捷键/输入：** Liz 支持：
    - 快捷键：`ctrl+c`
    - 输入字符串：`Liz and the Blue Bird`
    - 混合模式：`esc [STR]+ Liz and the Blue Bird`
- **暗黑/亮色模式：** 跟随系统设置
- **动态排名：** 根据使用频率对快捷键进行排名。最常用的快捷键将排在顶部。
- **快捷键管理器：** 内置漂亮的配置面板，用于管理快捷键
- **导入/导出：** 支持通过 json/txt 文件导入/导出快捷键。

> 你可以在这里查看 **sheet** 的[示例](./data/sheets/examples.json)，它定义了多个快捷键的 json 文件。示例中展示了如何添加不同类型的快捷键命令。在 `data/sheets` 目录下，你可以找到我创建的其他 sheet，欢迎尝试。

> 这个 [Python 脚本](./scripts/parse_shortcuts.py) 可以解析 [cheatsheets.zip](https://cheatsheets.zip/) 中的键盘快捷键，提取 markdown 文件中的快捷键（点击顶部栏的 github 图标下载原始 markdown 文件），并生成可以导入 Liz 的 json 文件。

## 使用方法

### 安装

请查看 [release](https://github.com/philia897/liz-desktop/releases) 并下载安装包进行安装。

> Arch [AUR](https://aur.archlinux.org/packages/liz-desktop-bin): `paru -S liz-desktop-bin`

### 快速开始

 - 下载快捷键 sheet 示例 [这里](./data/sheets/)。
 - 通过托盘菜单 `Config` 打开配置面板。右键点击表格并选择 `Import` 导入下载的 json sheet。
 - 点击托盘菜单 `Show` 激活 Liz 并开始使用。

> 你也可以使用 `trigger_shortcut` 来 `Show` Liz，默认快捷键是 `Ctrl+Alt+L`。

> 托盘菜单中的 `Persist` 选项会立即将数据保存到 .lock 文件中。Liz 会在程序退出时自动保存数据。

> 托盘菜单中的 `Reload` 选项表示如果快捷键没有正确添加到 Liz 中，可以重新加载 Liz 主界面。

### 配置

你可以通过以下方式控制 Liz 的配置：

- （推荐）使用内置的配置面板：点击左上角的按钮并选择 `Settings`。

- 按照这个 [示例](./data/rhythm.toml) 编写你自己的 `rhythm.toml` 文件，并通过 `liz -c /path/to/your/rhythm.toml` 使用它。

- 编写一个 `rhythm.toml` 文件并将其放在默认的 `<liz_path>/rhythm.toml` 路径下。Liz 会自动使用它。

> 根据 [enigo 的文档](https://github.com/enigo-rs/enigo#)，对于 Linux 用户，建议安装以下工具以支持 X11：
> 
> Debian 系：`apt install libxdo-dev`
>
> Arch：`pacman -S xdotool`
>
> Fedora：`dnf install libX11-devel libxdo-devel`
>
> Gentoo：`emerge -a xdotool`

## 故障排除

> 如果遇到任何错误或问题，请创建一个 issue

在首次运行时，Liz 会自动创建其数据目录 `liz_path`，并带有默认的配置文件路径，路径如下：

- **Windows：** `%APPDATA%\liz`，例如：`C:\Users\<YourUsername>\AppData\Roaming\liz`
- **Linux：** `$HOME/.config/liz`，例如：`/home/<YourUsername>/.config/liz`

> 也可以通过设置环境变量 `LIZ_DATA_DIR` 来自定义路径。

要重置设置为默认值，只需删除文件 `<liz_path>/rhythm.toml`，或在配置面板中清除值。

## 致谢与许可证

- 感谢优秀的项目 [Tauri 2.0](https://tauri.app/)、[Enigo](https://github.com/enigo-rs/enigo) 和 [Reference](https://github.com/Fechin/reference/tree/main)。
- 许可证：[GPL-3](./LICENSE)