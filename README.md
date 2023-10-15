<div align="center">
  <a href="https://discord.shaybox.com">
    <img alt="Discord" src="https://img.shields.io/discord/824865729445888041?color=404eed&label=Discord&logo=Discord&logoColor=FFFFFF">
  </a>
  <a href="https://github.com/shaybox/wooting-profile-switcher/releases/latest">
    <img alt="Downloads" src="https://img.shields.io/github/downloads/shaybox/wooting-profile-switcher/total?color=3fb950&label=Downloads&logo=github&logoColor=FFFFFF">
  </a>
</div>

# Wooting Profile Switcher

Automatically switch Wooting keyboard profiles based on focused window

## Installation

[Download the latest release](https://github.com/ShayBox/Wooting-Profile-Switcher/releases/latest)

## Screenshots

![MainApp](https://github.com/ShayBox/Wooting-Profile-Switcher/assets/9505196/2dabd348-2b5c-49b1-8a51-e9cc3fcdf6a9)

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
  // Auto launch at startup
  "auto_launch": null,
  // Auto update at startup
  "auto_update": null,
  // List of connected devices, their serial number properties, and profile names
  "devices": {
    "A02B2106W031H00418": {
      "model_name": "Wooting Two LE",
      "supplier": 2,
      "year": 21,
      "week": 6,
      "product": 3,
      "revision": 1,
      "product_id": 418,
      "production": true,
      "profiles": [
        "Typing Profile",
        "Rapid Profile",
        "Racing Profile",
        "Mixed Movement"
      ]
    }
  },
  // Sleep duration for the loop checking the active window
  "loop_sleep_ms": 250,
  // Sleep duration between sending Wooting USB commands
  "send_sleep_ms": 250,
  // Show the serial number instead of the model name
  "show_serial": false,
  // Swap the lighting effects with the keyboard profile
  "swap_lighting": true,
  // List of rule objects, all match rules support Wildcard and Regex
  "rules": [
    {
      "alias": "The Binding of Isaac",
      "device_indices": {
        "A02B2106W031H00418": 0
      },
      "match_app_name": null,
      "match_bin_name": null,
      "match_bin_path": "C:\\Program Files (x86)\\Steam\\steamapps\\common\\The Binding of Isaac Rebirth*",
      "match_win_name": null
    },
    {
      "alias": "Default Fallback",
      "device_indices": {
        "A02B2106W031H00418": 0
      },
      "match_app_name": "*",
      "match_bin_name": "*",
      "match_bin_path": "*",
      "match_win_name": "*"
    }
  ],
  "ui": {
    "scale": 1.25,
    "theme": "Dark"
  }
}
```

### Examples:

#### Matching a window title with a date variable

```json5
{
    "alias": "VRCX",
    "match_app_name": null,
    "match_bin_name": null,
    "match_bin_path": null,
    "match_win_name": "VRCX ????.??.??",
    "device_indices": {
      "A02B2106W031H00418": 0
    },
}
```

#### Matching a window title with a version variable

```json5
{
    "alias": "Minecraft",
    "match_app_name": null,
    "match_bin_name": null,
    "match_bin_path": null,
    "match_win_name": "Minecraft [\d]+.[\d]+.[\d]+",
    "device_indices": {
      "A02B2106W031H00418": 0
    },
}
```
