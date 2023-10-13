use std::{collections::HashMap, time::Duration};

use anyhow::{bail, Result};
use wooting_rgb_sys as rgb;

// https://gist.github.com/BigBrainAFK/0ba454a1efb43f7cb6301cda8838f432
const GET_SERIAL: u8 = 3;
const RELOAD_PROFILE: u8 = 7;
const GET_CURRENT_KEYBOARD_PROFILE_INDEX: u8 = 11;
const ACTIVATE_PROFILE: u8 = 23;
const REFRESH_RGB_COLORS: u8 = 29;
const WOOT_DEV_RESET_ALL: u8 = 32;

pub type DeviceIndices = HashMap<String, u8>;

pub fn get_active_serial_number() -> Result<String> {
    unsafe {
        let len = u8::MAX as usize + 1;
        let mut buf = vec![0u8; len];
        let response = rgb::wooting_usb_send_feature_with_response(
            buf.as_mut_ptr(),
            len,
            GET_SERIAL,
            0,
            0,
            0,
            0,
        );

        if response != len as i32 {
            bail!("Invalid response length");
        }

        let serial_number = format!(
            "A{:02X}B{:02}{:02}W{:02X}{}H{:05}",
            u16::from_le_bytes(buf[5..7].try_into().unwrap()),
            buf[7],
            buf[8],
            u16::from_le_bytes(buf[9..11].try_into().unwrap()),
            if buf[15] == 1 { 0 } else { 1 },
            u16::from_le_bytes(buf[13..15].try_into().unwrap()),
        );

        Ok(serial_number)
    }
}

pub fn get_active_profile_index() -> u8 {
    unsafe {
        let len = u8::MAX as usize + 1;
        let mut buf = vec![0u8; len];
        let response = rgb::wooting_usb_send_feature_with_response(
            buf.as_mut_ptr(),
            len,
            GET_CURRENT_KEYBOARD_PROFILE_INDEX,
            0,
            0,
            0,
            0,
        );

        if response == len as i32 {
            let is_v2 = rgb::wooting_usb_use_v2_interface();
            buf[if is_v2 { 5 } else { 4 }]
        } else {
            u8::MAX
        }
    }
}

pub fn set_active_profile_index(profile_index: u8, send_sleep_ms: u64, swap_lighting: bool) {
    unsafe {
        rgb::wooting_usb_send_feature(ACTIVATE_PROFILE, 0, 0, 0, profile_index);
        std::thread::sleep(Duration::from_millis(send_sleep_ms));
        rgb::wooting_usb_send_feature(RELOAD_PROFILE, 0, 0, 0, profile_index);

        if swap_lighting {
            std::thread::sleep(Duration::from_millis(send_sleep_ms));
            rgb::wooting_usb_send_feature(WOOT_DEV_RESET_ALL, 0, 0, 0, 0);
            std::thread::sleep(Duration::from_millis(send_sleep_ms));
            rgb::wooting_usb_send_feature(REFRESH_RGB_COLORS, 0, 0, 0, profile_index);
        }
    }
}

pub fn get_device_indices() -> Result<DeviceIndices> {
    unsafe {
        rgb::wooting_usb_disconnect(false);
        rgb::wooting_usb_find_keyboard();

        let mut device_indices = DeviceIndices::new();
        for device_index in 0..rgb::WOOTING_MAX_RGB_DEVICES as u8 {
            if !rgb::wooting_usb_select_device(device_index) {
                continue;
            }

            let serial_number = get_active_serial_number()?;
            let profile_index = get_active_profile_index();
            device_indices.insert(serial_number, profile_index);
        }

        Ok(device_indices)
    }
}

pub fn set_device_indices(
    device_indices: DeviceIndices,
    send_sleep_ms: u64,
    swap_lighting: bool,
) -> Result<()> {
    unsafe {
        rgb::wooting_usb_disconnect(false);
        rgb::wooting_usb_find_keyboard();

        for device_index in 0..rgb::WOOTING_MAX_RGB_DEVICES as u8 {
            if !rgb::wooting_usb_select_device(device_index) {
                continue;
            }

            let serial_number = get_active_serial_number()?;
            if let Some(profile_index) = device_indices.get(&serial_number) {
                set_active_profile_index(profile_index.to_owned(), send_sleep_ms, swap_lighting)
            };
        }
    }

    Ok(())
}

pub fn select_device(serial_number: &String) -> Result<bool> {
    unsafe {
        rgb::wooting_usb_disconnect(false);
        rgb::wooting_usb_find_keyboard();

        for device_index in 0..rgb::WOOTING_MAX_RGB_DEVICES as u8 {
            if !rgb::wooting_usb_select_device(device_index) {
                continue;
            }

            let active_serial_number = get_active_serial_number()?;
            if serial_number == &active_serial_number {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
