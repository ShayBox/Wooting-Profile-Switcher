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
| -------- | ---------------------------------------- |
| Portable | Same location as the binary              |
| Windows  | `C:\Users\...\AppData\Roaming`           |
| macOS    | `/Users/.../Library/Application Support` |
| Linux    | `/home/.../.config`                      |

```json5
{
    // The fallback profile to use when no match is found (optional)
    "fallback_profile_index": null,
    // Sleep duration for the loop checking the active window
    "loop_sleep_ms": 250,
    // Sleep duration between sending Wooting USB commands
    "send_sleep_ms": 250,
    // Swap the lighting effects with the keyboard profile
    "swap_lighting": true,
    // List of profile names, pulled from Wootility
    "profiles" [
      "Typing Profile",
      "Rapid Profile",
      "Racing Profile",
      "Mixed Movement",
    ],
    // List of rule objects, all match rules support Wildcard and Regex
    "rules": [
        {
            // Match against the official app name (optional)
            "app_name": null,
            // Match against the running process name (optional)
            "process_name": "Isaac",
            // Match against the running process path (optional)
            "process_path": null,
            // Match against the running window title (optional)
            "title": null,
            // The profile to switch to when a match is found for this rule (0-3)
            "profile_index": 1
        },
        {
            "app_name": null,
            "process_name": "isaac-ng.exe",
            "process_path": null,
            "title": null,
            "profile_index": 2
        }
    ]
}
```

### Examples:

#### Matching a window title with a date variable

```json5
{
    "app_name": null,
    "process_name": null,
    "process_path": null,
    "title": "VRCX ????.??.??",
    "profile_index": 0
}
```

#### Matching a window title with a version variable

```json5
{
    "app_name": null,
    "process_name": null,
    "process_path": null,
    "title": "Minecraft [\d]+.[\d]+.[\d]+",
    "profile_index": 0
}
```

[Wildcard]: https://crates.io/crates/wildflower
[Regex]: https://crates.io/crates/regex
