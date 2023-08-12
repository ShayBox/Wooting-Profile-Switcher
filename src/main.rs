// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::OsStr, time::Duration};

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

use crate::config::{Config, Rule};

mod app;
mod config;
mod wootility;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Set the active profile index and exit.
    /// Can be useful for automation scripting.
    #[arg(short, long)]
    profile_index: Option<u8>,

    /// Set the program to be paused by default.
    #[arg(long, default_value_t = false)]
    paused: bool,
}

fn main() -> Result<()> {
    // Reset the keyboard if the program panics
    std::panic::set_hook(Box::new(|_| unsafe {
        wooting_rgb_sys::wooting_rgb_reset();
        std::process::exit(0);
    }));

    // Reset the keyboard if the program is killed/terminated
    ctrlc::set_handler(move || unsafe {
        wooting_rgb_sys::wooting_rgb_reset();
        std::process::exit(0);
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

            if let Some(profile_index) = args.read().profile_index {
                wps::set_active_profile_index(
                    profile_index,
                    config.read().send_sleep_ms,
                    config.read().swap_lighting,
                );
                std::process::exit(0);
            }

            let auto_launch_manager = app.autolaunch();
            if let Some(auto_launch) = config.read().auto_launch {
                let _ = if auto_launch {
                    auto_launch_manager.enable()
                } else {
                    auto_launch_manager.disable()
                };
            }

            let updater = app.updater();
            if let Some(auto_update) = config.read().auto_update {
                if auto_update {
                    tauri::async_runtime::block_on(async move {
                        match updater.check().await {
                            Ok(update) => {
                                if !update.is_update_available() {
                                    return;
                                }

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

            // Load active profile names from Wootility
            if let Ok(wootility) = Wootility::load() {
                let mut config = config.write();
                config.profiles = wootility
                    .profiles
                    .device
                    .into_iter()
                    .map(|device| device.details.name)
                    .collect();
                config.save()?;
            }

            let tray_handle = app.tray_handle();
            let mut system_tray_menu = SystemTrayMenu::new();

            for (i, title) in config.read().profiles.iter().enumerate() {
                let id = &i.to_string();
                let menu_item = CustomMenuItem::new(id, title).selected();
                system_tray_menu = system_tray_menu.add_item(menu_item);
            }

            system_tray_menu = system_tray_menu
                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(CustomMenuItem::new(String::from("pause"), "Pause Scanning"))
                .add_item(CustomMenuItem::new(String::from("reload"), "Reload Config"))
                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(CustomMenuItem::new(String::from("quit"), "Quit Program"));

            tray_handle.set_menu(system_tray_menu)?;

            #[cfg(target_os = "windows")]
            unsafe {
                FreeConsole();
                AttachConsole(ATTACH_PARENT_PROCESS);
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
                        if let Ok(profile_index) = id.parse::<u8>() {
                            args.write().profile_index = Some(profile_index);
                            wps::set_active_profile_index(
                                profile_index,
                                config.read().send_sleep_ms,
                                config.read().swap_lighting,
                            );
                        }
                    }
                },
                _ => {}
            }
        })
        .build(tauri::generate_context!())?
        .run(move |app, event| {
            if let RunEvent::Ready = event {
                let app = app.clone();
                std::thread::spawn(move || {
                    active_window_polling_task(app).unwrap();
                });
            }
        });

    Ok(())
}

/// Polls the active window to matching rules and applies the keyboard profile
fn active_window_polling_task(app: AppHandle) -> Result<()> {
    let args = app.state::<RwLock<Args>>();
    let config = app.state::<RwLock<Config>>();
    let wooting_usb_meta = wps::get_wooting_usb_meta();

    let mut last_active_window: ActiveWindow = Default::default();
    let mut last_profile_index = wps::get_active_profile_index(wooting_usb_meta);
    args.write().profile_index = Some(last_profile_index);

    loop {
        std::thread::sleep(Duration::from_millis(config.read().loop_sleep_ms));

        // Update the selected profile system tray menu item
        if let Some(profile_index) = args.read().profile_index {
            for i in 0..config.read().profiles.len() {
                let id = &i.to_string();
                let tray_handle = app.tray_handle();
                let item_handle = tray_handle.get_item(id);
                let _ = item_handle.set_selected(i == profile_index as usize);
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
        } else {
            last_active_window = active_window.to_owned();
        }

        let rules = config.read().rules.clone();
        let profile_index = match find_match(active_window, rules) {
            Some(profile_index) => profile_index,
            None => {
                if let Some(fallback_profile_index) = config.read().fallback_profile_index {
                    fallback_profile_index
                } else {
                    continue;
                }
            }
        };

        if profile_index == last_profile_index {
            continue;
        } else {
            last_profile_index = profile_index;
        }

        println!("Updated Profile Index: {}", profile_index);
        wps::set_active_profile_index(
            profile_index,
            config.read().send_sleep_ms,
            config.read().swap_lighting,
        );
        args.write().profile_index = Some(profile_index);
    }
}

/// Find the first matching profile index for the given active window
fn find_match(active_window: ActiveWindow, rules: Vec<Rule>) -> Option<u8> {
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

    for (rule_prop_fn, active_prop) in match_active_window {
        if let Some(rule) = rules.iter().cloned().find(|rule| {
            if let Some(rule_prop) = rule_prop_fn(rule.clone()) {
                if Pattern::new(&rule_prop.replace('\\', "\\\\")).matches(&active_prop) {
                    true
                } else if let Ok(re) = Regex::new(&rule_prop) {
                    re.is_match(&active_prop)
                } else {
                    false
                }
            } else {
                false
            }
        }) {
            return Some(rule.profile_index);
        }
    }

    None
}
