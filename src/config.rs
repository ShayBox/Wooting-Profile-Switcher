use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Seek, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Rule {
    pub app_name: Option<String>,
    pub process_name: Option<String>,
    pub process_path: Option<String>,
    pub profile_index: u8,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub fallback_profile_index: Option<u8>,
    pub loop_sleep_ms: u64,
    pub rules: Vec<Rule>,
    pub send_sleep_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fallback_profile_index: None,
            loop_sleep_ms: 250,
            send_sleep_ms: 250,
            rules: vec![
                Rule {
                    app_name: None,
                    process_name: Some(String::from("Isaac")),
                    process_path: None,
                    profile_index: 1,
                    title: None,
                },
                Rule {
                    app_name: None,
                    process_name: Some(String::from("isaac-ng.exe")),
                    process_path: None,
                    profile_index: 2,
                    title: None,
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
        let mut file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let mut text = String::new();
        file.read_to_string(&mut text)?;
        file.rewind()?;

        match serde_json::from_str(&text) {
            Ok(config) => Ok(config),
            Err(_) => {
                let config = Config::default();
                let text = serde_json::to_string_pretty(&config)?;
                file.write_all(text.as_bytes())?;

                Ok(config)
            }
        }
    }
}
