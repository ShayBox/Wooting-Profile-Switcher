use std::{
    fs::File,
    io::{Read, Seek, Write},
    path::PathBuf,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

const CARGO_CRATE_NAME: &str = env!("CARGO_CRATE_NAME");

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Process {
    pub process_name: String,
    pub profile_index: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub loop_sleep_ms: u64,
    pub send_sleep_ms: u64,
    pub process_list: Vec<Process>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            loop_sleep_ms: 250,
            send_sleep_ms: 250,
            process_list: vec![
                Process {
                    process_name: String::from("Isaac"),
                    profile_index: 1,
                },
                Process {
                    process_name: String::from("isaac-ng.exe"),
                    profile_index: 2,
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
