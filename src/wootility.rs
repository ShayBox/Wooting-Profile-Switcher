use std::{borrow::Cow, path::PathBuf};

use anyhow::{anyhow, bail, Result};
use encoding_rs::{UTF_16LE, WINDOWS_1252};
use rusty_leveldb::{compressor::SnappyCompressor, CompressorId, Options, DB};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{json::JsonString, serde_as};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Color {
    pub blue: i64,
    pub green: i64,
    pub red: i64,
}

// This isn't exactly pretty but it reduces a lot of duplicated code
structstruck::strike! {
    #[strikethrough[serde_as]]
    #[strikethrough[derive(Clone, Debug, Deserialize, Serialize)]]
    #[strikethrough[serde(rename_all = "camelCase")]]
    pub struct Wootility {
        #[serde(rename = "_persist")]
        #[serde_as(as = "JsonString")]
        pub persist: struct {
            pub rehydrated: bool,
            pub version: i64,
        },
        #[serde_as(as = "JsonString")]
        pub profiles: struct {
            pub current: struct {
                pub device: i64,
            },
            pub device: Vec<pub struct {
                pub actuation_points: Vec<Vec<Value>>,
                pub akc: Vec<Value>,
                pub cfg: Value,
                pub colors: Value,
                pub details: struct {
                    pub name: String,
                    pub uuid: String,
                },
                pub gamepad: Value,
                pub keyboard_layout: Option<Value>,
            }>,
            pub inactive: Vec<Value>,
        },
        #[serde_as(as = "JsonString")]
        pub wootility_config: struct {
            pub active_theme: i64,
            pub default_language: String,
            pub preset_colors: Vec<Color>,
            pub show_intro_page: bool,
            pub show_update_notes: bool,
            pub start_wootility_on_startup: bool,
            pub version: i64,
            pub wooting_service_enabled: bool,
        },
    }
}

impl Wootility {
    pub fn get_path() -> Result<PathBuf> {
        ["", "-beta", "-alpha"]
            .into_iter()
            .map(|path| format!("wootility-lekker{path}/Local Storage/leveldb"))
            .map(|path| dirs::config_dir().unwrap().join(path))
            .find(|path| path.exists())
            .ok_or(anyhow!("Couldn't find Wootility path"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_path()?;
        let opts = Options {
            compressor: SnappyCompressor::ID,
            create_if_missing: false,
            paranoid_checks: true,
            ..Default::default()
        };

        let mut db = DB::open(path, opts)?;

        const KEY: &[u8; 22] = b"_file://\x00\x01persist:root";
        let encoded = db.get(KEY).ok_or(anyhow!("Couldn't find Wootility data"))?;
        let decoded = Self::decode_string(&encoded)?;

        Ok(serde_json::from_str(&decoded)?)
    }

    /// https://github.com/cclgroupltd/ccl_chrome_indexeddb
    pub fn decode_string(bytes: &[u8]) -> Result<Cow<'_, str>> {
        let prefix = bytes.first().ok_or(anyhow!("Invalid length"))?;
        match prefix {
            0 => Ok(UTF_16LE.decode(&bytes[1..]).0),
            1 => Ok(WINDOWS_1252.decode(&bytes[1..]).0),
            _ => bail!("Invalid prefix"),
        }
    }
}
