// Disable the Windows Command Prompt for release builds
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{ffi::OsStr, process::exit, time::Duration};

use active_win_pos_rs::{get_active_window, ActiveWindow};
use clap::Parser;
use wooting_profile_switcher::{get_active_profile_index, set_active_profile_index};
use wooting_rgb_sys::{wooting_rgb_device_info, wooting_rgb_kbd_connected, wooting_rgb_reset};

use crate::config::Config;

mod config;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Set the active profile index and exit.
    /// Can be useful for automation scripting.
    #[arg(short, long)]
    profile_index: Option<u8>,
}

fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|_| unsafe {
        wooting_rgb_reset();
    }));

    let args = Args::parse();
    let config = Config::load()?;
    let device_info = unsafe {
        if !wooting_rgb_kbd_connected() {
            println!("Keyboard not connected.");
            exit(1)
        }

        *wooting_rgb_device_info()
    };

    if let Some(profile_index) = args.profile_index {
        set_active_profile_index(profile_index, config.send_sleep_ms);
        return Ok(());
    }

    let mut last_active_window: ActiveWindow = Default::default();
    let mut last_profile_index = get_active_profile_index(device_info);

    loop {
        std::thread::sleep(Duration::from_millis(config.loop_sleep_ms));
        let Ok(active_window) = get_active_window() else {
            continue;
        };

        if active_window == last_active_window {
            continue;
        } else {
            last_active_window = active_window.to_owned();
        }

        let Some(active_process_name) = active_window.process_path.file_name().and_then(OsStr::to_str) else {
            continue;
        };

        println!("Active Process Name: {active_process_name}");

        let Some(process) = config
            .process_list
            .iter()
            .find(|process| process.process_name == active_process_name)
        else {
            continue;
        };

        if process.profile_index == last_profile_index {
            continue;
        } else {
            last_profile_index = process.profile_index;
        }

        println!("Process Profile Index: {}", process.profile_index);
        set_active_profile_index(process.profile_index, config.send_sleep_ms);
    }
}
