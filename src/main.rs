// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{backtrace::Backtrace, collections::HashMap, ffi::OsStr, str::FromStr, time::Duration};

use active_win_pos_rs::ActiveWindow;
use anyhow::Result;
use app::MainApp;
use clap::Parser;
use parking_lot::RwLock;
use regex::Regex;
use tauri::{
    image::Image,
    menu::{CheckMenuItem, MenuBuilder, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle,
    Builder,
    Manager,
    RunEvent,
};
use tauri_plugin_autostart::{MacosLauncher::LaunchAgent, ManagerExt};
use tauri_plugin_egui::Builder as EguiPluginBuilder;
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
mod theme;
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

type AppRuntime = tauri::Wry;

struct TrayMenuState {
    show_item:     MenuItem<AppRuntime>,
    pause_item:    MenuItem<AppRuntime>,
    profile_items: HashMap<String, CheckMenuItem<AppRuntime>>,
}

#[derive(Debug, Clone, Default)]
struct ActiveMatchInfo {
    app_name: String,
    bin_name: String,
    bin_path: String,
    win_name: String,
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    #[cfg(target_os = "linux")]
    if std::env::var_os("GDK_BACKEND").is_none() {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    // Reset the keyboard if the program panics
    std::panic::set_hook(Box::new(|info| unsafe {
        eprintln!("Panic: {info}");
        eprintln!("Backtrace:\n{}", Backtrace::force_capture());
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
        .setup(|app| {
            #[cfg(target_os = "macos")] // Hide the macOS dock icon
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            app.wry_plugin(EguiPluginBuilder::new(app.handle().clone()));
            app.manage(RwLock::new(Args::parse()));
            app.manage(RwLock::new(Config::load()?));
            app.manage(RwLock::new(ActiveMatchInfo::default()));
            app.manage(RwLock::new(MainApp::new(app.handle())));
            if let Err(error) = MainApp::init(app.handle()) {
                eprintln!("Failed to initialize main app window: {error}");
            }

            let args = app.state::<RwLock<Args>>();
            let config = app.state::<RwLock<Config>>();
            println!("{:#?}\n{:#?}", args.read(), config.read());

            // One-shot command line argument to set the device and profile index
            let (profile_index, device_serial) = {
                let args = args.read();
                (args.profile_index, args.device_serial.clone())
            };
            if let Some(profile_index) = profile_index {
                if let Some(device_serial) = device_serial {
                    wps::select_device_serial(&device_serial)?;
                }

                let (send_sleep_ms, swap_lighting) = {
                    let config = config.read();
                    (config.send_sleep_ms, config.swap_lighting)
                };
                let _ = wps::set_active_profile_index(profile_index, send_sleep_ms, swap_lighting);

                println!("Profile Index Updated");
                std::process::exit(0);
            }

            println!("Scanning Wootility for devices and profiles to save");
            match Wootility::load() {
                Ok(mut wootility) => {
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
                }
                Err(error) => {
                    eprintln!("Failed to access Wootility local storage");
                    eprintln!("Please make sure Wootility isn't running");
                    eprintln!("{error}"); // TODO: Add a GUI popup
                }
            }

            // Enable or disable auto-launch on startup
            let auto_launch_manager = app.autolaunch();
            let auto_launch = config.read().auto_launch;
            if let Some(auto_launch) = auto_launch {
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
            let auto_update = config.read().auto_update;
            if let Some(auto_update) = auto_update {
                if auto_update {
                    let updater = match app.updater() {
                        Ok(updater) => updater,
                        Err(error) => {
                            eprintln!("{error}");
                            return Ok(());
                        }
                    };

                    tauri::async_runtime::block_on(async move {
                        match updater.check().await {
                            Ok(Some(update)) => {
                                println!("Checking for updates...");
                                println!("Update found, please wait...");
                                if let Err(error) = update
                                    .download_and_install(|_received, _total| {}, || {})
                                    .await
                                {
                                    eprintln!("{error}");
                                }
                            }
                            Ok(None) => {
                                println!("Checking for updates...");
                                println!("No updates found.");
                            }
                            Err(error) => {
                                eprintln!("{error}");
                            }
                        }
                    });
                }
            }

            let app_handle = app.handle().clone();
            let mut profile_items = HashMap::new();
            let show_item =
                MenuItem::with_id(&app_handle, "show", "Show Window", true, None::<&str>)?;
            let mut tray_menu = MenuBuilder::new(&app_handle).item(&show_item).separator();

            let devices = config.read().devices.clone();
            for (device_serial, device) in devices {
                let serial_number = device_serial.to_string();
                let title = if config.read().show_serial {
                    &serial_number
                } else {
                    &device.model_name
                };

                let menu_item =
                    MenuItem::with_id(&app_handle, &serial_number, title, false, None::<&str>)?;
                tray_menu = tray_menu.item(&menu_item);

                for (i, title) in device.profiles.iter().enumerate() {
                    let id = format!("{device_serial}|{i}");
                    let menu_item =
                        CheckMenuItem::with_id(&app_handle, &id, title, true, false, None::<&str>)?;
                    profile_items.insert(id, menu_item.clone());
                    tray_menu = tray_menu.item(&menu_item);
                }

                tray_menu = tray_menu.separator();
            }

            let pause_item =
                MenuItem::with_id(&app_handle, "pause", "Pause Scanning", true, None::<&str>)?;
            let reload_item =
                MenuItem::with_id(&app_handle, "reload", "Reload Config", true, None::<&str>)?;
            let quit_item =
                MenuItem::with_id(&app_handle, "quit", "Quit Program", true, None::<&str>)?;
            tray_menu = tray_menu
                .item(&pause_item)
                .item(&reload_item)
                .separator()
                .item(&quit_item);

            let tray_menu = tray_menu.build()?;
            app.manage(RwLock::new(TrayMenuState {
                show_item: show_item.clone(),
                pause_item: pause_item.clone(),
                profile_items,
            }));

            let tray_app_handle = app.handle().clone();
            let tray_icon = Image::from_bytes(include_bytes!("../icons/icon.png"))?;
            TrayIconBuilder::new()
                .menu(&tray_menu)
                .icon(tray_icon)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    let args = app.state::<RwLock<Args>>();
                    let config = app.state::<RwLock<Config>>();
                    let tray_state = app.state::<RwLock<TrayMenuState>>();
                    let id = &event.id().0;

                    match id.as_str() {
                        "show" => {
                            toggle_main_window(app);
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
                            let _ = tray_state.read().pause_item.set_text(title);
                        }
                        _ => {
                            let Some((serial_number, profile_index)) = id.split_once('|') else {
                                return;
                            };

                            let Ok(device_serial) = DeviceSerial::from_str(serial_number);
                            let Ok(profile_index) = profile_index.parse::<ProfileIndex>() else {
                                return;
                            };
                            let Ok(profile_index_usize) = usize::try_from(profile_index) else {
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

                            let items = tray_state.read();
                            if let Some(active_device_serial) = args.device_serial.clone() {
                                let devices = config.read().devices.clone();
                                for (device_serial, device) in devices {
                                    if device_serial != active_device_serial {
                                        continue;
                                    }

                                    for i in 0..device.profiles.len() {
                                        let item_id = format!("{device_serial}|{i}");
                                        if let Some(item) = items.profile_items.get(&item_id) {
                                            let _ = item.set_checked(i == profile_index_usize);
                                        }
                                    }
                                }
                            }
                        }
                    }
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        toggle_main_window(&tray_app_handle);
                    }
                })
                .build(&app_handle)?;

            // Attempt to hide the Windows console
            #[cfg(target_os = "windows")]
            unsafe {
                let _ = FreeConsole();
                let _ = AttachConsole(ATTACH_PARENT_PROCESS);
            }

            Ok(())
        })
        .build(tauri::generate_context!())?
        .run(move |app, event| match event {
            RunEvent::Ready => {
                let app = app.clone();
                std::thread::spawn(move || {
                    active_window_polling_task(&app).unwrap();
                });
            }
            RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } => {
                if label == MAIN_WINDOW_LABEL {
                    api.prevent_close();
                    if let Some(window) = app.get_window(MAIN_WINDOW_LABEL) {
                        let _ = window.hide();
                    }
                    update_show_menu(app, false);
                }
            }
            _ => {}
        });

    Ok(())
}

const MAIN_WINDOW_LABEL: &str = env!("CARGO_PKG_NAME");

fn update_show_menu(app: &AppHandle, visible: bool) {
    let tray_state = app.state::<RwLock<TrayMenuState>>();
    let title = if visible {
        "Hide Window"
    } else {
        "Show Window"
    };
    let _ = tray_state.read().show_item.set_text(title);
}

fn toggle_main_window(app: &AppHandle) {
    if app.get_window(MAIN_WINDOW_LABEL).is_none() {
        if let Err(error) = MainApp::open(app) {
            eprintln!("Failed to open main app: {error}");
            return;
        }
    }

    if let Some(window) = app.get_window(MAIN_WINDOW_LABEL) {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            let _ = window.hide();
            update_show_menu(app, false);
        } else {
            let _ = window.show();
            let _ = window.set_focus();
            update_show_menu(app, true);
        }
    }
}

// Polls the active window to matching rules and applies the keyboard profile
fn active_window_polling_task(app: &AppHandle) -> Result<()> {
    let args = app.state::<RwLock<Args>>();
    let config = app.state::<RwLock<Config>>();
    let active_info = app.state::<RwLock<ActiveMatchInfo>>();

    let mut last_active_window = ActiveWindow::default();
    let mut last_device_indices = wps::get_device_indices()?;

    loop {
        let loop_sleep_ms = config.read().loop_sleep_ms;
        std::thread::sleep(Duration::from_millis(loop_sleep_ms));

        // Update the selected profile system tray menu item
        let (active_profile_index, active_device_serial, paused) = {
            let args = args.read();
            (args.profile_index, args.device_serial.clone(), args.paused)
        };

        if let (Some(active_profile_index), Some(active_device_serial)) =
            (active_profile_index, active_device_serial)
        {
            if let Ok(active_profile_index_usize) = usize::try_from(active_profile_index) {
                let devices = config.read().devices.clone();
                let tray_state = app.state::<RwLock<TrayMenuState>>();
                let tray_items = tray_state.read();
                for (device_serial, device) in devices {
                    if device_serial != active_device_serial {
                        continue;
                    }

                    for i in 0..device.profiles.len() {
                        let id = format!("{device_serial}|{i}");
                        if let Some(item_handle) = tray_items.profile_items.get(&id) {
                            let _ = item_handle.set_checked(i == active_profile_index_usize);
                        }
                    }
                }
            }
        }

        let Ok(active_window) = active_win_pos_rs::get_active_window() else {
            continue;
        };

        if active_window == last_active_window {
            continue;
        }

        last_active_window = active_window.clone();
        let active_window_bin_path = active_window.process_path.display().to_string();
        let active_window_bin_name = active_window
            .process_path
            .file_name()
            .and_then(OsStr::to_str)
            .map(String::from)
            .unwrap_or_default();

        {
            let mut active_info = active_info.write();
            active_info.app_name.clone_from(&active_window.app_name);
            active_info.bin_name.clone_from(&active_window_bin_name);
            active_info.bin_path.clone_from(&active_window_bin_path);
            active_info.win_name.clone_from(&active_window.title);
        }

        if paused {
            continue;
        }

        let rules = config.read().rules.clone();
        let Some(device_indices) = find_match(active_window, rules) else {
            continue;
        };

        if device_indices == last_device_indices {
            continue;
        }

        last_device_indices.clone_from(&device_indices);

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
