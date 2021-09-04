use crate::Tile;
use cairo::Context;
use n18hex::consts::PI;
use n18hex::{
    Colour, Coord, Hex, HexColour, HexCorner, HexFace, HexPosition,
};

/// The different types of labels that may appear on a tile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Label {
    /// A unique identifier for tiles that can only be placed on a single map
    /// hex, corresponding to a particular city.
    City(String),
    /// An identifier for tiles that can only be placed on a single kind of
    /// city (e.g., "Y", "YY"), which generally applies to multiple map hexes.
    CityKind(String),
    /// The tile name, typically shown in the bottom-right corner of a tile.
    TileName,
    /// A map location (i.e., city or town name), typically used to identify
    /// towns and cities on a map before they are covered with tiles.
    MapLocation(String),
    /// Tile-specific notes, such as construction costs.
    Note(String),
    /// Displays the revenue associated with the tile's nth revenue centre.
    Revenue(usize),
    /// Displays a revenue (`usize`) for each game phase (identified by
    /// [HexColour]) and highlights the active phase (`bool`).
    PhaseRevenue(Vec<(HexColour, usize, bool)>),
}

impl Label {
    /// Returns a new city kind label "Y".
    pub fn y() -> Self {
        Label::CityKind("Y".to_string())
    }

    /// Returns `true` if this label restricts placing and/or upgrading tiles.
    pub fn is_tile_restriction(&self) -> bool {
        match self {
            Self::City(_) => true,
            Self::CityKind(_) => true,
            Self::TileName => false,
            Self::MapLocation(_) => false,
            Self::Revenue(_) => false,
            Self::PhaseRevenue(_) => false,
            Self::Note(_) => false,
        }
    }

    /// Draw this label on a tile.
    pub fn draw(
        &self,
        ctx: &Context,
        hex: &Hex,
        pos: &HexPosition,
        tile: &Tile,
    ) {
        let style = match self {
            Self::City(_) => &hex.theme.city_label,
            Self::CityKind(_) => &hex.theme.city_kind_label,
            Self::TileName => &hex.theme.tile_label,
            Self::MapLocation(_) => &hex.theme.location_label,
            Self::Revenue(_) => &hex.theme.revenue_label,
            Self::PhaseRevenue(_) => &hex.theme.phase_revenue_label,
            Self::Note(_) => &hex.theme.note_label,
        };
        let mut labeller = style.labeller(ctx, hex);
        let coord = pos.coord(hex);

        // Set horizontal/vertical alignment based on the HexPosition anchor.
        let horiz = h_align(pos);
        let vert = v_align(pos);
        labeller.halign(horiz);
        labeller.valign(vert);

        match self {
            Self::City(text) | Self::CityKind(text) | Self::Note(text) => {
                labeller.draw(text, coord);
            }
            Self::TileName => {
                let label_text = &tile.name;
                labeller.draw(label_text, coord);
            }
            Self::MapLocation(name) => {
                let colour = if tile.colour == HexColour::Red
                    || tile.colour == HexColour::Blue
                {
                    Colour::WHITE
                } else {
                    Colour::BLACK
                };
                labeller.colour(colour);
                labeller.draw(name, coord);
            }
            Self::Revenue(amount_ix) => {
                // Determine the text dimensions.
                let amount = tile.revenues()[*amount_ix];
                let label_text = format!("{}", amount);
                let text_size = labeller.size(&label_text);
                // Make the circle/ellipse radius a bit larger than the
                // minimum size required to include the text bounding box.
                let ratio = text_size.width / text_size.height;
                let radius = ELLIPSE_RADIUS_SCALE
                    * (0.5 * text_size.width).max(0.5 * text_size.height);
                let width = 2.0 * radius;
                let height = ellipse_height(radius, ratio);

                let ellipse_size = n18hex::theme::Size {
                    dx: 0.0,
                    dy: 0.0,
                    width,
                    height,
                };
                let origin = ellipse_size.top_left(&coord, horiz, vert);
                let centre = Coord::from((
                    origin.x + 0.5 * width,
                    origin.y + 0.5 * height,
                ));

                // Draw the surrounding circle/ellipse.
                ctx.new_path();
                define_ellipse(ctx, radius, ratio, centre);
                hex.theme.label_circle.apply_fill(ctx);
                ctx.fill_preserve().unwrap();
                hex.theme.label_circle.apply_line_and_stroke(ctx, hex);
                ctx.stroke().unwrap();
                ctx.new_path();

                // NOTE: Draw the text in the centre of the ellipse.
                labeller.halign(n18hex::theme::AlignH::Centre);
                labeller.valign(n18hex::theme::AlignV::Middle);
                labeller.draw(&label_text, centre);
            }
            Self::PhaseRevenue(amounts) => {
                let boxes: Vec<(HexColour, String)> = amounts
                    .iter()
                    .map(|(colour, amount, _active)| {
                        (*colour, format!("{}", amount))
                    })
                    .collect();
                let active_box = amounts
                    .iter()
                    .enumerate()
                    .find_map(
                        |(ix, (_colour, _amount, active))| {
                            if *active {
                                Some(ix)
                            } else {
                                None
                            }
                        },
                    )
                    .expect("No active revenue box");
                let (box_width, box_height) = boxes
                    .iter()
                    .map(|(_colour, text)| {
                        let size = labeller.size(text);
                        (size.width, size.height)
                    })
                    .fold(
                        (0.0, 0.0),
                        |(curr_w, curr_h): (f64, f64), (new_w, new_h)| {
                            (curr_w.max(new_w), curr_h.max(new_h))
                        },
                    );

                // Add a margin around the text.
                let margin_width =
                    hex.theme.phase_revenue_margin_x.absolute(hex);
                let margin_height =
                    hex.theme.phase_revenue_margin_y.absolute(hex);
                let box_width = box_width + 2.0 * margin_width;
                let box_height = box_height + 2.0 * margin_height;
                let net_width = box_width * boxes.len() as f64;

                // Determine the top-left point of these boxes.
                let mut origin = coord;
                origin.x += horiz.hjust() * net_width;
                origin.y += vert.vjust() * box_height;

                // NOTE: override the text alignment, we will manually centre
                // the text in each box.
                labeller.halign(n18hex::theme::AlignH::Left);
                labeller.valign(n18hex::theme::AlignV::Top);

                // Draw the background and text for each box.
                boxes.iter().enumerate().for_each(
                    |(ix, (bg_colour, text))| {
                        let dx = ix as f64 * box_width;
                        let x0 = origin.x + dx;
                        let y0 = origin.y;

                        // Draw the background.
                        ctx.rectangle(x0, y0, box_width, box_height);
                        hex.theme.apply_hex_colour(ctx, *bg_colour);
                        ctx.fill().unwrap();

                        // Centre the label text in the box.
                        let size = labeller.size(text);
                        let x = x0 - size.dx + 0.5 * (box_width - size.width);
                        let y =
                            y0 - size.dy + 0.5 * (box_height - size.height);
                        labeller.draw(text, (x, y).into());
                    },
                );

                // Draw a border around the active box.
                let dx = active_box as f64 * box_width;
                ctx.rectangle(origin.x + dx, origin.y, box_width, box_height);
                Colour::BLACK.apply_colour(ctx);
                ctx.stroke().unwrap();
            }
        };
    }

    /// Draw a tile name label with custom text, in the default position of
    /// the bottom-right corner.
    pub fn draw_custom_tile_name(ctx: &Context, hex: &Hex, name: &str) {
        let mut labeller = hex.theme.tile_label.labeller(ctx, hex);
        let pos = HexPosition::Corner(HexCorner::BottomRight, None);
        let coord = pos.coord(hex);
        let horiz = h_align(&pos);
        let vert = v_align(&pos);
        labeller.halign(horiz);
        labeller.valign(vert);
        labeller.draw(name, coord);
    }
}

/// The horizontal alignment for a label relative to its position coordinates.
///
/// - **Left:** corners and faces on the left half of the hex.
/// - **Centre:** [HexPosition::Centre], and the top and bottom faces.
/// - **Right:** corners and faces on the right half of the hex.
fn h_align(pos: &HexPosition) -> n18hex::theme::AlignH {
    use n18hex::theme::AlignH;
    use HexCorner::*;
    use HexFace::*;
    use HexPosition::*;
    match pos {
        Centre(_) => AlignH::Centre,
        Face(Bottom, _) | Face(Top, _) => AlignH::Centre,
        Face(LowerLeft, _)
        | Face(UpperLeft, _)
        | Corner(BottomLeft, _)
        | Corner(Left, _)
        | Corner(TopLeft, _) => AlignH::Left,
        Face(LowerRight, _)
        | Face(UpperRight, _)
        | Corner(BottomRight, _)
        | Corner(Right, _)
        | Corner(TopRight, _) => AlignH::Right,
    }
}

/// The vertical alignment for a label relative to its position coordinates.
///
/// - **Top:** corners and faces in the upper half of the hex.
/// - **Middle:** [HexPosition::Centre], and the left and right corners.
/// - **Bottom:** corners and faces in the lower half of the hex.
fn v_align(pos: &HexPosition) -> n18hex::theme::AlignV {
    use n18hex::theme::AlignV;
    use HexCorner::*;
    use HexFace::*;
    use HexPosition::*;
    match pos {
        Centre(_) => AlignV::Middle,
        Corner(Left, _) | Corner(Right, _) => AlignV::Middle,
        Face(UpperLeft, _)
        | Face(Top, _)
        | Face(UpperRight, _)
        | Corner(TopLeft, _)
        | Corner(TopRight, _) => AlignV::Top,
        Face(LowerLeft, _)
        | Face(Bottom, _)
        | Face(LowerRight, _)
        | Corner(BottomLeft, _)
        | Corner(BottomRight, _) => AlignV::Bottom,
    }
}
/// The ratio of text width to text height at which we switch from drawing a
/// circle around the text to drawing an ellipse around the text.
/// This improves the appearance of, e.g., revenue labels for $100 and above.
const ELLIPSE_RATIO: f64 = 1.25;

/// The scaling factor for the relationship between the text bounding box
/// dimensions and the surrounding circle/ellipse radius.
/// We want this radius to be larger than the minimum size required to include
/// the text bounding box, so that there is some space between the text and
/// the circle/ellipse border.
const ELLIPSE_RADIUS_SCALE: f64 = 4.0 / 3.0;

fn ellipse_height(radius: f64, ratio: f64) -> f64 {
    let diam = 2.0 * radius;
    if ratio >= ELLIPSE_RATIO {
        diam / ratio
    } else {
        diam
    }
}

fn define_ellipse(ctx: &Context, radius: f64, ratio: f64, centre: Coord) {
    if ratio >= ELLIPSE_RATIO {
        let matrix = ctx.matrix();
        let scale = 1.0 / ratio;
        ctx.scale(1.0, scale);
        ctx.arc(centre.x, centre.y / scale, radius, 0.0, 2.0 * PI);
        ctx.set_matrix(matrix);
    } else {
        ctx.arc(centre.x, centre.y, radius, 0.0, 2.0 * PI);
    };
}
