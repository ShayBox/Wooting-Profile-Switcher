use std::ops::Not;

use egui_extras::{Column, TableBuilder};
use game_scanner::prelude::*;
use image::DynamicImage;
use parking_lot::RwLock;
use tauri::{AppHandle, Manager};
use tauri_egui::{
    eframe::{App, CreationContext, Frame, IconData, NativeOptions},
    egui::{
        self,
        menu::bar as MenuBar,
        Align,
        Button,
        CentralPanel,
        Color32,
        Context,
        Layout,
        ScrollArea,
        SidePanel,
        Slider,
        Stroke,
        TopBottomPanel,
        Vec2,
        Visuals,
        Window,
    },
    EguiPluginHandle,
    Error,
};
use tauri_plugin_autostart::ManagerExt;
use wooting_profile_switcher as wps;

use crate::{
    config::{Config, Rule, Theme},
    Args,
};

const CARGO_PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const CARGO_PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");
const CARGO_PKG_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

struct SelectedRule {
    alias:          String,
    match_app_name: String,
    match_bin_name: String,
    match_bin_path: String,
    match_win_name: String,
    profile_index:  u8,
    rule_index:     usize,
}

impl SelectedRule {
    fn new(rule: Rule, i: usize) -> Self {
        Self {
            alias:          rule.alias,
            match_app_name: rule.match_app_name.unwrap_or_default(),
            match_bin_name: rule.match_bin_name.unwrap_or_default(),
            match_bin_path: rule.match_bin_path.unwrap_or_default(),
            match_win_name: rule.match_win_name.unwrap_or_default(),
            profile_index:  rule.profile_index,
            rule_index:     i,
        }
    }

    fn to_rule(&self) -> Rule {
        Rule {
            alias:          self.alias.clone(),
            match_app_name: self
                .match_app_name
                .is_empty()
                .not()
                .then_some(self.match_app_name.clone()),
            match_bin_name: self
                .match_bin_name
                .is_empty()
                .not()
                .then_some(self.match_bin_name.clone()),
            match_bin_path: self
                .match_bin_path
                .is_empty()
                .not()
                .then_some(self.match_bin_path.clone()),
            match_win_name: self
                .match_win_name
                .is_empty()
                .not()
                .then_some(self.match_win_name.clone()),
            profile_index:  self.profile_index,
        }
    }
}

pub struct MainApp {
    app_handle:          AppHandle,
    open_about:          bool,
    open_auto_launch:    bool,
    open_auto_update:    bool,
    open_new_rule_setup: bool,
    open_confirm_delete: bool,
    selected_rule:       Option<SelectedRule>,
}

impl MainApp {
    pub fn open(app: &AppHandle) -> Result<(), Error> {
        let egui_handle = app.state::<EguiPluginHandle>();

        let native_options = NativeOptions {
            initial_window_size: Some(Vec2::new(720.0, 480.0)),
            icon_data: Self::get_icon_data(),
            ..Default::default()
        };

        let app = app.clone();
        egui_handle.create_window(
            CARGO_PKG_NAME.into(),
            Box::new(|cc| Box::new(Self::new(cc, app))),
            CARGO_PKG_DESCRIPTION.into(),
            native_options,
        )?;

        Ok(())
    }

    fn new(cc: &CreationContext<'_>, app: AppHandle) -> Self {
        let config = app.state::<RwLock<Config>>().read().clone();
        let visuals = match config.ui.theme {
            Theme::Dark => Visuals::dark(),
            Theme::Light => Visuals::light(),
        };

        cc.egui_ctx.set_visuals(visuals);
        cc.egui_ctx.set_pixels_per_point(config.ui.scale);

        Self {
            app_handle:          app,
            open_about:          false,
            open_auto_launch:    config.auto_launch.is_none(),
            open_auto_update:    config.auto_update.is_none(),
            open_new_rule_setup: false,
            open_confirm_delete: false,
            selected_rule:       None,
        }
    }

    /// https://github.com/emilk/egui/discussions/1574
    fn get_icon_data() -> Option<IconData> {
        let buffer = include_bytes!("../icons/icon.png");
        let Ok(image) = image::load_from_memory(buffer).map(DynamicImage::into_rgba8) else {
            return None;
        };

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();

        Some(IconData {
            rgba,
            width,
            height,
        })
    }
}

impl App for MainApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let Self {
            app_handle: app,
            open_about,
            open_auto_launch,
            open_auto_update,
            open_new_rule_setup,
            open_confirm_delete,
            selected_rule,
        } = self;

        let args = app.state::<RwLock<Args>>();
        let config = app.state::<RwLock<Config>>();

        Window::new("About")
            .collapsible(false)
            .open(open_about)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading(CARGO_PKG_DESCRIPTION);
                ui.heading(CARGO_PKG_VERSION);
                ui.label(CARGO_PKG_AUTHORS.split(':').collect::<Vec<_>>().join("\n"));
                ui.hyperlink_to("Source Code Repository", CARGO_PKG_REPOSITORY);
            });

        if *open_auto_launch {
            let auto_launch = app.autolaunch();
            Window::new("Auto Startup")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Would you like to enable automatic startup?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            let _ = auto_launch.enable();
                            let mut config = config.write();
                            config.auto_launch = Some(true);
                            config.save().expect("Failed to save config");
                            *open_auto_launch = false;
                        }
                        if ui.button("No").clicked() {
                            let _ = auto_launch.disable();
                            let mut config = config.write();
                            config.auto_launch = Some(false);
                            config.save().expect("Failed to save config");
                            *open_auto_launch = false;
                        }
                    });
                });
        }

        if *open_auto_update {
            Window::new("Auto Update")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Would you like to enable automatic updates?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            let mut config = config.write();
                            config.auto_update = Some(true);
                            config.save().expect("Failed to save config");
                            *open_auto_update = false;
                        }
                        if ui.button("No").clicked() {
                            let mut config = config.write();
                            config.auto_update = Some(false);
                            config.save().expect("Failed to save config");
                            *open_auto_update = false;
                        }
                    });
                });
        }

        if *open_new_rule_setup {
            Window::new("New Rule Setup")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label("Select a game or blank to create a rule");
                    ui.vertical_centered_justified(|ui| {
                        ScrollArea::vertical().id_source("rules").show(ui, |ui| {
                            let mut games = [
                                game_scanner::amazon::games(),
                                game_scanner::blizzard::games(),
                                game_scanner::epicgames::games(),
                                game_scanner::gog::games(),
                                game_scanner::origin::games(),
                                game_scanner::riotgames::games(),
                                game_scanner::steam::games(),
                                game_scanner::ubisoft::games(),
                            ]
                            .into_iter()
                            .filter_map(Result::ok)
                            .flatten()
                            .collect::<Vec<_>>();

                            games.sort_by(|a, b| String::cmp(&a.name, &b.name));
                            games.insert(
                                0,
                                Game {
                                    name: String::from("Blank"),
                                    ..Default::default()
                                },
                            );

                            for game in games {
                                if ui.button(&game.name).clicked() {
                                    let mut config = config.write();
                                    let rule = Rule {
                                        alias: game.name,
                                        match_bin_path: game
                                            .path
                                            .map(|path| path.display().to_string() + "*"),
                                        ..Default::default()
                                    };
                                    config.rules.push(rule.clone());
                                    config.save().expect("Failed to save config");

                                    let i = config.rules.len() - 1;
                                    *selected_rule = Some(SelectedRule::new(rule, i));
                                    *open_new_rule_setup = false;
                                }
                            }

                            if ui.button("Cancel").clicked() {
                                *open_new_rule_setup = false;
                            }
                        });
                    });
                });
        };

        if *open_confirm_delete {
            Window::new("Confirm Deletion")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Are you sure you want to delete this rule?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            if let Some(rule) = &selected_rule {
                                let mut config = config.write();
                                config.rules.remove(rule.rule_index);
                                config.save().expect("Failed to save config");
                            }
                            *selected_rule = None;
                            *open_confirm_delete = false;
                        }
                        if ui.button("No").clicked() {
                            *open_confirm_delete = false;
                        }
                    });
                });
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            MenuBar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    for (profile_index, title) in config.read().profiles.iter().enumerate() {
                        if ui.button(title).clicked() {
                            ui.close_menu();

                            args.write().profile_index = Some(profile_index as u8);
                            wps::set_active_profile_index(
                                profile_index as u8,
                                config.read().send_sleep_ms,
                                config.read().swap_lighting,
                            );
                        }
                    }

                    ui.separator();

                    let paused = args.read().paused;
                    let text = if paused {
                        "Resume Scanning"
                    } else {
                        "Pause Scanning"
                    };
                    if ui.button(text).clicked() {
                        ui.close_menu();

                        args.write().paused = !paused;
                    }

                    ui.separator();

                    if ui.button("Quit Program").clicked() {
                        ui.close_menu();

                        app.exit(0);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Open Config File").clicked() {
                        ui.close_menu();

                        let config_path = Config::get_path().expect("Failed to get config path");
                        open::that(config_path).expect("Failed to open config file");
                    }
                    if ui.button("Reload Config File").clicked() {
                        ui.close_menu();

                        *config.write() = Config::load().expect("Failed to reload config");
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Swap Theme").clicked() {
                        ui.close_menu();

                        let mut config = config.write();
                        config.ui.theme = match config.ui.theme {
                            Theme::Dark => Theme::Light,
                            Theme::Light => Theme::Dark,
                        };
                        config.save().expect("Failed to save config");
                        ctx.set_visuals(match config.ui.theme {
                            Theme::Dark => Visuals::dark(),
                            Theme::Light => Visuals::light(),
                        });
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        ui.close_menu();

                        *open_about = true;
                    }
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::warn_if_debug_build(ui);
                })
            });
        });

        SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Rules");

                    let add_button = Button::new("+").small();
                    if ui.add(add_button).clicked() {
                        *open_new_rule_setup = true;
                    }

                    let enabled = selected_rule.is_some();
                    let del_button = Button::new("-").small();
                    if ui.add_enabled(enabled, del_button).clicked() {
                        *open_confirm_delete = true;
                    }
                });

                ScrollArea::vertical().id_source("rules").show(ui, |ui| {
                    let rules = config.read().rules.clone();
                    for (i, rule) in rules.into_iter().enumerate() {
                        let mut button = Button::new(&rule.alias).wrap(false);
                        if let Some(rule) = selected_rule {
                            if rule.rule_index == i {
                                let color = ui.visuals().strong_text_color();
                                button = button.stroke(Stroke::new(1.0, color));
                            }
                        }
                        if ui.add(button).clicked() {
                            *selected_rule = Some(SelectedRule::new(rule, i));
                        }
                    }
                });
            });

        CentralPanel::default().show(ctx, |ui| {
            let Some(selected_rule) = selected_rule else {
                ui.heading("No rule selected");
                return;
            };

            ui.colored_label(Color32::KHAKI, "Match variables support Wildcard and Regex");

            let height = 18.0;
            TableBuilder::new(ui)
                .column(Column::exact(100.0))
                .column(Column::remainder())
                .body(|mut body| {
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Rule Alias/Name");
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut selected_rule.alias);
                        });
                    });
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Match App Name");
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut selected_rule.match_app_name);
                        });
                    });
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Match Bin Name");
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut selected_rule.match_bin_name);
                        });
                    });
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Match Bin Path");
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut selected_rule.match_bin_path);
                        });
                    });
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Match Win Name");
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut selected_rule.match_win_name);
                        });
                    });
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Profile Index");
                        });
                        row.col(|ui| {
                            let slider = Slider::new(&mut selected_rule.profile_index, 0..=3)
                                .clamp_to_range(true);
                            ui.add(slider);
                        });
                    });
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            if ui.button("Save").clicked() {
                                let mut config = config.write();
                                config.rules[selected_rule.rule_index] = selected_rule.to_rule();
                                config.save().expect("Failed to save config");
                            }
                        });
                    });
                });
        });
    }
}
