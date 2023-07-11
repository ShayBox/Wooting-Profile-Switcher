// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    ffi::OsStr,
    sync::{Arc, RwLock},
    time::Duration,
};

use active_win_pos_rs::ActiveWindow;
use anyhow::Result;
use clap::Parser;
use regex::Regex;
use tauri::{
    AppHandle,
    Builder,
    CustomMenuItem,
    SystemTray,
    SystemTrayEvent,
    SystemTrayMenu,
    SystemTrayMenuItem,
};
use wildflower::Pattern;

use crate::config::{Config, Rule};

mod config;

const MENU_ITEMS: [(&str, &str); 4] = [
    ("digital", "Digital"),
    ("analog_1", "Analog 1"),
    ("analog_2", "Analog 2"),
    ("analog_3", "Analog 3"),
];

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

#[tokio::main]
async fn main() -> Result<()> {
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

    let args = Arc::new(RwLock::new(Args::parse()));
    let config = Arc::new(RwLock::new(Config::load()?));

    let mut app_builder = Builder::default();

    {
        let args = args.clone();
        let config = config.clone();

        let mut system_tray_menu = SystemTrayMenu::new();
        for (id, title) in MENU_ITEMS {
            let menu_item = CustomMenuItem::new(String::from(id), title).selected();
            system_tray_menu = system_tray_menu.add_item(menu_item);
        }

        system_tray_menu = system_tray_menu
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(CustomMenuItem::new(String::from("pause"), "Pause Scanning"))
            .add_item(CustomMenuItem::new(String::from("reload"), "Reload Config"))
            .add_native_item(SystemTrayMenuItem::Separator)
            .add_item(CustomMenuItem::new(String::from("quit"), "Quit Program"));

        app_builder = app_builder
            .system_tray(SystemTray::new().with_menu(system_tray_menu))
            .on_system_tray_event(move |app, event| {
                if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                    let item_handle = app.tray_handle().get_item(&id);
                    match id.as_str() {
                        "quit" => {
                            std::process::exit(0);
                        }
                        "reload" => {
                            let mut config_write_lock = config.write().unwrap();
                            *config_write_lock = Config::load().expect("Failed to reload config");
                        }
                        "pause" => {
                            let paused = args.read().unwrap().paused;
                            let title = if paused {
                                "Pause Scanning"
                            } else {
                                "Resume Scanning"
                            };
                            args.write().unwrap().paused = !paused;
                            item_handle.set_title(title).unwrap();
                        }
                        "digital" => {
                            wooting_profile_switcher::set_active_profile_index(
                                0,
                                config.read().unwrap().send_sleep_ms,
                                config.read().unwrap().swap_lighting,
                            );
                            args.write().unwrap().profile_index = Some(0);
                        }
                        "analog_1" => {
                            wooting_profile_switcher::set_active_profile_index(
                                1,
                                config.read().unwrap().send_sleep_ms,
                                config.read().unwrap().swap_lighting,
                            );
                            args.write().unwrap().profile_index = Some(1);
                        }
                        "analog_2" => {
                            wooting_profile_switcher::set_active_profile_index(
                                2,
                                config.read().unwrap().send_sleep_ms,
                                config.read().unwrap().swap_lighting,
                            );
                            args.write().unwrap().profile_index = Some(2);
                        }
                        "analog_3" => {
                            wooting_profile_switcher::set_active_profile_index(
                                3,
                                config.read().unwrap().send_sleep_ms,
                                config.read().unwrap().swap_lighting,
                            );
                            args.write().unwrap().profile_index = Some(3);
                        }
                        _ => {}
                    }
                }
            });
    }

    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    let mut app = app_builder.build(tauri::generate_context!())?;

    #[cfg(target_os = "macos")] // Hide the macOS dock icon
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    {
        // Start polling for a new active window in a background task
        let args = args.clone();
        let config = config.clone();
        tokio::spawn(async move {
            poll_system_active_window_task(args, config).unwrap();
        });
    }

    app.run(move |app, _event| {
        // Start polling for a new active profile in a background task
        let app = app.clone();
        let args = args.clone();
        let config = config.clone();
        tokio::spawn(async move {
            poll_args_active_profile_task(args, config, &app).unwrap();
        });
    });

    Ok(())
}

/// Poll args for a new active profile and update the selected tauri system tray menu item
fn poll_args_active_profile_task(
    args: Arc<RwLock<Args>>,
    config: Arc<RwLock<Config>>,
    app: &AppHandle,
) -> Result<()> {
    loop {
        std::thread::sleep(Duration::from_millis(config.read().unwrap().loop_sleep_ms));

        if let Some(profile_index) = args.read().unwrap().profile_index {
            MENU_ITEMS.iter().enumerate().for_each(|(i, (id, _title))| {
                let item_handle = app.tray_handle().get_item(id);
                item_handle
                    .set_selected(i == profile_index as usize)
                    .unwrap();
            })
        }
    }
}

/// Poll system for a new active window, find a matching rule, and update the keyboard active profile index
fn poll_system_active_window_task(
    args: Arc<RwLock<Args>>,
    config: Arc<RwLock<Config>>,
) -> Result<()> {
    let wooting_usb_meta = wooting_profile_switcher::get_wooting_usb_meta();

    if let Some(profile_index) = args.read().unwrap().profile_index {
        wooting_profile_switcher::set_active_profile_index(
            profile_index,
            config.read().unwrap().send_sleep_ms,
            config.read().unwrap().swap_lighting,
        );
        return Ok(());
    }

    let mut last_active_window: ActiveWindow = Default::default();
    let mut last_profile_index =
        wooting_profile_switcher::get_active_profile_index(wooting_usb_meta);
    args.write().unwrap().profile_index = Some(last_profile_index);

    loop {
        std::thread::sleep(Duration::from_millis(config.read().unwrap().loop_sleep_ms));

        if args.read().unwrap().paused {
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

        let profile_index = match find_match(active_process_state, &config.read().unwrap().rules) {
            Some(profile_index) => profile_index,
            None => {
                if let Some(fallback_profile_index) = config.read().unwrap().fallback_profile_index
                {
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
        wooting_profile_switcher::set_active_profile_index(
            profile_index,
            config.read().unwrap().send_sleep_ms,
            config.read().unwrap().swap_lighting,
        );
        args.write().unwrap().profile_index = Some(profile_index);
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
