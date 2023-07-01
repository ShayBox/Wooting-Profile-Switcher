use std::{
    fs::File,
    io::{Read, Seek, Write},
    path::PathBuf,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

const CARGO_CRATE_NAME: &str = env!("CARGO_CRATE_NAME");

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
        path.set_file_name(CARGO_CRATE_NAME);
        path.set_extension("json");

        Ok(path)
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
