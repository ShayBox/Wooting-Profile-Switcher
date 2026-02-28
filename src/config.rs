use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use wooting_profile_switcher as wps;
use wps::{Device, DeviceIndices, DeviceSerial};

use crate::theme::Theme;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Rule {
    pub alias:          String,
    pub device_indices: DeviceIndices,
    #[serde(alias = "app_name")]
    pub match_app_name: Option<String>,
    #[serde(alias = "process_name")]
    pub match_bin_name: Option<String>,
    #[serde(alias = "process_path")]
    pub match_bin_path: Option<String>,
    #[serde(alias = "title")]
    pub match_win_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Ui {
    pub frames: u64,
    pub scale:  f32,
    pub theme:  Theme,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            frames: 60,
            scale:  1.25,
            theme:  Theme::Dark,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub auto_launch: Option<bool>,
    pub auto_update: Option<bool>,
    pub devices: HashMap<DeviceSerial, Device>,
    pub loop_sleep_ms: u64,
    pub send_sleep_ms: u64,
    pub show_serial: bool,
    pub swap_lighting: bool,
    pub rules: Vec<Rule>,
    pub ui: Ui,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_launch: None,
            auto_update: None,
            devices: HashMap::new(),
            loop_sleep_ms: 250,
            send_sleep_ms: 250,
            show_serial: false,
            swap_lighting: true,
            rules: vec![
                Rule {
                    alias: String::from("The Binding of Isaac"),
                    device_indices: DeviceIndices::new(),
                    match_app_name: None,
                    match_bin_name: None,
                    match_bin_path: Some(String::from("C:\\Program Files (x86)\\Steam\\steamapps\\common\\The Binding of Isaac Rebirth*")),
                    match_win_name: None,
                },
                Rule {
                    alias: String::from("Default Fallback"),
                    device_indices: wps::get_device_indices().unwrap_or_default(),
                    match_app_name: Some(String::from("*")),
                    match_bin_name: Some(String::from("*")),
                    match_bin_path: Some(String::from("*")),
                    match_win_name: Some(String::from("*")),
                }
            ],
            ui: Ui::default(),
        }
    }
}

impl Config {
    pub fn get_path() -> Result<PathBuf> {
        let mut path = std::env::current_exe()?;
        path.set_extension("json");

        let config_path = if cfg!(debug_assertions) {
            path
        } else {
            let Some(file_name) = path.file_name().and_then(OsStr::to_str) else {
                bail!("Could not get current executable file name")
            };

            let Some(path) = dirs::config_dir() else {
                bail!("Could not get config directory path");
            };

            path.join(file_name)
        };

        Ok(config_path)
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_path()?;
        let config = if let Ok(mut file) = File::open(&path) {
            let mut text = String::new();
            file.read_to_string(&mut text)?;

            serde_json::from_str(&text).unwrap_or_else(|error| {
                eprintln!("There was an error parsing the config: {error}");
                eprintln!("Temporarily using the default config");
                Self::default()
            })
        } else {
            if path.exists() {
                // Rename the existing config file
                let new_path = path.join(".bak");
                std::fs::rename(&path, &new_path)?;
                eprintln!("Config file renamed to: {}", new_path.display());
            }

            // Create a new config file and write default config
            let mut file = File::create(&path)?;
            let config = Self::default();
            let text = serde_json::to_string_pretty(&config)?;
            file.write_all(text.as_bytes())?;
            config
        };

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_path()?;
        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let content = serde_json::to_string_pretty(&self)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }
}
