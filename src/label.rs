use cairo::{Context, FontSlant, FontWeight};

/// The different types of labels that may appear on a tile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Label {
    City(String),
    Y,
    TileName,
    Revenue(usize),
}

impl Label {
    /// Select the font for writing this label.
    pub fn select_font(self: &Self, ctx: &Context) {
        match *self {
            Self::City(_) => {
                ctx.select_font_face(
                    "Serif",
                    FontSlant::Normal,
                    FontWeight::Bold,
                );
                ctx.set_font_size(14.0);
            }
            Self::Y => {
                ctx.select_font_face(
                    "Serif",
                    FontSlant::Normal,
                    FontWeight::Bold,
                );
                ctx.set_font_size(12.0);
            }
            Self::TileName => {
                ctx.select_font_face(
                    "Sans",
                    FontSlant::Normal,
                    FontWeight::Normal,
                );
                ctx.set_font_size(8.0);
            }
            Self::Revenue(_) => {
                ctx.select_font_face(
                    "Sans",
                    FontSlant::Normal,
                    FontWeight::Normal,
                );
                ctx.set_font_size(10.0);
            }
        }
    }
}
