use serde::{Deserialize, Serialize};
use tauri_plugin_egui::egui::{Color32, Stroke, Visuals};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    Dracula,
    Nord,
    GruvboxDark,
    GruvboxLight,
    SolarizedDark,
    SolarizedLight,
    CatppuccinMocha,
    CatppuccinLatte,
    OneDark,
}

impl Theme {
    pub const ALL: [Self; 11] = [
        Self::Dark,
        Self::Light,
        Self::Dracula,
        Self::Nord,
        Self::GruvboxDark,
        Self::GruvboxLight,
        Self::SolarizedDark,
        Self::SolarizedLight,
        Self::CatppuccinMocha,
        Self::CatppuccinLatte,
        Self::OneDark,
    ];

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Dracula => "Dracula",
            Self::Nord => "Nord",
            Self::GruvboxDark => "Gruvbox Dark",
            Self::GruvboxLight => "Gruvbox Light",
            Self::SolarizedDark => "Solarized Dark",
            Self::SolarizedLight => "Solarized Light",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::OneDark => "One Dark",
        }
    }

    pub fn visuals(self) -> Visuals {
        match self {
            Self::Dark => Visuals::dark(),
            Self::Light => Visuals::light(),
            _ => {
                let visuals = if self.is_light_theme() {
                    Visuals::light()
                } else {
                    Visuals::dark()
                };
                apply_palette(visuals, self.palette().expect("Theme palette missing"))
            }
        }
    }

    const fn is_light_theme(self) -> bool {
        matches!(
            self,
            Self::Light | Self::GruvboxLight | Self::SolarizedLight | Self::CatppuccinLatte
        )
    }

    const fn palette(self) -> Option<Palette> {
        Some(match self {
            Self::Dracula => Palette::new(
                rgb(0x28, 0x2a, 0x36),
                rgb(0x44, 0x47, 0x5a),
                rgb(0x62, 0x72, 0xa4),
                rgb(0xf8, 0xf8, 0xf2),
                rgb(0xbd, 0x93, 0xf9),
                rgb(0x50, 0xfa, 0x7b),
            ),
            Self::Nord => Palette::new(
                rgb(0x2e, 0x34, 0x40),
                rgb(0x3b, 0x42, 0x52),
                rgb(0x4c, 0x56, 0x6a),
                rgb(0xec, 0xef, 0xf4),
                rgb(0x88, 0xc0, 0xd0),
                rgb(0xa3, 0xbe, 0x8c),
            ),
            Self::GruvboxDark => Palette::new(
                rgb(0x28, 0x28, 0x28),
                rgb(0x3c, 0x38, 0x36),
                rgb(0x50, 0x49, 0x45),
                rgb(0xeb, 0xdb, 0xb2),
                rgb(0xd7, 0x99, 0x21),
                rgb(0x45, 0x85, 0x88),
            ),
            Self::GruvboxLight => Palette::new(
                rgb(0xfb, 0xf1, 0xc7),
                rgb(0xeb, 0xdb, 0xb2),
                rgb(0xd5, 0xc4, 0xa1),
                rgb(0x3c, 0x38, 0x36),
                rgb(0xd7, 0x99, 0x21),
                rgb(0x45, 0x85, 0x88),
            ),
            Self::SolarizedDark => Palette::new(
                rgb(0x00, 0x2b, 0x36),
                rgb(0x07, 0x36, 0x42),
                rgb(0x58, 0x6e, 0x75),
                rgb(0xee, 0xe8, 0xd5),
                rgb(0x26, 0x8b, 0xd2),
                rgb(0x2a, 0xa1, 0x98),
            ),
            Self::SolarizedLight => Palette::new(
                rgb(0xfd, 0xf6, 0xe3),
                rgb(0xee, 0xe8, 0xd5),
                rgb(0x93, 0xa1, 0xa1),
                rgb(0x07, 0x36, 0x42),
                rgb(0x26, 0x8b, 0xd2),
                rgb(0x2a, 0xa1, 0x98),
            ),
            Self::CatppuccinMocha => Palette::new(
                rgb(0x1e, 0x1e, 0x2e),
                rgb(0x31, 0x32, 0x44),
                rgb(0x45, 0x47, 0x5a),
                rgb(0xcd, 0xd6, 0xf4),
                rgb(0x89, 0xb4, 0xfa),
                rgb(0xa6, 0xe3, 0xa1),
            ),
            Self::CatppuccinLatte => Palette::new(
                rgb(0xef, 0xf1, 0xf5),
                rgb(0xe6, 0xe9, 0xef),
                rgb(0xcc, 0xd0, 0xda),
                rgb(0x4c, 0x4f, 0x69),
                rgb(0x1e, 0x66, 0xf5),
                rgb(0x40, 0xa0, 0x2b),
            ),
            Self::OneDark => Palette::new(
                rgb(0x28, 0x2c, 0x34),
                rgb(0x3b, 0x40, 0x48),
                rgb(0x4b, 0x52, 0x63),
                rgb(0xab, 0xb2, 0xbf),
                rgb(0x61, 0xaf, 0xef),
                rgb(0x98, 0xc3, 0x79),
            ),
            Self::Dark | Self::Light => return None,
        })
    }
}

#[derive(Clone, Copy)]
struct Palette {
    base:        Color32,
    surface:     Color32,
    surface_alt: Color32,
    text:        Color32,
    accent:      Color32,
    accent_alt:  Color32,
}

impl Palette {
    const fn new(
        base: Color32,
        surface: Color32,
        surface_alt: Color32,
        text: Color32,
        accent: Color32,
        accent_alt: Color32,
    ) -> Self {
        Self {
            base,
            surface,
            surface_alt,
            text,
            accent,
            accent_alt,
        }
    }
}

fn apply_palette(mut visuals: Visuals, palette: Palette) -> Visuals {
    visuals.override_text_color = Some(palette.text);
    visuals.hyperlink_color = palette.accent;
    visuals.selection.bg_fill = palette.surface_alt;
    visuals.selection.stroke = Stroke::new(1.0, palette.text);

    visuals.window_fill = palette.base;
    visuals.panel_fill = palette.base;
    visuals.faint_bg_color = palette.surface;
    visuals.extreme_bg_color = palette.surface_alt;

    visuals.widgets.noninteractive.bg_fill = palette.base;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.surface_alt);
    visuals.widgets.inactive.bg_fill = palette.surface;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.surface_alt);
    visuals.widgets.hovered.bg_fill = palette.surface_alt;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, palette.accent);
    visuals.widgets.active.bg_fill = palette.surface_alt;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, palette.accent_alt);
    visuals.widgets.open.bg_fill = palette.surface;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, palette.accent);

    visuals
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}
