use std::{collections::HashMap, ffi::CStr, time::Duration};

use anyhow::{bail, Error, Result};
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};
use strum::FromRepr;
use wooting_rgb_sys as rgb;

/* Constants */

// Reverse Engineered from Wootility
// https://gist.github.com/BigBrainAFK/0ba454a1efb43f7cb6301cda8838f432
const MAGIC_WORD_V2: u16 = 0xDAD0;
const MAGIC_WORD_V3: u16 = 0xDAD1;
const GET_SERIAL: u8 = 3;
const RELOAD_PROFILE: u8 = 7;
const GET_CURRENT_KEYBOARD_PROFILE_INDEX: u8 = 11;
const ACTIVATE_PROFILE: u8 = 23;
const REFRESH_RGB_COLORS: u8 = 29;
const WOOT_DEV_RESET_ALL: u8 = 32;

#[allow(clippy::cast_possible_truncation)] // Max is 10
const WOOTING_RGB_MAX_DEVICES: u8 = rgb::WOOTING_MAX_RGB_DEVICES as u8;

/* Typings */

pub type ProfileIndex = i8;
pub type DeviceIndices = HashMap<DeviceSerial, ProfileIndex>;

/* Structures */

#[derive(Clone, Debug, Default, Display, Deserialize, FromRepr, Eq, Hash, PartialEq, Serialize)]
pub enum Stage {
    #[default]
    H = 0, // Mass
    P = 1, // PVT
    T = 2, // DVT
    E = 3, // EVT
    X = 4, // Prototype
}

#[derive(Clone, Debug, Default, Display, Deserialize, Eq, FromStr, Hash, PartialEq, Serialize)]
pub struct U32(u32);

#[derive(Clone, Debug, Default, Display, Deserialize, Eq, FromStr, Hash, PartialEq, Serialize)]
pub struct DeviceID(String);

#[derive(Clone, Debug, Default, Display, Deserialize, Eq, FromStr, Hash, PartialEq, Serialize)]
pub struct DeviceSerial(String);

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Device {
    pub model_name: String,
    pub supplier:   u32,
    pub year:       u32,
    pub week:       u32,
    pub product:    u32,
    pub revision:   u32,
    pub product_id: u32,
    pub stage:      Stage,
    pub variant:    Option<u32>,
    pub pcb_design: Option<u32>,
    pub minor_rev:  Option<u32>,
    pub profiles:   Vec<String>,
}

/* Implementations */

/// Reverse engineered from Wootility
impl TryFrom<Vec<u8>> for U32 {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;

        for byte in &bytes {
            result |= u32::from(byte & 0x7F) << shift;
            shift += 7;

            if byte & 0x80 == 0 {
                return Ok(Self(result));
            }

            if shift > 28 {
                bail!("Integer too large");
            }
        }

        println!("Incomplete integer");
        Ok(Self(result))
    }
}

/// Reverse Engineered from Wootility
impl TryFrom<Vec<u8>> for Device {
    type Error = Error;

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        const OFFSET: usize = 5;
        let length = buffer[4] as usize;
        let mut index = OFFSET;
        let mut device = Self::default();
        while index < length + OFFSET {
            let field = buffer[index];
            index += 1;

            match field >> 3 {
                1 => {
                    let bytes = vec![buffer[index]];
                    device.supplier = U32::try_from(bytes)?.0;
                    index += 1;
                }
                2 => {
                    let bytes = vec![buffer[index]];
                    device.year = U32::try_from(bytes)?.0;
                    index += 1;
                }
                3 => {
                    let bytes = vec![buffer[index]];
                    device.week = U32::try_from(bytes)?.0;
                    index += 1;
                }
                4 => {
                    let bytes = vec![buffer[index]];
                    device.product = U32::try_from(bytes)?.0;
                    index += 1;
                }
                5 => {
                    let bytes = vec![buffer[index]];
                    device.revision = U32::try_from(bytes)?.0;
                    index += 1;
                }
                6 => {
                    let mut bytes = vec![buffer[index]];
                    while buffer[index] >> 3 != 0 {
                        index += 1;
                        bytes.push(buffer[index]);
                    }
                    device.product_id = U32::try_from(bytes)?.0;
                    index += 1;
                }
                7 => {
                    let bytes = vec![buffer[index]];
                    let discriminant = U32::try_from(bytes)?.0 as usize;
                    device.stage = Stage::from_repr(discriminant).unwrap_or_default();
                    index += 1;
                }
                9 => {
                    let bytes = vec![buffer[index]];
                    device.variant = Some(U32::try_from(bytes)?.0);
                    index += 1;
                }
                10 => {
                    let bytes = vec![buffer[index]];
                    device.pcb_design = Some(U32::try_from(bytes)?.0);
                    index += 1;
                }
                11 => {
                    let bytes = vec![buffer[index]];
                    device.minor_rev = Some(U32::try_from(bytes)?.0);
                    index += 1;
                }
                _ => {
                    // Skip unknown field
                    let wire_type = field & 7;
                    match wire_type {
                        0 => index += 1,
                        1 => index += 8,
                        2 => {
                            let length = buffer[index] as usize;
                            index += length + 1;
                        }
                        3 => {
                            // Skip nested fields
                            while buffer[index] & 7 != 4 {
                                index += 1;
                            }
                            index += 1;
                        }
                        5 => index += 4,
                        _ => bail!("Invalid wire type"),
                    }
                }
            }
        }

        Ok(device)
    }
}

impl From<&Device> for DeviceID {
    fn from(device: &Device) -> Self {
        // These must match exactly what the Wooting firmware reports (Wootility var KeyboardType)
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
            "Wooting 60HE+" => 9,
            "Wooting 80HE" => 10,
            &_ => 11,
        };

        let device_id = format!(
            "{}{}{}{}{}{}{}{}{}",
            keyboard_type,
            device.product_id,
            device.product,
            device.revision,
            device.week,
            device.year,
            device.pcb_design.map_or(String::new(), |v| v.to_string()),
            device.minor_rev.map_or(String::new(), |v| v.to_string()),
            device.variant.map_or(String::new(), |v| v.to_string()),
        );

        Self(device_id)
    }
}

impl From<&Device> for DeviceSerial {
    fn from(device: &Device) -> Self {
        Self(format!(
            "A{:02X}B{:02}{:02}W{:02X}{}{}{}{}{}{:05}",
            device.supplier,
            device.year,
            device.week,
            device.product,
            device
                .pcb_design
                .map_or_else(String::new, |pcb_design| format!("T{pcb_design:02}")),
            device.revision,
            device
                .minor_rev
                .map_or_else(String::new, |minor_rev| format!("{minor_rev:02}")),
            device
                .variant
                .map_or_else(String::new, |variant| format!("S{variant:02}")),
            device.stage,
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

            for device_index in 0..WOOTING_RGB_MAX_DEVICES {
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

            for device_index in 0..WOOTING_RGB_MAX_DEVICES {
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

#[allow(clippy::too_many_lines)]
pub fn get_active_device() -> Result<Device> {
    unsafe {
        /* Response Bytes (standard reports)
         * 0-1 Magic Word
         * 2   Command
         * 3   Unknown
         * 4   Length
         * 5-L Buffer
         *
         * Multi-report responses include a report ID at byte 0, shifting the
         * fields above by +1 and using magic word 0xD1DA.
         */
        let response_size = rgb::wooting_usb_get_response_size() as usize;
        let uses_multi_report = rgb::wooting_usb_use_multi_report();
        let report_offset = usize::from(uses_multi_report);
        let extra_data_offset = usize::from(uses_multi_report);
        let expected_magic_word = if uses_multi_report {
            MAGIC_WORD_V3
        } else {
            MAGIC_WORD_V2
        };

        let mut buffer = vec![0u8; response_size];
        let response = rgb::wooting_usb_send_feature_with_response(
            buffer.as_mut_ptr(),
            response_size,
            GET_SERIAL,
            0,
            0,
            0,
            2,
        );

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        if response != response_size as i32 {
            bail!("Invalid response length: got {response}, expected {response_size}");
        }

        let normalized = if report_offset == 0 {
            buffer
        } else {
            buffer[report_offset..].to_vec()
        };

        let magic_word = u16::from_le_bytes([normalized[0], normalized[1]]);
        if magic_word != expected_magic_word {
            bail!("Invalid response type");
        }

        let command = normalized[2];
        if command != GET_SERIAL {
            bail!("Invalid response command");
        }

        let length = normalized[4] as usize;
        let data_start = 5 + extra_data_offset;

        #[cfg(debug_assertions)]
        println!(
            "Serial Buffer: {:?}",
            &normalized[data_start..data_start + length]
        );
        let wooting_usb_meta = *rgb::wooting_usb_get_meta();
        let c_str_model_name = CStr::from_ptr(wooting_usb_meta.model);
        let mut parse_buffer = normalized;
        if uses_multi_report && parse_buffer.len() > data_start {
            parse_buffer.remove(5);
        }
        let mut device = Device::try_from(parse_buffer)?;
        device.model_name = c_str_model_name.to_str()?.replace("Lekker Edition", "LE");

        Ok(device)
    }
}

pub fn get_all_devices() -> Result<Vec<Device>> {
    let mut devices = Vec::new();

    unsafe {
        rgb::wooting_usb_disconnect(false);
        if !rgb::wooting_usb_find_keyboard() {
            bail!("Failed to find keyboard(s)")
        }

        for device_index in 0..WOOTING_RGB_MAX_DEVICES {
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

#[must_use]
pub fn get_active_profile_index() -> ProfileIndex {
    unsafe {
        let response_size = rgb::wooting_usb_get_response_size() as usize;
        let uses_multi_report = rgb::wooting_usb_use_multi_report();
        let report_offset = if uses_multi_report { 1 } else { 0 };
        let extra_data_offset = if uses_multi_report { 1 } else { 0 };

        let mut buff = vec![0u8; response_size];
        let response = rgb::wooting_usb_send_feature_with_response(
            buff.as_mut_ptr(),
            response_size,
            GET_CURRENT_KEYBOARD_PROFILE_INDEX,
            0,
            0,
            0,
            0,
        );

        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        if response == response_size as i32 {
            let is_v2 = rgb::wooting_usb_use_v2_interface();
            let data_offset = report_offset + if is_v2 { 5 } else { 4 } + extra_data_offset;
            buff.get(data_offset)
                .copied()
                .unwrap_or(ProfileIndex::MAX as u8) as ProfileIndex
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

        for device_index in 0..WOOTING_RGB_MAX_DEVICES {
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

        for device_index in 0..WOOTING_RGB_MAX_DEVICES {
            if !rgb::wooting_usb_select_device(device_index) {
                continue;
            }

            let device = get_active_device()?;
            let device_serial = DeviceSerial::from(&device);
            if let Some(profile_index) = device_indices.remove(&device_serial) {
                // Silently ignore negative profile indexes as a way to skip updating devices
                let _ = set_active_profile_index(profile_index, send_sleep_ms, swap_lighting);
            }
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

        for device_index in 0..WOOTING_RGB_MAX_DEVICES {
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
