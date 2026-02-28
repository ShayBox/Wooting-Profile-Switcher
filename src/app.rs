use std::{
    ops::{Not, Sub},
    sync::LazyLock,
    time::{Duration, Instant},
};

use egui_extras::{Column, TableBody, TableBuilder};
use game_scanner::prelude::*;
use parking_lot::RwLock;
use tauri::{AppHandle, Manager, WindowBuilder};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_egui::{
    egui::{
        self,
        viewport::ViewportCommand,
        Align,
        Button,
        CentralPanel,
        Color32,
        ComboBox,
        Context,
        Layout,
        ScrollArea,
        SidePanel,
        Stroke,
        TopBottomPanel,
        Ui,
        Window,
    },
    AppHandleExt,
};
use wooting_profile_switcher as wps;
use wps::{DeviceIndices, ProfileIndex};

use crate::{
    config::{Config, Rule},
    theme::Theme,
    ActiveMatchInfo,
    Args,
};

const CARGO_PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

static GAMES: LazyLock<Vec<Game>> = LazyLock::new(|| {
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

    games
});

#[derive(Clone, Debug)]
struct SelectedRule {
    alias:          String,
    device_indices: DeviceIndices,
    match_app_name: String,
    match_bin_name: String,
    match_bin_path: String,
    match_win_name: String,
    rule_index:     usize,
}

impl SelectedRule {
    fn new(rule: Rule, i: usize) -> Self {
        Self {
            alias:          rule.alias,
            device_indices: rule.device_indices,
            match_app_name: rule.match_app_name.unwrap_or_default(),
            match_bin_name: rule.match_bin_name.unwrap_or_default(),
            match_bin_path: rule.match_bin_path.unwrap_or_default(),
            match_win_name: rule.match_win_name.unwrap_or_default(),
            rule_index:     i,
        }
    }
}

impl From<SelectedRule> for Rule {
    fn from(rule: SelectedRule) -> Self {
        Self {
            alias:          rule.alias,
            device_indices: rule.device_indices,
            match_app_name: rule
                .match_app_name
                .is_empty()
                .not()
                .then_some(rule.match_app_name),
            match_bin_name: rule
                .match_bin_name
                .is_empty()
                .not()
                .then_some(rule.match_bin_name),
            match_bin_path: rule
                .match_bin_path
                .is_empty()
                .not()
                .then_some(rule.match_bin_path),
            match_win_name: rule
                .match_win_name
                .is_empty()
                .not()
                .then_some(rule.match_win_name),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::struct_excessive_bools)]
pub struct MainApp {
    open_auto_launch:    bool,
    open_auto_update:    bool,
    open_new_rule_setup: bool,
    open_confirm_delete: bool,
    selected_rule:       Option<SelectedRule>,
    base_style:          Option<egui::Style>,
    last_scale:          f32,
    last_theme:          Option<Theme>,
}

impl MainApp {
    pub fn init(app: &AppHandle) -> tauri::Result<()> {
        if app.get_window(CARGO_PKG_NAME).is_some() {
            return Ok(());
        }

        let config = app.state::<RwLock<Config>>().read().clone();
        let window = WindowBuilder::new(app, CARGO_PKG_NAME)
            .title(CARGO_PKG_DESCRIPTION)
            .inner_size(860.0, 560.0)
            .build()?;

        let app_handle = app.clone();
        let mut last = Instant::now();
        app.start_egui_for_window(
            CARGO_PKG_NAME,
            Box::new(move |ctx| {
                let start = Instant::now();
                let delta = start.duration_since(last);
                last = start;

                let main_app = app_handle.state::<RwLock<Self>>();
                main_app.write().update(ctx, &app_handle, delta);

                #[allow(clippy::cast_precision_loss)]
                ctx.request_repaint_after_secs(1.0 / config.ui.frames as f32);
                ctx.send_viewport_cmd(ViewportCommand::IMEAllowed(false));

                let elapsed = start.elapsed();
                let time = Duration::from_nanos(1_000_000_000 / config.ui.frames.max(1));
                if elapsed < time {
                    let remaining = time.sub(elapsed);
                    if remaining > Duration::from_millis(1) {
                        std::thread::sleep(remaining);
                    }

                    while start.elapsed() < time {
                        std::hint::spin_loop();
                    }
                }
            }),
        )?;

        let _ = window.hide();

        Ok(())
    }

    pub fn open(app: &AppHandle) -> tauri::Result<()> {
        if app.get_window(CARGO_PKG_NAME).is_none() {
            Self::init(app)?;
        }

        if let Some(window) = app.get_window(CARGO_PKG_NAME) {
            let _ = window.show();
            let _ = window.set_focus();
        }

        Ok(())
    }

    pub fn new(app: &AppHandle) -> Self {
        let config = app.state::<RwLock<Config>>().read().clone();
        Self {
            open_auto_launch:    config.auto_launch.is_none(),
            open_auto_update:    config.auto_update.is_none(),
            open_new_rule_setup: false,
            open_confirm_delete: false,
            selected_rule:       None,
            base_style:          None,
            last_scale:          1.0,
            last_theme:          None,
        }
    }
}

impl MainApp {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::missing_const_for_fn)]
    fn clamp_i8(value: f32) -> i8 {
        value.round().clamp(f32::from(i8::MIN), f32::from(i8::MAX)) as i8
    }

    fn scale_margin(margin: egui::Margin, scale: f32) -> egui::Margin {
        egui::Margin {
            left:   Self::clamp_i8(margin.leftf() * scale),
            right:  Self::clamp_i8(margin.rightf() * scale),
            top:    Self::clamp_i8(margin.topf() * scale),
            bottom: Self::clamp_i8(margin.bottomf() * scale),
        }
    }

    fn apply_style_scale(style: &mut egui::Style, scale: f32) {
        if (scale - 1.0).abs() < f32::EPSILON {
            return;
        }

        if let Some(font_id) = style.override_font_id.as_mut() {
            font_id.size *= scale;
        }

        for font_id in style.text_styles.values_mut() {
            font_id.size *= scale;
        }

        style.spacing.item_spacing *= scale;
        style.spacing.window_margin = Self::scale_margin(style.spacing.window_margin, scale);
        style.spacing.button_padding *= scale;
        style.spacing.menu_margin = Self::scale_margin(style.spacing.menu_margin, scale);
        style.spacing.indent *= scale;
        style.spacing.interact_size *= scale;
        style.spacing.slider_width *= scale;
        style.spacing.slider_rail_height *= scale;
        style.spacing.combo_width *= scale;
        style.spacing.text_edit_width *= scale;
        style.spacing.icon_width *= scale;
        style.spacing.icon_width_inner *= scale;
        style.spacing.icon_spacing *= scale;
        style.spacing.default_area_size *= scale;
        style.spacing.tooltip_width *= scale;
        style.spacing.menu_width *= scale;
        style.spacing.menu_spacing *= scale;
        style.spacing.combo_height *= scale;

        style.spacing.scroll.bar_width *= scale;
        style.spacing.scroll.handle_min_length *= scale;
        style.spacing.scroll.bar_inner_margin *= scale;
        style.spacing.scroll.bar_outer_margin *= scale;
        style.spacing.scroll.floating_width *= scale;
        style.spacing.scroll.floating_allocated_width *= scale;

        style.interaction.interact_radius *= scale;
        style.interaction.resize_grab_radius_side *= scale;
        style.interaction.resize_grab_radius_corner *= scale;
    }

    fn apply_theme(&mut self, ctx: &Context, config: &RwLock<Config>) {
        let config = config.read();
        let visuals = config.ui.theme.visuals();
        ctx.set_visuals(visuals.clone());

        let theme = config.ui.theme;
        if self.last_theme.as_ref() != Some(&theme) || self.base_style.is_none() {
            let mut fresh_style = egui::Style {
                visuals,
                ..Default::default()
            };
            fresh_style.interaction.selectable_labels = false;
            self.base_style = Some(fresh_style);
            self.last_theme = Some(theme);
            self.last_scale = 0.0;
        }

        if (config.ui.scale - self.last_scale).abs() > f32::EPSILON {
            if let Some(mut style) = self.base_style.clone() {
                Self::apply_style_scale(&mut style, config.ui.scale);
                style.interaction.selectable_labels = false;
                ctx.set_style(style);
                self.last_scale = config.ui.scale;
            }
        }
    }

    fn render_auto_launch_popup(
        &mut self,
        ctx: &Context,
        app: &AppHandle,
        config: &RwLock<Config>,
    ) {
        if !self.open_auto_launch {
            return;
        }

        let auto_launch = app.autolaunch();
        Window::new("Auto Startup")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Would you like to enable automatic startup?");
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        self.open_auto_launch = false;
                        let _ = auto_launch.enable();

                        let mut config = config.write();
                        config.auto_launch = Some(true);
                        config.save().expect("Failed to save config");
                    }
                    if ui.button("No").clicked() {
                        self.open_auto_launch = false;
                        let _ = auto_launch.disable();

                        let mut config = config.write();
                        config.auto_launch = Some(false);
                        config.save().expect("Failed to save config");
                    }
                });
            });
    }

    fn render_auto_update_popup(&mut self, ctx: &Context, config: &RwLock<Config>) {
        if !self.open_auto_update {
            return;
        }

        Window::new("Auto Update")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Would you like to enable automatic updates?");
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        self.open_auto_update = false;

                        let mut config = config.write();
                        config.auto_update = Some(true);
                        config.save().expect("Failed to save config");
                    }
                    if ui.button("No").clicked() {
                        self.open_auto_update = false;

                        let mut config = config.write();
                        config.auto_update = Some(false);
                        config.save().expect("Failed to save config");
                    }
                });
            });
    }

    fn render_new_rule_popup(&mut self, ctx: &Context, config: &RwLock<Config>) {
        if !self.open_new_rule_setup {
            return;
        }

        Window::new("New Rule Setup")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Select a game or blank to create a rule");
                ui.vertical_centered_justified(|ui| {
                    ScrollArea::vertical().id_salt("rules").show(ui, |ui| {
                        for game in GAMES.iter() {
                            let button = Button::new(&game.name);
                            if ui.add_sized([ui.available_width(), 0.0], button).clicked() {
                                let mut config = config.write();
                                let rule = Rule {
                                    alias: game.name.clone(),
                                    match_bin_path: game
                                        .path
                                        .clone()
                                        .map(|path| path.display().to_string() + "*"),
                                    ..Default::default()
                                };
                                self.selected_rule = Some(SelectedRule::new(rule.clone(), 0));
                                self.open_new_rule_setup = false;

                                config.rules.insert(0, rule);
                                config.save().expect("Failed to save config");
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            self.open_new_rule_setup = false;
                        }
                    });
                });
            });
    }

    fn render_confirm_delete_popup(&mut self, ctx: &Context, config: &RwLock<Config>) {
        if !self.open_confirm_delete {
            return;
        }

        Window::new("Confirm Deletion")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Are you sure you want to delete this rule?");
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        if let Some(rule) = &self.selected_rule {
                            let mut config = config.write();
                            config.rules.remove(rule.rule_index);
                            config.save().expect("Failed to save config");
                        }
                        self.selected_rule = None;
                        self.open_confirm_delete = false;
                    }
                    if ui.button("No").clicked() {
                        self.open_confirm_delete = false;
                    }
                });
            });
    }

    fn render_popups(&mut self, ctx: &Context, app: &AppHandle, config: &RwLock<Config>) {
        self.render_auto_launch_popup(ctx, app, config);
        self.render_auto_update_popup(ctx, config);
        self.render_new_rule_popup(ctx, config);
        self.render_confirm_delete_popup(ctx, config);
    }

    fn render_header_controls(ui: &mut Ui, args: &RwLock<Args>, config: &RwLock<Config>) {
        let paused = args.read().paused;
        let status_color = if paused {
            Color32::from_rgb(220, 178, 48)
        } else {
            Color32::from_rgb(100, 210, 140)
        };
        let status_text = if paused { "Paused" } else { "Active" };

        ui.horizontal(|ui| {
            ui.heading("Wooting Profile Switcher");
            ui.separator();
            ui.colored_label(status_color, status_text);
            ui.separator();

            let pause_label = if paused {
                "Resume Scanning"
            } else {
                "Pause Scanning"
            };
            if ui.button(pause_label).clicked() {
                args.write().paused = !paused;
            }

            if ui.button("Open Config").clicked() {
                let config_path = Config::get_path().expect("Failed to get config path");
                open::that(config_path).expect("Failed to open config file");
            }
            if ui.button("Reload Config").clicked() {
                *config.write() = Config::load().expect("Failed to reload config");
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let current_theme = config.read().ui.theme;
                ComboBox::from_id_salt("theme_select")
                    .selected_text(current_theme.display_name())
                    .show_ui(ui, |ui| {
                        for theme in Theme::ALL {
                            let selected = theme == current_theme;
                            if ui
                                .selectable_label(selected, theme.display_name())
                                .clicked()
                            {
                                let mut config = config.write();
                                config.ui.theme = theme;
                                config.save().expect("Failed to save config");
                            }
                        }
                    });
            });
        });
    }

    fn render_keyboard_switcher(ui: &mut Ui, args: &RwLock<Args>, config: &RwLock<Config>) {
        let devices = config.read().devices.clone();
        let show_serial = config.read().show_serial;
        let send_sleep_ms = config.read().send_sleep_ms;
        let swap_lighting = config.read().swap_lighting;

        if devices.is_empty() {
            ui.label("No devices detected.");
            return;
        }

        ui.label("Keyboards");
        ScrollArea::horizontal()
            .id_salt("quick_profiles")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (device_serial, device) in devices {
                        let serial_number = device_serial.to_string();
                        let text = if show_serial {
                            &serial_number
                        } else {
                            &device.model_name
                        };

                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label(text);
                                ui.horizontal_wrapped(|ui| {
                                    for (profile_index, profile_name) in
                                        device.profiles.iter().enumerate()
                                    {
                                        if ui.button(profile_name).clicked() {
                                            if wps::select_device_serial(&device_serial).is_err() {
                                                return;
                                            }

                                            #[allow(clippy::cast_possible_truncation)]
                                            let profile_index = profile_index as ProfileIndex;
                                            let _ = wps::set_active_profile_index(
                                                profile_index,
                                                send_sleep_ms,
                                                swap_lighting,
                                            );

                                            let mut args = args.write();
                                            args.device_serial = Some(device_serial.clone());
                                            args.profile_index = Some(profile_index);
                                        }
                                    }
                                });
                            });
                        });
                    }
                });
            });
    }

    fn render_top_panel(&mut self, ctx: &Context, args: &RwLock<Args>, config: &RwLock<Config>) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(6.0);
            Self::render_header_controls(ui, args, config);

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            Self::render_keyboard_switcher(ui, args, config);

            ui.add_space(6.0);
        });
    }

    fn render_rules_panel(&mut self, ctx: &Context, config: &RwLock<Config>) {
        SidePanel::left("side_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Rules");

                    let add_button = Button::new("+").small();
                    if ui.add(add_button).clicked() {
                        self.open_new_rule_setup = true;
                    }

                    let enabled = self.selected_rule.is_some();
                    let del_button = Button::new("-").small();
                    if ui.add_enabled(enabled, del_button).clicked() {
                        self.open_confirm_delete = true;
                    }
                });

                ScrollArea::vertical().id_salt("rules").show(ui, |ui| {
                    let rules = config.read().rules.clone();
                    for (i, rule) in rules.into_iter().enumerate() {
                        ui.horizontal(|ui| {
                            let row_height = ui.spacing().interact_size.y;
                            if ui
                                .add_sized([row_height, row_height], Button::new("⬆"))
                                .clicked()
                            {
                                let mut config = config.write();
                                let end = config.rules.len() - 1;
                                config.rules.swap(i, if i == 0 { end } else { i - 1 });
                                config.save().expect("Failed to move rule up");
                            }

                            if ui
                                .add_sized([row_height, row_height], Button::new("⬇"))
                                .clicked()
                            {
                                let mut config = config.write();
                                let end = config.rules.len() - 1;
                                config.rules.swap(i, if i == end { 0 } else { i + 1 });
                                config.save().expect("Failed to move rule down");
                            }

                            let mut button = Button::new(&rule.alias);
                            if let Some(rule) = &self.selected_rule {
                                if rule.rule_index == i {
                                    let color = ui.visuals().strong_text_color();
                                    button = button.stroke(Stroke::new(1.0, color));
                                }
                            }
                            let remaining = ui.available_width();
                            if ui.add_sized([remaining, row_height], button).clicked() {
                                self.selected_rule = Some(SelectedRule::new(rule, i));
                            }
                        });
                    }
                });
            });
    }

    fn render_active_window_info(
        ui: &mut Ui,
        active_info: &RwLock<ActiveMatchInfo>,
        selected_rule: &mut SelectedRule,
    ) {
        ui.group(|ui| {
            ui.heading("Current Active Window");
            let active_info = active_info.read();
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Use").clicked() {
                    selected_rule
                        .match_app_name
                        .clone_from(&active_info.app_name);
                }
                ui.label("App Name:");
                ui.monospace(&active_info.app_name);
            });
            ui.horizontal(|ui| {
                if ui.button("Use").clicked() {
                    selected_rule
                        .match_bin_name
                        .clone_from(&active_info.bin_name);
                }
                ui.label("Bin Name:");
                ui.monospace(&active_info.bin_name);
            });
            ui.horizontal(|ui| {
                if ui.button("Use").clicked() {
                    selected_rule
                        .match_bin_path
                        .clone_from(&active_info.bin_path);
                }
                ui.label("Bin Path:");
                ui.monospace(&active_info.bin_path);
            });
            ui.horizontal(|ui| {
                if ui.button("Use").clicked() {
                    selected_rule
                        .match_win_name
                        .clone_from(&active_info.win_name);
                }
                ui.label("Win Name:");
                ui.monospace(&active_info.win_name);
            });
        });
    }

    fn render_rule_match_rows(
        body: &mut TableBody<'_>,
        height: f32,
        selected_rule: &mut SelectedRule,
    ) {
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
    }

    fn render_rule_device_header(body: &mut TableBody<'_>, height: f32, config: &RwLock<Config>) {
        body.row(height, |mut row| {
            row.col(|ui| {
                ui.label(if config.read().show_serial {
                    "Serial Numbers"
                } else {
                    "Model Names"
                });
            });
            row.col(|ui| {
                ui.label("Target Profile");
            });
        });
    }

    fn render_rule_device_rows(
        body: &mut TableBody<'_>,
        height: f32,
        config: &RwLock<Config>,
        selected_rule: &mut SelectedRule,
    ) {
        let devices = config.read().devices.clone();
        for (device_serial, device) in devices {
            let profile_index = selected_rule.device_indices.get_mut(&device_serial);
            if profile_index.is_none() {
                selected_rule
                    .device_indices
                    .insert(device_serial.clone(), 0);
                continue;
            }

            body.row(height, |mut row| {
                row.col(|ui| {
                    let serial_number = device_serial.to_string();
                    ui.label(if config.read().show_serial {
                        &serial_number
                    } else {
                        &device.model_name
                    });
                });
                row.col(|ui| {
                    let profile_index = profile_index.unwrap();
                    let selected_text = if *profile_index == -1 {
                        "Skip".to_string()
                    } else {
                        let index = usize::try_from(*profile_index).ok();
                        index
                            .and_then(|idx| device.profiles.get(idx))
                            .cloned()
                            .unwrap_or_else(|| format!("Index {profile_index}"))
                    };

                    ComboBox::from_id_salt(("profile_select", &device_serial))
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(*profile_index == -1, "Skip").clicked() {
                                *profile_index = -1;
                            }

                            for (idx, name) in device.profiles.iter().enumerate() {
                                let Ok(idx_i8) = i8::try_from(idx) else {
                                    continue;
                                };
                                if ui
                                    .selectable_label(*profile_index == idx_i8, name)
                                    .clicked()
                                {
                                    *profile_index = idx_i8;
                                }
                            }
                        });
                });
            });
        }
    }

    fn render_rule_save_row(
        body: &mut TableBody<'_>,
        height: f32,
        config: &RwLock<Config>,
        selected_rule: &SelectedRule,
    ) {
        body.row(height, |mut row| {
            row.col(|ui| {
                if ui.button("Save").clicked() {
                    let rule = selected_rule.clone().into();
                    let mut config = config.write();
                    config.rules[selected_rule.rule_index] = rule;
                    config.save().expect("Failed to save config");
                }
            });
        });
    }

    fn render_rule_fields_table(
        ui: &mut Ui,
        config: &RwLock<Config>,
        selected_rule: &mut SelectedRule,
    ) {
        let height = 18.0;
        TableBuilder::new(ui)
            .column(Column::exact(140.0))
            .column(Column::remainder())
            .body(|mut body| {
                Self::render_rule_match_rows(&mut body, height, selected_rule);
                Self::render_rule_device_header(&mut body, height, config);
                Self::render_rule_device_rows(&mut body, height, config, selected_rule);
                Self::render_rule_save_row(&mut body, height, config, selected_rule);
            });
    }

    fn render_rule_editor(
        &mut self,
        ctx: &Context,
        config: &RwLock<Config>,
        active_info: &RwLock<ActiveMatchInfo>,
    ) {
        CentralPanel::default().show(ctx, |ui| {
            let Some(selected_rule) = self.selected_rule.as_mut() else {
                ui.heading("No rule selected");
                return;
            };

            Self::render_active_window_info(ui, active_info, selected_rule);

            ui.add_space(6.0);
            ui.colored_label(Color32::KHAKI, "Match variables support Wildcard and Regex");

            Self::render_rule_fields_table(ui, config, selected_rule);
        });
    }

    fn update(&mut self, ctx: &Context, app: &AppHandle, _delta: Duration) {
        let args = app.state::<RwLock<Args>>();
        let config = app.state::<RwLock<Config>>();
        let active_info = app.state::<RwLock<ActiveMatchInfo>>();

        self.apply_theme(ctx, &config);
        self.render_popups(ctx, app, &config);
        self.render_top_panel(ctx, &args, &config);
        self.render_rules_panel(ctx, &config);
        self.render_rule_editor(ctx, &config, &active_info);
    }
}
