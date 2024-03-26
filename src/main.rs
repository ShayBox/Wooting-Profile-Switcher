// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::OsStr, str::FromStr, time::Duration};

use active_win_pos_rs::ActiveWindow;
use anyhow::Result;
use app::MainApp;
use clap::Parser;
use parking_lot::RwLock;
use regex::Regex;
use tauri::{
    AppHandle,
    Builder,
    CustomMenuItem,
    Manager,
    RunEvent,
    SystemTray,
    SystemTrayEvent,
    SystemTrayMenu,
    SystemTrayMenuItem,
};
use tauri_egui::EguiPluginBuilder;
use tauri_plugin_autostart::{MacosLauncher::LaunchAgent, ManagerExt};
use tauri_plugin_updater::UpdaterExt;
use wildflower::Pattern;
#[cfg(target_os = "windows")]
use windows::Win32::System::Console::{AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS};
use wootility::Wootility;
use wooting_profile_switcher as wps;
use wooting_rgb_sys as rgb;
use wps::{DeviceID, DeviceIndices, DeviceSerial, ProfileIndex};

use crate::config::{Config, Rule};

mod app;
mod config;
mod wootility;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// One-shot command line service for automation scripting.
    /// Select an active profile index to apply to a device and exit.
    /// You can specify which device with the serial number argument.
    #[arg(short, long)]
    profile_index: Option<ProfileIndex>,

    /// Select which device to apply the profile.
    /// Defaults to the first found device.
    #[arg(short, long)]
    device_serial: Option<DeviceSerial>,

    /// Pause the active window scanning at startup.
    #[arg(long, default_value_t = false)]
    paused: bool,
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    // Reset the keyboard if the program panics
    std::panic::set_hook(Box::new(|_| unsafe {
        rgb::wooting_rgb_reset();
        std::process::exit(1);
    }));

    // Reset the keyboard if the program is killed/terminated
    ctrlc::set_handler(move || unsafe {
        rgb::wooting_rgb_reset();
        std::process::exit(1);
    })?;

    Builder::default()
        .plugin(tauri_plugin_autostart::init(LaunchAgent, None))
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .system_tray(SystemTray::new())
        .setup(|app| {
            #[cfg(target_os = "macos")] // Hide the macOS dock icon
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            app.wry_plugin(EguiPluginBuilder::new(app.handle()));
            app.manage(RwLock::new(Args::parse()));
            app.manage(RwLock::new(Config::load()?));

            let args = app.state::<RwLock<Args>>();
            let config = app.state::<RwLock<Config>>();
            println!("{:#?}\n{:#?}", args.read(), config.read());

            // One-shot command line argument to set the device and profile index
            if let Some(profile_index) = args.read().profile_index {
                if let Some(device_serial) = args.read().device_serial.clone() {
                    wps::select_device_serial(&device_serial)?;
                }

                let _ = wps::set_active_profile_index(
                    profile_index,
                    config.read().send_sleep_ms,
                    config.read().swap_lighting,
                );

                println!("Profile Index Updated");
                std::process::exit(0);
            }

            println!("Scanning Wootility for devices and profiles to save");
            if let Ok(mut wootility) = Wootility::load() {
                let devices = match wps::get_all_devices() {
                    Ok(devices) => devices,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(1);
                        // TODO: Add a GUI popup
                    }
                };

                println!("Found Devices: {devices:#?}");

                let mut config = config.write();
                config.devices = devices
                    .into_iter()
                    .filter_map(|mut device| {
                        let device_id = DeviceID::from(&device);
                        let device_serial = DeviceSerial::from(&device);
                        println!("Device ID: {device_id}");
                        println!("Device Serial: {device_serial}");
                        println!("Found Profiles: {:#?}", wootility.profiles);

                        device.profiles = wootility
                            .profiles
                            .devices
                            .remove(&device_id)?
                            .into_iter()
                            .map(|profile| profile.details.name)
                            .collect();

                        Some((device_serial, device))
                    })
                    .collect();
                config.save()?;
            } else {
                eprintln!("Failed to access Wootility local storage");
                eprintln!("Please make sure Wootility isn't running");
                // TODO: Add a GUI popup
            };

            // Enable or disable auto-launch on startup
            let auto_launch_manager = app.autolaunch();
            if let Some(auto_launch) = config.read().auto_launch {
                let _ = if auto_launch {
                    println!("Auto Launch Enabled");
                    auto_launch_manager.enable()
                } else {
                    println!("Auto Launch Disabled");
                    auto_launch_manager.disable()
                };
            }

            // Check for and install updates automatically
            // TODO: Add an option to ask prior to installing
            let updater = app.updater();
            if let Some(auto_update) = config.read().auto_update {
                if auto_update {
                    tauri::async_runtime::block_on(async move {
                        match updater.check().await {
                            Ok(update) => {
                                println!("Checking for updates...");
                                if !update.is_update_available() {
                                    println!("No updates found.");
                                    return;
                                }

                                println!("Update found, please wait...");
                                if let Err(error) = update.download_and_install(|_event| {}).await {
                                    eprintln!("{error}");
                                }
                            }
                            Err(error) => {
                                eprintln!("{error}");
                            }
                        }
                    });
                }
            }

            let tray_handle = app.tray_handle();
            let mut system_tray_menu = SystemTrayMenu::new()
                .add_item(CustomMenuItem::new("show", "Show Window"))
                .add_native_item(SystemTrayMenuItem::Separator);

            let devices = config.read().devices.clone();
            for (device_serial, device) in devices {
                let serial_number = device_serial.to_string();
                let title = if config.read().show_serial {
                    &serial_number
                } else {
                    &device.model_name
                };

                let menu_item = CustomMenuItem::new(&serial_number, title).disabled();
                system_tray_menu = system_tray_menu.add_item(menu_item);

                for (i, title) in device.profiles.iter().enumerate() {
                    let id = format!("{device_serial}|{i}");
                    let menu_item = CustomMenuItem::new(id, title).selected();
                    system_tray_menu = system_tray_menu.add_item(menu_item);
                }

                system_tray_menu = system_tray_menu.add_native_item(SystemTrayMenuItem::Separator);
            }

            system_tray_menu = system_tray_menu
                .add_item(CustomMenuItem::new(String::from("pause"), "Pause Scanning"))
                .add_item(CustomMenuItem::new(String::from("reload"), "Reload Config"))
                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(CustomMenuItem::new(String::from("quit"), "Quit Program"));

            tray_handle.set_menu(system_tray_menu)?;

            // Attempt to hide the Windows console
            #[cfg(target_os = "windows")]
            unsafe {
                let _ = FreeConsole();
                let _ = AttachConsole(ATTACH_PARENT_PROCESS);
            }

            Ok(())
        })
        .on_system_tray_event(move |app, event| {
            let args = app.state::<RwLock<Args>>();
            let config = app.state::<RwLock<Config>>();
            match event {
                SystemTrayEvent::LeftClick { .. } => {
                    MainApp::open(app).expect("Failed to open main app");
                }
                SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    "show" => {
                        MainApp::open(app).expect("Failed to open main app");
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    "reload" => {
                        *config.write() = Config::load().expect("Failed to reload config");
                    }
                    "pause" => {
                        let paused = args.read().paused;
                        let title = if paused {
                            "Pause Scanning"
                        } else {
                            "Resume Scanning"
                        };
                        args.write().paused = !paused;

                        let tray_handle = app.tray_handle();
                        let item_handle = tray_handle.get_item(&id);
                        item_handle.set_title(title).unwrap();
                    }
                    _ => {
                        let Some((serial_number, profile_index)) = id.split_once('|') else {
                            return;
                        };

                        let Ok(device_serial) = DeviceSerial::from_str(serial_number) else {
                            return;
                        };

                        let Ok(profile_index) = profile_index.parse::<ProfileIndex>() else {
                            return;
                        };

                        if wps::select_device_serial(&device_serial).is_err() {
                            return;
                        }

                        let _ = wps::set_active_profile_index(
                            profile_index,
                            config.read().send_sleep_ms,
                            config.read().swap_lighting,
                        );

                        let mut args = args.write();
                        args.device_serial = Some(device_serial);
                        args.profile_index = Some(profile_index);
                    }
                },
                _ => {}
            }
        })
        .build(tauri::generate_context!())?
        .run(move |app, event| {
            if matches!(event, RunEvent::Ready) {
                let app = app.clone();
                std::thread::spawn(move || {
                    active_window_polling_task(&app).unwrap();
                });
            }
        });

    Ok(())
}

// Polls the active window to matching rules and applies the keyboard profile
fn active_window_polling_task(app: &AppHandle) -> Result<()> {
    let args = app.state::<RwLock<Args>>();
    let config = app.state::<RwLock<Config>>();

    let mut last_active_window = ActiveWindow::default();
    let mut last_device_indices = wps::get_device_indices()?;

    loop {
        std::thread::sleep(Duration::from_millis(config.read().loop_sleep_ms));

        // Update the selected profile system tray menu item
        if let Some(active_profile_index) = args.read().profile_index {
            if let Some(active_device_serial) = args.read().device_serial.clone() {
                let devices = config.read().devices.clone();
                for (device_serial, device) in devices {
                    if device_serial != active_device_serial {
                        continue;
                    }

                    for i in 0..device.profiles.len() {
                        let id = format!("{device_serial}|{i}");
                        let tray_handle = app.tray_handle();
                        let item_handle = tray_handle.get_item(&id);
                        #[allow(clippy::cast_sign_loss)]
                        let _ = item_handle.set_selected(i == active_profile_index as usize);
                    }
                }
            }
        }

        if args.read().paused {
            continue;
        }

        let Ok(active_window) = active_win_pos_rs::get_active_window() else {
            continue;
        };

        if active_window == last_active_window {
            continue;
        }

        last_active_window = active_window.clone();

        let rules = config.read().rules.clone();
        let Some(device_indices) = find_match(active_window, rules) else {
            continue;
        };

        if device_indices == last_device_indices {
            continue;
        }

        last_device_indices = device_indices.clone();

        println!("Updated Device Indices: {device_indices:#?}");
        wps::set_device_indices(
            device_indices,
            config.read().send_sleep_ms,
            config.read().swap_lighting,
        )?;
    }
}

// Find the first matching device indices for the given active window
#[allow(clippy::uninlined_format_args)]
fn find_match(active_window: ActiveWindow, rules: Vec<Rule>) -> Option<DeviceIndices> {
    type RulePropFn = fn(Rule) -> Option<String>;

    let active_window_bin_path = active_window.process_path.display().to_string();
    let active_window_bin_name = active_window
        .process_path
        .file_name()
        .and_then(OsStr::to_str)
        .map(String::from)
        .unwrap_or_default();

    println!("Updated Active Window:");
    println!("- App Name: {}", active_window.app_name);
    println!("- Bin Name: {}", active_window_bin_name);
    println!("- Bin Path: {}", active_window_bin_path);
    println!("- Win Name: {}", active_window.title);

    let match_active_window: Vec<(RulePropFn, String)> = vec![
        (|rule| rule.match_app_name, active_window.app_name),
        (|rule| rule.match_bin_name, active_window_bin_name),
        (|rule| rule.match_bin_path, active_window_bin_path),
        (|rule| rule.match_win_name, active_window.title),
    ];

    for rule in rules {
        for (rule_prop_fn, active_prop) in &match_active_window {
            if let Some(rule_prop) = rule_prop_fn(rule.clone()) {
                if Pattern::new(&rule_prop.replace('\\', "\\\\")).matches(active_prop) {
                    return Some(rule.device_indices);
                } else if let Ok(re) = Regex::new(&rule_prop) {
                    if re.is_match(active_prop) {
                        return Some(rule.device_indices);
                    }
                }
            }
        }
    }

    None
}
