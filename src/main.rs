// Disable the Windows Command Prompt for release builds
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{ffi::OsStr, process::exit, time::Duration};

use active_win_pos_rs::{get_active_window, ActiveWindow};
use wooting_rgb_sys::{
    wooting_rgb_device_info,
    wooting_rgb_kbd_connected,
    wooting_rgb_reset,
    wooting_usb_send_feature,
    wooting_usb_send_feature_with_response,
};

use crate::config::Config;

mod config;

// https://gist.github.com/BigBrainAFK/0ba454a1efb43f7cb6301cda8838f432
const RELOAD_PROFILE: u8 = 7;
const GET_CURRENT_KEYBOARD_PROFILE_INDEX: u8 = 11;
const ACTIVATE_PROFILE: u8 = 23;
const REFRESH_RGB_COLORS: u8 = 29;
const WOOT_DEV_RESET_ALL: u8 = 32;

fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|_| unsafe {
        wooting_rgb_reset();
    }));

    let config = Config::load()?;
    let device_info = unsafe {
        if !wooting_rgb_kbd_connected() {
            println!("Keyboard not connected.");
            exit(1)
        }

        *wooting_rgb_device_info()
    };

    let mut last_active_window: ActiveWindow = Default::default();
    let mut last_profile_index = unsafe {
        let len = u8::MAX as usize + 1;
        let mut buf = vec![0u8; len];
        let response = wooting_usb_send_feature_with_response(
            buf.as_mut_ptr(),
            len,
            GET_CURRENT_KEYBOARD_PROFILE_INDEX,
            0,
            0,
            0,
            0,
        );

        if response == len as i32 {
            buf[if device_info.v2_interface { 5 } else { 4 }]
        } else {
            u8::MAX
        }
    };

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

        unsafe {
            wooting_usb_send_feature(ACTIVATE_PROFILE, 0, 0, 0, process.profile_index);
            std::thread::sleep(Duration::from_millis(config.send_sleep_ms));
            wooting_usb_send_feature(RELOAD_PROFILE, 0, 0, 0, process.profile_index);
            std::thread::sleep(Duration::from_millis(config.send_sleep_ms));
            wooting_usb_send_feature(WOOT_DEV_RESET_ALL, 0, 0, 0, 0);
            std::thread::sleep(Duration::from_millis(config.send_sleep_ms));
            wooting_usb_send_feature(REFRESH_RGB_COLORS, 0, 0, 0, process.profile_index);
        }
    }
}
