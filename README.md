<div align="center">
  <a href="https://discord.shaybox.com">
    <img alt="Discord" src="https://img.shields.io/discord/824865729445888041?color=404eed&label=Discord&logo=Discord&logoColor=FFFFFF">
  </a>
  <a href="https://github.com/shaybox/wootingprofileswitcher/releases/latest">
    <img alt="Downloads" src="https://img.shields.io/github/downloads/shaybox/wootingprofileswitcher/total?color=3fb950&label=Downloads&logo=github&logoColor=FFFFFF">
  </a>
</div>

# WootingProfileSwitcher

Automatically switch Wooting keyboard profiles based on focused window

## Installation

[Download the latest release](https://github.com/ShayBox/WootingProfileSwitcher/releases/latest)

## System Tray Icon

The system tray icon allows you to pause/resume, reload, quit, and set the active profile

## Configuration

The config file is generated on first-run in the following location and format

| Platform | Location                                 |
|----------|------------------------------------------|
| Portable | Same location as the binary              |
| Windows  | `C:\Users\...\AppData\Roaming`           |
| macOS    | `/Users/.../Library/Application Support` |
| Linux    | `/home/.../.config`                      |

```json
{
    // The fallback profile to use when no match is found (optional)
    "fallback_profile_index": null,
    // Sleep duration for the loop checking the active window
    "loop_sleep_ms": 250,
    // Sleep duration between sending Wooting USB commands
    "send_sleep_ms": 250,
    // List of rule objects
    "rules": [
        {
            // The official app name (optional)
            "app_name": null,
            // The running process name (optional)
            "process_name": "Isaac",
            // The running process path (optional)
            "process_path": null,
            // The profile to switch to when a match is found for this rule (0-3)
            "profile_index": 1,
            // The running window title (optional)
            "title": null
        },
        {
            "app_name": null,
            "process_name": "isaac-ng.exe",
            "process_path": null,
            "profile_index": 2,
            "title": null
        }
    ]
}
```

### Examples:

#### Matching a window title with a date variable

```json
{
    "app_name": null,
    "process_name": null,
    "process_path": null,
    "profile_index": 0,
    "title": "VRCX ????.??.??"
}
```

#### Matching a window title with a version variable

```json
{
    "app_name": null,
    "process_name": null,
    "process_path": null,
    "profile_index": 0,
    "title": "Minecraft [\d]+.[\d]+.[\d]+"
}
```

[Wildcard]: https://crates.io/crates/wildflower
[Regex]: https://crates.io/crates/regex
