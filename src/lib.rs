use std::{collections::HashMap, ffi::CStr, time::Duration};

use anyhow::{bail, Result};
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};
use wooting_rgb_sys as rgb;

/* Constants */

// https://gist.github.com/BigBrainAFK/0ba454a1efb43f7cb6301cda8838f432
const GET_SERIAL: u8 = 3;
const RELOAD_PROFILE: u8 = 7;
const GET_CURRENT_KEYBOARD_PROFILE_INDEX: u8 = 11;
const ACTIVATE_PROFILE: u8 = 23;
const REFRESH_RGB_COLORS: u8 = 29;
const WOOT_DEV_RESET_ALL: u8 = 32;

/* Typings */

pub type ProfileIndex = i8;
pub type DeviceIndices = HashMap<DeviceSerial, ProfileIndex>;

/* Structures */

#[derive(Clone, Debug, Default, Display, Deserialize, Eq, FromStr, Hash, PartialEq, Serialize)]
pub struct DeviceID(String);

#[derive(Clone, Debug, Default, Display, Deserialize, Eq, FromStr, Hash, PartialEq, Serialize)]
pub struct DeviceSerial(String);

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Device {
    pub model_name: String,
    pub supplier:   u16,
    pub year:       u8,
    pub week:       u8,
    pub product:    u16,
    pub revision:   u16,
    pub product_id: u16,
    pub production: bool,
    pub profiles:   Vec<String>,
}

/* Implementations */

impl From<&Device> for DeviceID {
    fn from(device: &Device) -> Self {
        let keyboard_type = match device.model_name.as_str() {
            "Wooting One" => 0,
            "Wooting Two" => 1,
            "Wooting Two LE" => 2,
            "Wooting Two HE" => 3,
            "Wooting 60HE" => 4,
            "Wooting 60HE (ARM)" => 5,
            "Wooting Two HE (ARM)" => 6,
            "Wooting UwU" => 7,
            "Wooting UwU RGB" => 8,
            &_ => 9,
        };

        let device_id = format!(
            "{}{}{}{}{}{}",
            keyboard_type,
            device.product_id,
            device.product,
            device.revision,
            device.week,
            device.year
        );

        Self(device_id)
    }
}

impl From<&Device> for DeviceSerial {
    fn from(device: &Device) -> Self {
        Self(format!(
            "A{:02X}B{:02}{:02}W{:02X}{}H{:05}",
            device.supplier,
            device.year,
            device.week,
            device.product,
            device.production as u8,
            device.product_id
        ))
    }
}

impl TryFrom<DeviceID> for Device {
    type Error = anyhow::Error;

    fn try_from(device_id: DeviceID) -> Result<Self> {
        unsafe {
            rgb::wooting_usb_disconnect(false);
            rgb::wooting_usb_find_keyboard();

            for device_index in 0..rgb::WOOTING_MAX_RGB_DEVICES as u8 {
                if !rgb::wooting_usb_select_device(device_index) {
                    continue;
                }

                let device = get_active_device()?;
                if device_id == DeviceID::from(&device) {
                    return Ok(device);
                }
            }
        }

        bail!("Device ({device_id}) not found")
    }
}

impl TryFrom<DeviceSerial> for Device {
    type Error = anyhow::Error;

    fn try_from(device_serial: DeviceSerial) -> Result<Self> {
        unsafe {
            rgb::wooting_usb_disconnect(false);
            rgb::wooting_usb_find_keyboard();

            for device_index in 0..rgb::WOOTING_MAX_RGB_DEVICES as u8 {
                if !rgb::wooting_usb_select_device(device_index) {
                    continue;
                }

                let device = get_active_device()?;
                if device_serial == DeviceSerial::from(&device) {
                    return Ok(device);
                }
            }
        }

        bail!("Device ({device_serial}) not found")
    }
}

/* Getters */

pub fn get_active_device() -> Result<Device> {
    unsafe {
        if !rgb::wooting_usb_find_keyboard() {
            bail!("Failed to find keyboard")
        };

        let wooting_usb_meta = *rgb::wooting_usb_get_meta();
        let c_str_model_name = CStr::from_ptr(wooting_usb_meta.model);

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

        let device = Device {
            model_name: c_str_model_name.to_str()?.replace("Lekker Edition", "LE"),
            supplier:   u16::from_le_bytes(buf[5..7].try_into()?),
            year:       buf[7],
            week:       buf[8],
            product:    u16::from_le_bytes(buf[9..11].try_into()?),
            revision:   u16::from_le_bytes(buf[11..13].try_into()?),
            product_id: u16::from_le_bytes(buf[13..15].try_into()?),
            production: buf[15] == 0,
            profiles:   Vec::new(),
        };

        Ok(device)
    }
}

pub fn get_all_devices() -> Result<Vec<Device>> {
    let mut devices = Vec::new();

    unsafe {
        rgb::wooting_usb_disconnect(false);
        rgb::wooting_usb_find_keyboard();

        for device_index in 0..rgb::WOOTING_MAX_RGB_DEVICES as u8 {
            if !rgb::wooting_usb_select_device(device_index) {
                continue;
            }

            let device = get_active_device()?;
            devices.push(device);
        }

        rgb::wooting_rgb_reset_rgb();
    }

    Ok(devices)
}

pub fn get_active_profile_index() -> ProfileIndex {
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
            buf[if is_v2 { 5 } else { 4 }] as ProfileIndex
        } else {
            ProfileIndex::MAX
        }
    }
}

pub fn get_device_indices() -> Result<DeviceIndices> {
    let mut device_indices = DeviceIndices::new();

    unsafe {
        rgb::wooting_usb_disconnect(false);
        rgb::wooting_usb_find_keyboard();

        for device_index in 0..rgb::WOOTING_MAX_RGB_DEVICES as u8 {
            if !rgb::wooting_usb_select_device(device_index) {
                continue;
            }

            let device = get_active_device()?;
            let device_serial = DeviceSerial::from(&device);
            let profile_index = get_active_profile_index();
            device_indices.insert(device_serial, profile_index);
        }

        rgb::wooting_rgb_reset_rgb();
    }

    Ok(device_indices)
}

/* Setters */

pub fn set_active_profile_index(
    profile_index: ProfileIndex,
    send_sleep_ms: u64,
    swap_lighting: bool,
) -> Result<()> {
    let profile_index = u8::try_from(profile_index)?;

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

    Ok(())
}

pub fn set_device_indices(
    mut device_indices: DeviceIndices,
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

            let device = get_active_device()?;
            let device_serial = DeviceSerial::from(&device);
            if let Some(profile_index) = device_indices.remove(&device_serial) {
                // Silently ignore negative profile indexes as a way to skip updating devices
                let _ = set_active_profile_index(profile_index, send_sleep_ms, swap_lighting);
            };
        }

        rgb::wooting_rgb_reset_rgb();
    }

    Ok(())
}

/* Helpers */

pub fn select_device_serial(device_serial: &DeviceSerial) -> Result<Device> {
    unsafe {
        rgb::wooting_usb_disconnect(false);
        rgb::wooting_usb_find_keyboard();

        for device_index in 0..rgb::WOOTING_MAX_RGB_DEVICES as u8 {
            if !rgb::wooting_usb_select_device(device_index) {
                continue;
            }

            let device = get_active_device()?;
            if device_serial == &DeviceSerial::from(&device) {
                return Ok(device);
            }
        }
    }

    bail!("Device ({device_serial}) not found")
}
