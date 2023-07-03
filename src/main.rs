// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    ffi::OsStr,
    sync::{Arc, RwLock},
    time::Duration,
};

use active_win_pos_rs::{get_active_window, ActiveWindow};
use anyhow::Result;
use clap::Parser;
use regex::Regex;
use tauri::{
    Builder,
    CustomMenuItem,
    SystemTray,
    SystemTrayEvent,
    SystemTrayMenu,
    SystemTrayMenuItem,
};
use wildflower::Pattern;
use wooting_profile_switcher::{get_active_profile_index, set_active_profile_index};
use wooting_rgb_sys::{wooting_rgb_device_info, wooting_rgb_kbd_connected, wooting_rgb_reset};

use crate::config::{Config, Rule};

mod config;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Set the active profile index and exit.
    /// Can be useful for automation scripting.
    #[arg(short, long)]
    profile_index: Option<u8>,

    /// Pause the program by default.
    #[arg(long, default_value_t = false)]
    paused: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Reset the keyboard if the program is killed
    ctrlc::set_handler(move || unsafe {
        wooting_rgb_reset();
        std::process::exit(0);
    })?;

    // Reset the keyboard if the program panics
    std::panic::set_hook(Box::new(|_| unsafe {
        wooting_rgb_reset();
        std::process::exit(0);
    }));

    let args = Arc::new(RwLock::new(Args::parse()));
    let config = Arc::new(Config::load()?);

    // Poll the active window in a background task
    let args_clone = args.clone();
    let config_clone = config.clone();
    tokio::spawn(async move {
        poll_active_window(args_clone, &config_clone).await.unwrap();
    });

    Builder::default()
        .system_tray(
            SystemTray::new().with_menu(
                SystemTrayMenu::new()
                    .add_item(CustomMenuItem::new(String::from("digital"), "Digital"))
                    .add_item(CustomMenuItem::new(String::from("analog_1"), "Analog 1"))
                    .add_item(CustomMenuItem::new(String::from("analog_2"), "Analog 2"))
                    .add_item(CustomMenuItem::new(String::from("analog_3"), "Analog 3"))
                    .add_native_item(SystemTrayMenuItem::Separator)
                    .add_item(CustomMenuItem::new(String::from("pause"), "Pause"))
                    .add_native_item(SystemTrayMenuItem::Separator)
                    .add_item(CustomMenuItem::new(String::from("quit"), "Quit")),
            ),
        )
        .on_system_tray_event(move |app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                let item_handle = app.tray_handle().get_item(&id);
                match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "pause" => {
                        let paused = args.read().unwrap().paused;
                        let title = if paused { "Pause" } else { "Resume" };
                        args.write().unwrap().paused = !paused;
                        item_handle.set_title(title).unwrap();
                    }
                    "digital" => {
                        set_active_profile_index(0, config.send_sleep_ms);
                    }
                    "analog_1" => {
                        set_active_profile_index(1, config.send_sleep_ms);
                    }
                    "analog_2" => {
                        set_active_profile_index(2, config.send_sleep_ms);
                    }
                    "analog_3" => {
                        set_active_profile_index(3, config.send_sleep_ms);
                    }
                    _ => {}
                }
            }
        })
        .run(tauri::generate_context!())?;

    Ok(())
}

/// Poll the active window for matching rules and apply the profile
async fn poll_active_window(args: Arc<RwLock<Args>>, config: &Config) -> Result<()> {
    let device_info = unsafe {
        if !wooting_rgb_kbd_connected() {
            println!("Keyboard not connected.");
            std::process::exit(1)
        }

        wooting_rgb_reset();
        *wooting_rgb_device_info()
    };

    if let Some(profile_index) = args.read().unwrap().profile_index {
        set_active_profile_index(profile_index, config.send_sleep_ms);
        return Ok(());
    }

    let mut last_active_window: ActiveWindow = Default::default();
    let mut last_profile_index = get_active_profile_index(device_info);

    loop {
        std::thread::sleep(Duration::from_millis(config.loop_sleep_ms));
        if args.read().unwrap().paused {
            continue;
        }

        let Ok(active_window) = get_active_window() else {
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

        let profile_index = match find_match(active_process_state, &config.rules) {
            Some(profile_index) => profile_index,
            None => {
                if let Some(fallback_profile_index) = config.fallback_profile_index {
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
        set_active_profile_index(profile_index, config.send_sleep_ms);
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
