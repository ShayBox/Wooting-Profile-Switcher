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

You can [Download] and Extract the latest release for your operating system  
You can also install via cargo:  
`$ cargo install --git https://github.com/ShayBox/WootingProfileSwitcher`

## Configuration

The config file is generated on first run with the following format

```json
{
    "fallback_profile_index": null,
    "loop_sleep_ms": 250,
    "send_sleep_ms": 250,
    "rules": [
        {
            "app_name": null,
            "process_name": "Isaac",
            "process_path": null,
            "profile_index": 1,
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

The `fallback_profile_index` variable allows you to set a fallback profile index to use when no match is found.  
The `sleep_ms` variables allow you to customize the duration between checking the active process, and duration between sending Wooting USB commands.  
The `rules` variable is a list of rules that supports [Wildcard] and [Regex] for `app_name`, `process_name`, `process_path` and `title` variables.

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

[Download]: https://github.com/ShayBox/WootingProfileSwitcher/releases/latest
[Wildcard]: https://crates.io/crates/wildflower
[Regex]: https://crates.io/crates/regex
