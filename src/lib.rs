use std::time::Duration;

use wooting_rgb_sys::{
    wooting_rgb_device_info,
    wooting_rgb_kbd_connected,
    wooting_rgb_reset,
    wooting_usb_send_feature,
    wooting_usb_send_feature_with_response,
    WOOTING_USB_META,
};

// https://gist.github.com/BigBrainAFK/0ba454a1efb43f7cb6301cda8838f432
const RELOAD_PROFILE: u8 = 7;
const GET_CURRENT_KEYBOARD_PROFILE_INDEX: u8 = 11;
const ACTIVATE_PROFILE: u8 = 23;
const REFRESH_RGB_COLORS: u8 = 29;
const WOOT_DEV_RESET_ALL: u8 = 32;

pub fn get_wooting_usb_meta() -> WOOTING_USB_META {
    unsafe {
        if !wooting_rgb_kbd_connected() {
            println!("Keyboard not connected.");
            std::process::exit(1)
        }

        wooting_rgb_reset();
        *wooting_rgb_device_info()
    }
}

pub fn get_active_profile_index(wooting_usb_meta: WOOTING_USB_META) -> u8 {
    unsafe {
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
            buf[if wooting_usb_meta.v2_interface { 5 } else { 4 }]
        } else {
            u8::MAX
        }
    }
}

pub fn set_active_profile_index(profile_index: u8, send_sleep_ms: u64, swap_lighting: bool) {
    unsafe {
        wooting_usb_send_feature(ACTIVATE_PROFILE, 0, 0, 0, profile_index);
        std::thread::sleep(Duration::from_millis(send_sleep_ms));
        wooting_usb_send_feature(RELOAD_PROFILE, 0, 0, 0, profile_index);

        if swap_lighting {
            std::thread::sleep(Duration::from_millis(send_sleep_ms));
            wooting_usb_send_feature(WOOT_DEV_RESET_ALL, 0, 0, 0, 0);
            std::thread::sleep(Duration::from_millis(send_sleep_ms));
            wooting_usb_send_feature(REFRESH_RGB_COLORS, 0, 0, 0, profile_index);
        }
    }
}
