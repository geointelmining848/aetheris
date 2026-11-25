use iced::highlighter;
use iced::theme::Palette;
use iced::{Color, Theme};

/// Curated built-in themes for Aetheris (both Dark and Light).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AetherisTheme {
    #[default]
    Midnight,
    TokyoNight,
    CatppuccinMocha,
    Nord,
    SolarizedDark,
    PaperLight,
}

/// Visual chrome colors derived from the active theme.
#[derive(Debug, Clone, Copy)]
pub struct ChromeColors {
    pub app_bg: Color,
    pub panel_bg: Color,
    pub header_bg: Color,
    pub tab_bg: Color,
    pub tab_active_bg: Color,
    pub status_bg: Color,
    pub border: Color,
    pub accent: Color,
    pub text: Color,
    pub text_muted: Color,
    pub diff_add_bg: Color,
    pub diff_del_bg: Color,
    pub diff_add_fg: Color,
    pub diff_del_fg: Color,
    pub danger: Color,
    pub success: Color,
}

impl AetherisTheme {
    pub const ALL: &'static [AetherisTheme] = &[
        AetherisTheme::Midnight,
        AetherisTheme::TokyoNight,
        AetherisTheme::CatppuccinMocha,
        AetherisTheme::Nord,
        AetherisTheme::SolarizedDark,
        AetherisTheme::PaperLight,
    ];

    pub fn name(self) -> &'static str {
        match self {
            AetherisTheme::Midnight => "Aetheris Midnight",
            AetherisTheme::TokyoNight => "Tokyo Night",
            AetherisTheme::CatppuccinMocha => "Catppuccin Mocha",
            AetherisTheme::Nord => "Nord",
            AetherisTheme::SolarizedDark => "Solarized Dark",
            AetherisTheme::PaperLight => "Paper Light",
        }
    }

    pub fn from_name(name: &str) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|t| t.name().eq_ignore_ascii_case(name))
            .unwrap_or_default()
    }

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&t| t == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn is_dark(self) -> bool {
        !matches!(self, AetherisTheme::PaperLight)
    }

    pub fn to_iced_theme(self) -> Theme {
        match self {
            AetherisTheme::Midnight => Theme::custom(
                "Aetheris Midnight",
                Palette {
                    background: Color::from_rgb8(15, 17, 26),
                    text: Color::from_rgb8(226, 232, 240),
                    primary: Color::from_rgb8(122, 162, 247),
                    success: Color::from_rgb8(115, 218, 202),
                    warning: Color::from_rgb8(224, 175, 104),
                    danger: Color::from_rgb8(247, 118, 142),
                },
            ),
            AetherisTheme::TokyoNight => Theme::TokyoNight,
            AetherisTheme::CatppuccinMocha => Theme::CatppuccinMocha,
            AetherisTheme::Nord => Theme::Nord,
            AetherisTheme::SolarizedDark => Theme::SolarizedDark,
            AetherisTheme::PaperLight => Theme::custom(
                "Paper Light",
                Palette {
                    background: Color::from_rgb8(250, 249, 246),
                    text: Color::from_rgb8(30, 34, 42),
                    primary: Color::from_rgb8(59, 91, 219),
                    success: Color::from_rgb8(18, 138, 92),
                    warning: Color::from_rgb8(189, 115, 16),
                    danger: Color::from_rgb8(201, 42, 66),
                },
            ),
        }
    }

    pub fn highlighter_theme(self) -> highlighter::Theme {
        match self {
            AetherisTheme::Midnight | AetherisTheme::TokyoNight | AetherisTheme::Nord => {
                highlighter::Theme::Base16Ocean
            }
            AetherisTheme::CatppuccinMocha => highlighter::Theme::Base16Mocha,
            AetherisTheme::SolarizedDark => highlighter::Theme::SolarizedDark,
            AetherisTheme::PaperLight => highlighter::Theme::InspiredGitHub,
        }
    }

    pub fn chrome(self) -> ChromeColors {
        if self.is_dark() {
            match self {
                AetherisTheme::Midnight => ChromeColors {
                    app_bg: Color::from_rgb8(15, 17, 26),
                    panel_bg: Color::from_rgb8(20, 23, 35),
                    header_bg: Color::from_rgb8(17, 20, 31),
                    tab_bg: Color::from_rgb8(22, 26, 39),
                    tab_active_bg: Color::from_rgb8(32, 38, 58),
                    status_bg: Color::from_rgb8(18, 22, 34),
                    border: Color::from_rgb8(42, 49, 72),
                    accent: Color::from_rgb8(122, 162, 247),
                    text: Color::from_rgb8(226, 232, 240),
                    text_muted: Color::from_rgb8(136, 148, 176),
                    diff_add_bg: Color::from_rgba8(34, 92, 66, 0.35),
                    diff_del_bg: Color::from_rgba8(120, 38, 52, 0.38),
                    diff_add_fg: Color::from_rgb8(134, 239, 172),
                    diff_del_fg: Color::from_rgb8(252, 165, 165),
                    danger: Color::from_rgb8(247, 118, 142),
                    success: Color::from_rgb8(115, 218, 202),
                },
                AetherisTheme::SolarizedDark => ChromeColors {
                    app_bg: Color::from_rgb8(0, 43, 54),
                    panel_bg: Color::from_rgb8(7, 54, 66),
                    header_bg: Color::from_rgb8(4, 48, 60),
                    tab_bg: Color::from_rgb8(7, 54, 66),
                    tab_active_bg: Color::from_rgb8(18, 72, 86),
                    status_bg: Color::from_rgb8(5, 49, 61),
                    border: Color::from_rgb8(24, 82, 96),
                    accent: Color::from_rgb8(38, 139, 210),
                    text: Color::from_rgb8(238, 232, 213),
                    text_muted: Color::from_rgb8(131, 148, 150),
                    diff_add_bg: Color::from_rgba8(42, 110, 62, 0.35),
                    diff_del_bg: Color::from_rgba8(150, 42, 42, 0.38),
                    diff_add_fg: Color::from_rgb8(133, 193, 108),
                    diff_del_fg: Color::from_rgb8(240, 128, 128),
                    danger: Color::from_rgb8(220, 50, 47),
                    success: Color::from_rgb8(133, 153, 0),
                },
                _ => {
                    let pal = self.to_iced_theme().palette();
                    ChromeColors {
                        app_bg: pal.background,
                        panel_bg: darken_or_lighten(pal.background, 0.03),
                        header_bg: darken_or_lighten(pal.background, 0.02),
                        tab_bg: darken_or_lighten(pal.background, 0.03),
                        tab_active_bg: darken_or_lighten(pal.background, 0.09),
                        status_bg: darken_or_lighten(pal.background, 0.02),
                        border: darken_or_lighten(pal.background, 0.14),
                        accent: pal.primary,
                        text: pal.text,
                        text_muted: Color {
                            a: 0.65,
                            ..pal.text
                        },
                        diff_add_bg: Color::from_rgba8(34, 92, 66, 0.35),
                        diff_del_bg: Color::from_rgba8(120, 38, 52, 0.38),
                        diff_add_fg: Color::from_rgb8(134, 239, 172),
                        diff_del_fg: Color::from_rgb8(252, 165, 165),
                        danger: pal.danger,
                        success: pal.success,
                    }
                }
            }
        } else {
            ChromeColors {
                app_bg: Color::from_rgb8(250, 249, 246),
                panel_bg: Color::from_rgb8(242, 241, 237),
                header_bg: Color::from_rgb8(238, 237, 232),
                tab_bg: Color::from_rgb8(233, 232, 227),
                tab_active_bg: Color::from_rgb8(255, 255, 255),
                status_bg: Color::from_rgb8(236, 235, 230),
                border: Color::from_rgb8(210, 209, 202),
                accent: Color::from_rgb8(59, 91, 219),
                text: Color::from_rgb8(30, 34, 42),
                text_muted: Color::from_rgb8(102, 108, 122),
                diff_add_bg: Color::from_rgba8(34, 150, 82, 0.18),
                diff_del_bg: Color::from_rgba8(205, 48, 66, 0.16),
                diff_add_fg: Color::from_rgb8(17, 112, 58),
                diff_del_fg: Color::from_rgb8(175, 28, 46),
                danger: Color::from_rgb8(201, 42, 66),
                success: Color::from_rgb8(18, 138, 92),
            }
        }
    }
}

fn darken_or_lighten(c: Color, delta: f32) -> Color {
    Color::from_rgb(
        (c.r + delta).clamp(0.0, 1.0),
        (c.g + delta).clamp(0.0, 1.0),
        (c.b + delta).clamp(0.0, 1.0),
    )
}
