use cairo::{Context, FontSlant, FontWeight};
use rusty_hex::{Hex, HexColour};

/// The different types of labels that may appear on a tile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Label {
    City(String),
    Y,
    TileName,
    MapLocation(String),
    Revenue(usize),
    PhaseRevenue(Vec<(HexColour, usize, bool)>),
}

impl Label {
    /// Select the font for writing this label.
    pub fn select_font(self: &Self, ctx: &Context, hex: &Hex) {
        // NOTE: scale font size relative to hex diameter.
        let scale = hex.max_d / 125.0;
        match *self {
            Self::City(_) => {
                ctx.select_font_face(
                    "Serif",
                    FontSlant::Normal,
                    FontWeight::Bold,
                );
                ctx.set_font_size(14.0 * scale);
            }
            Self::Y => {
                ctx.select_font_face(
                    "Serif",
                    FontSlant::Normal,
                    FontWeight::Bold,
                );
                ctx.set_font_size(12.0 * scale);
            }
            Self::TileName => {
                ctx.select_font_face(
                    "Sans",
                    FontSlant::Normal,
                    FontWeight::Normal,
                );
                ctx.set_font_size(8.0 * scale);
            }
            Self::MapLocation(_) => {
                ctx.select_font_face(
                    "Serif",
                    FontSlant::Normal,
                    FontWeight::Normal,
                );
                ctx.set_font_size(12.0 * scale);
            }
            Self::Revenue(_) | Self::PhaseRevenue(_) => {
                ctx.select_font_face(
                    "Sans",
                    FontSlant::Normal,
                    FontWeight::Normal,
                );
                ctx.set_font_size(10.0 * scale);
            }
        }
    }
}
