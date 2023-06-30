// Disable the Windows Command Prompt for release builds
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{ffi::OsStr, process::exit, time::Duration};

use active_win_pos_rs::{get_active_window, ActiveWindow};
use clap::Parser;
use regex::Regex;
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
}

fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|_| unsafe {
        wooting_rgb_reset();
    }));

    let args = Args::parse();
    let config = Config::load()?;
    let device_info = unsafe {
        if !wooting_rgb_kbd_connected() {
            println!("Keyboard not connected.");
            exit(1)
        }

        *wooting_rgb_device_info()
    };

    if let Some(profile_index) = args.profile_index {
        set_active_profile_index(profile_index, config.send_sleep_ms);
        return Ok(());
    }

    let mut last_active_window: ActiveWindow = Default::default();
    let mut last_profile_index = get_active_profile_index(device_info);

    loop {
        std::thread::sleep(Duration::from_millis(config.loop_sleep_ms));
        let Ok(active_window) = get_active_window() else {
            continue;
        };

        if active_window == last_active_window {
            continue;
        } else {
            last_active_window = active_window.to_owned();
        }

        let Some(active_process_name) = active_window.process_path.file_name().and_then(OsStr::to_str) else {
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

        let Some(profile_index) = find_match(active_process_state, &config.rules) else {
            continue;
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
