use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Seek, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Rule {
    pub alias:          String,
    #[serde(rename = "app_name")]
    pub match_app_name: Option<String>,
    #[serde(rename = "process_name")]
    pub match_bin_name: Option<String>,
    #[serde(rename = "process_path")]
    pub match_bin_path: Option<String>,
    #[serde(rename = "title")]
    pub match_win_name: Option<String>,
    pub profile_index:  u8,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Ui {
    pub scale: f32,
    pub theme: Theme,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub auto_launch: Option<bool>,
    pub auto_update: Option<bool>,
    pub fallback_profile_index: Option<u8>,
    pub loop_sleep_ms: u64,
    pub send_sleep_ms: u64,
    pub swap_lighting: bool,
    pub profiles: Vec<String>,
    pub rules: Vec<Rule>,
    pub ui: Ui,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_launch: None,
            auto_update: None,
            fallback_profile_index: Some(0),
            loop_sleep_ms: 250,
            send_sleep_ms: 250,
            swap_lighting: true,
            profiles: vec![
                String::from("Typing Profile"),
                String::from("Rapid Profile"),
                String::from("Racing Profile"),
                String::from("Mixed Movement"),
            ],
            rules: vec![Rule {
                alias: String::from("The Binding of Isaac"),
                match_app_name: Some(String::from("isaac-ng")),
                match_bin_name: Some(String::from("isaac-ng.exe")),
                match_bin_path: Some(String::from("C:\\Program Files (x86)\\Steam\\steamapps\\common\\The Binding of Isaac Rebirth\\isaac-ng.exe")),
                match_win_name: Some(String::from("Binding of Isaac: Repentance")),
                profile_index: 1,
            }],
            ui: Ui {
                scale: 1.25,
                theme: Theme::Dark,
            },
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
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => {
                let mut file = File::create(&path)?;
                let config = Config::default();
                let text = serde_json::to_string_pretty(&config)?;
                file.write_all(text.as_bytes())?;

                return Ok(config);
            }
        };

        let mut text = String::new();
        file.read_to_string(&mut text)?;
        file.rewind()?;

        let config = match serde_json::from_str(&text) {
            Ok(config) => config,
            Err(error) => {
                eprintln!("There was an error parsing the config: {error}");
                eprintln!("Temporarily using the default config");

                Self::default()
            }
        };

        Ok(config)
    }

    pub fn save(&mut self) -> Result<()> {
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
