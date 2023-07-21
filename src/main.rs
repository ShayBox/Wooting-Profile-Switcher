// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ffi::OsStr, time::Duration};

use active_win_pos_rs::ActiveWindow;
use anyhow::Result;
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
use wildflower::Pattern;
use wootility::Wootility;
use wooting_profile_switcher as wps;

use crate::config::{Config, Rule};

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
        .system_tray(SystemTray::new())
        .setup(|app| {
            #[cfg(target_os = "macos")] // Hide the macOS dock icon
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
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

            // Load active profile names from Wootility
            if let Ok(wootility) = Wootility::load() {
                config.write().profiles = wootility
                    .profiles
                    .device
                    .into_iter()
                    .map(|device| device.details.name)
                    .collect();
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

            Ok(())
        })
        .on_system_tray_event(move |app, event| {
            let args = app.state::<RwLock<Args>>();
            let config = app.state::<RwLock<Config>>();
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                match id.as_str() {
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
                }
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

        let Some(active_process_name) = active_window
            .process_path
            .file_name()
            .and_then(OsStr::to_str)
        else {
            continue;
        };

        let active_process_state = Rule {
            app_name: Some(active_window.app_name),
            process_name: Some(active_process_name.to_string()),
            process_path: Some(active_window.process_path.display().to_string()),
            profile_index: last_profile_index,
            title: Some(active_window.title),
        };
        println!("Active Process State: {active_process_state:#?}");

        let profile_index = match find_match(active_process_state, &config.read().rules) {
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

        println!("Process Profile Index: {}", profile_index);
        wps::set_active_profile_index(
            profile_index,
            config.read().send_sleep_ms,
            config.read().swap_lighting,
        );
        args.write().profile_index = Some(profile_index);
    }
}

/// Find a matching rule for the active process using Wildcard and Regex
fn find_match(active_process_state: Rule, rules: &[Rule]) -> Option<u8> {
    type RulePropFn = Box<dyn Fn(&Rule) -> Option<&String>>;

    let active_state_props: Vec<(Option<String>, RulePropFn)> = vec![
        (
            active_process_state.app_name,
            Box::new(|rule| rule.app_name.as_ref()),
        ),
        (
            active_process_state.process_name,
            Box::new(|rule| rule.process_name.as_ref()),
        ),
        (
            active_process_state.process_path,
            Box::new(|rule| rule.process_path.as_ref()),
        ),
        (
            active_process_state.title,
            Box::new(|rule| rule.title.as_ref()),
        ),
    ];

    for (active_prop, rule_prop_fn) in active_state_props {
        let Some(active_prop) = active_prop else {
            continue;
        };

        let Some(rule) = rules.iter().find(|rule| {
            if let Some(rule_prop) = rule_prop_fn(rule) {
                if Pattern::new(rule_prop).matches(&active_prop) {
                    true
                } else if let Ok(re) = Regex::new(rule_prop) {
                    re.is_match(&active_prop)
                } else {
                    false
                }
            } else {
                false
            }
        }) else {
            continue;
        };

        return Some(rule.profile_index);
    }

    None
}
