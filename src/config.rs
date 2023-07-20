use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Seek, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Rule {
    pub app_name: Option<String>,
    pub process_name: Option<String>,
    pub process_path: Option<String>,
    pub title: Option<String>,
    pub profile_index: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub fallback_profile_index: Option<u8>,
    pub loop_sleep_ms: u64,
    pub send_sleep_ms: u64,
    pub swap_lighting: bool,
    pub rules: Vec<Rule>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fallback_profile_index: Some(0),
            loop_sleep_ms: 250,
            send_sleep_ms: 250,
            swap_lighting: true,
            rules: vec![
                Rule {
                    app_name: None,
                    process_name: Some(String::from("Isaac")),
                    process_path: None,
                    title: None,
                    profile_index: 1,
                },
                Rule {
                    app_name: None,
                    process_name: Some(String::from("isaac-ng.exe")),
                    process_path: None,
                    title: None,
                    profile_index: 2,
                },
            ],
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
}
