//! Swappable visual palettes for Shrinkz.

use bevy::prelude::*;

/// Named built-in theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeId {
    Midnight,
    Arcade,
    Neon,
    Paper,
    Forest,
    Sunset,
}

impl ThemeId {
    pub const ALL: [ThemeId; 6] = [
        ThemeId::Midnight,
        ThemeId::Arcade,
        ThemeId::Neon,
        ThemeId::Paper,
        ThemeId::Forest,
        ThemeId::Sunset,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            ThemeId::Midnight => "Midnight",
            ThemeId::Arcade => "Arcade",
            ThemeId::Neon => "Neon",
            ThemeId::Paper => "Paper",
            ThemeId::Forest => "Forest",
            ThemeId::Sunset => "Sunset",
        }
    }

    pub fn next(self) -> Self {
        let all = Self::ALL;
        let i = all.iter().position(|&t| t == self).unwrap_or(0);
        all[(i + 1) % all.len()]
    }

    pub fn palette(self) -> ThemePalette {
        match self {
            ThemeId::Midnight => ThemePalette {
                window_clear: srgb(0.04, 0.045, 0.06),
                hud_text: srgb(0.9, 0.92, 0.95),
                help_text: srgb(0.65, 0.7, 0.75),
                button_text: Color::WHITE,
                axis_horizontal: srgb(0.2, 0.35, 0.55),
                axis_vertical: srgb(0.25, 0.4, 0.35),
                axis_shift: srgb(0.45, 0.32, 0.15),
                playfield_border: srgb(0.55, 0.58, 0.65),
                claimed: srgb(0.32, 0.40, 0.55),
                free: srgb(0.06, 0.07, 0.10),
                wall: srgb(0.90, 0.92, 0.96),
                wall_growing: srgb(0.95, 0.7, 0.25),
                ball: srgb(0.95, 0.35, 0.4),
            },
            ThemeId::Arcade => ThemePalette {
                window_clear: srgb(0.0, 0.0, 0.0),
                hud_text: srgb(1.0, 1.0, 0.2),
                help_text: srgb(0.7, 0.7, 0.5),
                button_text: Color::BLACK,
                axis_horizontal: srgb(0.2, 0.6, 1.0),
                axis_vertical: srgb(0.2, 0.9, 0.4),
                axis_shift: srgb(1.0, 0.85, 0.1),
                playfield_border: srgb(0.85, 0.85, 0.85),
                claimed: srgb(0.12, 0.12, 0.14),
                free: srgb(0.0, 0.0, 0.0),
                wall: srgb(1.0, 1.0, 1.0),
                wall_growing: srgb(1.0, 0.55, 0.0),
                ball: srgb(1.0, 0.15, 0.15),
            },
            ThemeId::Neon => ThemePalette {
                window_clear: srgb(0.02, 0.0, 0.06),
                hud_text: srgb(0.7, 1.0, 0.95),
                help_text: srgb(0.55, 0.7, 0.85),
                button_text: srgb(0.05, 0.05, 0.1),
                axis_horizontal: srgb(0.2, 0.9, 1.0),
                axis_vertical: srgb(0.7, 0.3, 1.0),
                axis_shift: srgb(1.0, 0.2, 0.75),
                playfield_border: srgb(0.4, 0.95, 0.9),
                claimed: srgb(0.15, 0.05, 0.28),
                free: srgb(0.03, 0.01, 0.08),
                wall: srgb(0.3, 1.0, 0.85),
                wall_growing: srgb(1.0, 0.3, 0.9),
                ball: srgb(1.0, 0.95, 0.2),
            },
            ThemeId::Paper => ThemePalette {
                window_clear: srgb(0.88, 0.86, 0.82),
                hud_text: srgb(0.2, 0.18, 0.16),
                help_text: srgb(0.4, 0.38, 0.35),
                button_text: srgb(0.15, 0.12, 0.1),
                axis_horizontal: srgb(0.55, 0.7, 0.85),
                axis_vertical: srgb(0.55, 0.75, 0.6),
                axis_shift: srgb(0.85, 0.65, 0.4),
                playfield_border: srgb(0.35, 0.32, 0.28),
                claimed: srgb(0.72, 0.78, 0.82),
                free: srgb(0.96, 0.94, 0.9),
                wall: srgb(0.25, 0.22, 0.2),
                wall_growing: srgb(0.85, 0.45, 0.2),
                ball: srgb(0.75, 0.2, 0.22),
            },
            ThemeId::Forest => ThemePalette {
                window_clear: srgb(0.04, 0.07, 0.04),
                hud_text: srgb(0.85, 0.92, 0.8),
                help_text: srgb(0.55, 0.65, 0.5),
                button_text: Color::WHITE,
                axis_horizontal: srgb(0.25, 0.45, 0.3),
                axis_vertical: srgb(0.35, 0.5, 0.25),
                axis_shift: srgb(0.55, 0.4, 0.15),
                playfield_border: srgb(0.45, 0.55, 0.4),
                claimed: srgb(0.18, 0.32, 0.2),
                free: srgb(0.05, 0.09, 0.05),
                wall: srgb(0.85, 0.9, 0.75),
                wall_growing: srgb(0.95, 0.75, 0.25),
                ball: srgb(0.9, 0.4, 0.25),
            },
            ThemeId::Sunset => ThemePalette {
                window_clear: srgb(0.08, 0.04, 0.06),
                hud_text: srgb(1.0, 0.9, 0.85),
                help_text: srgb(0.75, 0.55, 0.5),
                button_text: Color::WHITE,
                axis_horizontal: srgb(0.55, 0.3, 0.35),
                axis_vertical: srgb(0.5, 0.25, 0.4),
                axis_shift: srgb(0.9, 0.5, 0.2),
                playfield_border: srgb(0.85, 0.55, 0.45),
                claimed: srgb(0.45, 0.22, 0.28),
                free: srgb(0.1, 0.05, 0.08),
                wall: srgb(1.0, 0.85, 0.7),
                wall_growing: srgb(1.0, 0.65, 0.2),
                ball: srgb(1.0, 0.85, 0.35),
            },
        }
    }
}

/// Full visual palette for one theme.
#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub window_clear: Color,
    pub hud_text: Color,
    pub help_text: Color,
    pub button_text: Color,
    pub axis_horizontal: Color,
    pub axis_vertical: Color,
    pub axis_shift: Color,
    pub playfield_border: Color,
    pub claimed: Color,
    pub free: Color,
    pub wall: Color,
    pub wall_growing: Color,
    pub ball: Color,
}

/// Currently selected theme.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ActiveTheme {
    pub id: ThemeId,
}

impl Default for ActiveTheme {
    fn default() -> Self {
        Self {
            id: ThemeId::Midnight,
        }
    }
}

impl ActiveTheme {
    pub fn palette(self) -> ThemePalette {
        self.id.palette()
    }

    pub fn cycle(&mut self) {
        self.id = self.id.next();
    }
}

fn srgb(r: f32, g: f32, b: f32) -> Color {
    Color::srgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycling_themes_visits_every_built_in_palette() {
        let mut theme = ActiveTheme::default();
        assert_eq!(theme.id, ThemeId::Midnight);
        let mut seen = vec![theme.id];
        for _ in 0..ThemeId::ALL.len() {
            theme.cycle();
            seen.push(theme.id);
        }
        // Full cycle returns to start
        assert_eq!(theme.id, ThemeId::Midnight);
        for id in ThemeId::ALL {
            assert!(seen.contains(&id), "missing theme {id:?}");
            let _ = id.palette();
        }
    }
}
