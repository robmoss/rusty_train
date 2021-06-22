use crate::Tile;
use cairo::Context;
use n18hex::consts::PI;
use n18hex::{Coord, Hex, HexColour, HexCorner, HexFace, HexPosition};

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
    /// Describe the ideal font for this label.
    pub fn font_description(
        self: &Self,
        hex: &Hex,
    ) -> pango::FontDescription {
        // NOTE: scale font size relative to hex diameter.
        let scale = hex.max_d / 125.0;
        let mut font_descr = pango::FontDescription::new();
        let (family, size, style, weight) = match *self {
            Self::City(_) => {
                ("Serif", 14.0, pango::Style::Normal, pango::Weight::Bold)
            }
            Self::Y => {
                ("Serif", 12.0, pango::Style::Normal, pango::Weight::Bold)
            }
            Self::TileName => {
                ("Sans", 8.0, pango::Style::Normal, pango::Weight::Normal)
            }
            Self::MapLocation(_) => {
                ("Serif", 12.0, pango::Style::Normal, pango::Weight::Bold)
            }
            Self::Revenue(_) | Self::PhaseRevenue(_) => {
                ("Sans", 10.0, pango::Style::Normal, pango::Weight::Normal)
            }
        };
        font_descr.set_family(family);
        // NOTE: font size in *points* is used by set_size(), while
        // *device units* as used by set_absolute_size().
        font_descr.set_absolute_size(size * scale * pango::SCALE as f64);
        font_descr.set_style(style);
        font_descr.set_weight(weight);
        font_descr
    }

    /// Draw this label on a tile, using the current source pattern.
    pub fn draw(
        self: &Self,
        ctx: &Context,
        hex: &Hex,
        pos: &HexPosition,
        tile: &Tile,
    ) {
        let layout = pangocairo::create_layout(ctx)
            .expect("Could not create Pango layout");

        // Select the appropriate font for this label.
        let font_descr = self.font_description(hex);
        layout.set_font_description(Some(&font_descr));

        // Construct the label and determine where to draw it.
        let label_type = self.label_type(hex, tile, layout);
        let origin = get_origin(hex, pos, &label_type.size());
        ctx.new_path();
        label_type.draw(ctx, hex, origin);
        // Prevent the label from being included in a subsequent path.
        ctx.new_path();
    }

    /// Construct a LabelType value for this label.
    fn label_type(
        self: &Self,
        hex: &Hex,
        tile: &Tile,
        layout: pango::Layout,
    ) -> LabelType {
        let scale = hex.max_d / 125.0;
        let black = (0.0, 0.0, 0.0);
        let white = (1.0, 1.0, 1.0);
        match self {
            Label::City(name) => {
                layout.set_text(name);
                layout.set_alignment(pango::Alignment::Center);
                layout.set_wrap(pango::WrapMode::WordChar);
                LabelType::Text {
                    layout,
                    colour: black,
                }
            }
            Label::MapLocation(name) => {
                layout.set_text(name);
                layout.set_alignment(pango::Alignment::Center);
                layout.set_wrap(pango::WrapMode::WordChar);
                layout.set_width((80.0 * scale) as i32 * pango::SCALE);
                let colour = if tile.colour == HexColour::Red
                    || tile.colour == HexColour::Blue
                {
                    white
                } else {
                    black
                };
                LabelType::Text { layout, colour }
            }
            Label::PhaseRevenue(amounts) => {
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
                        layout.set_text(text);
                        let size = layout_size(&layout);
                        (size.width, size.height)
                    })
                    .fold(
                        (0.0, 0.0),
                        |(curr_w, curr_h): (f64, f64), (new_w, new_h)| {
                            (curr_w.max(new_w), curr_h.max(new_h))
                        },
                    );

                // Add a margin around the text.
                let margin_width = 2.0;
                let margin_height = 1.0;
                let box_width = box_width + 2.0 * margin_width;
                let box_height = box_height + 2.0 * margin_height;

                LabelType::HorizBoxes {
                    layout,
                    boxes,
                    active_box,
                    box_width,
                    box_height,
                }
            }
            Label::Revenue(amount_ix) => {
                let amount = tile.get_revenues()[*amount_ix];
                layout.set_text(format!("{}", amount).as_str());
                layout.set_alignment(pango::Alignment::Center);
                layout.set_wrap(pango::WrapMode::WordChar);
                let size = layout_size(&layout);
                let ratio = size.width / size.height;
                // Make the circle/ellipse radius a bit larger than the
                // minimum size required to include the text bounding box.
                let radius = ELLIPSE_RADIUS_SCALE
                    * (0.5 * size.width).max(0.5 * size.height);
                LabelType::CircledText {
                    layout,
                    radius,
                    ratio,
                }
            }
            Label::TileName => {
                layout.set_text(&tile.name);
                layout.set_alignment(pango::Alignment::Center);
                layout.set_wrap(pango::WrapMode::WordChar);
                LabelType::Text {
                    layout,
                    colour: black,
                }
            }
            Label::Y => {
                layout.set_text("Y");
                layout.set_alignment(pango::Alignment::Center);
                layout.set_wrap(pango::WrapMode::WordChar);
                LabelType::Text {
                    layout,
                    colour: black,
                }
            }
        }
    }

    /// Draw a tile name label with custom text, in the default position of
    /// the bottom-right corner.
    pub fn draw_custom_tile_name(ctx: &Context, hex: &Hex, name: &str) {
        let layout = pangocairo::create_layout(ctx)
            .expect("Could not create Pango layout");

        // Select the appropriate font for this label.
        let font_descr = Label::TileName.font_description(hex);
        layout.set_font_description(Some(&font_descr));

        layout.set_text(name);
        layout.set_alignment(pango::Alignment::Center);
        layout.set_wrap(pango::WrapMode::WordChar);
        let label_type = LabelType::Text {
            layout,
            colour: (0.0, 0.0, 0.0),
        };

        let pos = HexPosition::Corner(HexCorner::BottomRight, None);
        let origin = get_origin(hex, &pos, &label_type.size());
        ctx.new_path();
        label_type.draw(ctx, hex, origin);
        ctx.new_path();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    width: f64,
    height: f64,
    dx: f64,
    dy: f64,
}

impl From<pango::Rectangle> for Size {
    fn from(rect: pango::Rectangle) -> Self {
        let width = rect.width as f64;
        let height = rect.height as f64;
        let dx = rect.x as f64;
        let dy = rect.y as f64;
        Self {
            width,
            height,
            dx,
            dy,
        }
    }
}

/// Return the width and height of the space occupied by the layout.
///
/// This currently uses the logical extents rather than the ink extents.
/// For an explanation of how logical and ink extents differ, see:
/// https://mail.gnome.org/archives/gtk-i18n-list/2004-April/msg00007.html
fn layout_size(layout: &pango::Layout) -> Size {
    let use_logical = true;
    let (ink, logical) = layout.get_pixel_extents();
    let rect = if use_logical { logical } else { ink };
    rect.into()
}

/// The supported options for horizontal alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignHoriz {
    Left,
    Centre,
    Right,
}

/// The supported options for vertical alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignVert {
    Top,
    Middle,
    Bottom,
}

/// The alignment for a label, relative to its position coordinates.
///
/// Construct values of this type from [HexPosition] values:
///
/// ```
/// # use n18hex::HexPosition;
/// # use n18tile::label::{Alignment, AlignHoriz, AlignVert};
/// let delta = None;
/// let pos = HexPosition::Centre(delta);
/// let alignment: Alignment = pos.into();
/// assert!(alignment.horiz == AlignHoriz::Centre);
/// assert!(alignment.vert == AlignVert::Middle);
/// ```
///
/// # Horizontal alignment
///
/// - **Centre:** [HexPosition::Centre], and the top and bottom faces.
/// - **Left:** corners and faces on the left half of the hex.
/// - **Right:** corners and faces on the right half of the hex.
///
/// # Vertical alignment
///
/// - **Middle:** [HexPosition::Centre], and the left and right corners.
/// - **Top:** corners and faces in the upper half of the hex.
/// - **Bottom:** corners and faces in the lower half of the hex.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alignment {
    pub horiz: AlignHoriz,
    pub vert: AlignVert,
}

impl Alignment {
    /// Return the coordinates at which to draw a label of the given size, so
    /// that it is aligned correctly with respect to the provided origin.
    pub fn align_origin(&self, size: &Size, mut origin: Coord) -> Coord {
        use AlignHoriz::*;
        use AlignVert::*;
        let shift_h = match self.horiz {
            Left => 0.0,
            Centre => -0.5,
            Right => -1.0,
        };
        let shift_v = match self.vert {
            Top => 0.0,
            Middle => -0.5,
            Bottom => -1.0,
        };
        origin.x += shift_h * size.width;
        origin.y += shift_v * size.height;
        origin
    }
}

/// Define the alignment associated with each [HexPosition].
impl From<HexPosition> for Alignment {
    fn from(pos: HexPosition) -> Self {
        (&pos).into()
    }
}

/// Define the alignment associated with each [HexPosition].
impl From<&HexPosition> for Alignment {
    fn from(pos: &HexPosition) -> Self {
        use HexCorner::*;
        use HexFace::*;
        use HexPosition::*;
        let horiz = match *pos {
            Centre(_) => AlignHoriz::Centre,
            Face(Bottom, _) | Face(Top, _) => AlignHoriz::Centre,
            Face(LowerLeft, _)
            | Face(UpperLeft, _)
            | Corner(BottomLeft, _)
            | Corner(Left, _)
            | Corner(TopLeft, _) => AlignHoriz::Left,
            Face(LowerRight, _)
            | Face(UpperRight, _)
            | Corner(BottomRight, _)
            | Corner(Right, _)
            | Corner(TopRight, _) => AlignHoriz::Right,
        };
        let vert = match *pos {
            Centre(_) => AlignVert::Middle,
            Corner(Left, _) | Corner(Right, _) => AlignVert::Middle,
            Face(UpperLeft, _)
            | Face(Top, _)
            | Face(UpperRight, _)
            | Corner(TopLeft, _)
            | Corner(TopRight, _) => AlignVert::Top,
            Face(LowerLeft, _)
            | Face(Bottom, _)
            | Face(LowerRight, _)
            | Corner(BottomLeft, _)
            | Corner(BottomRight, _) => AlignVert::Bottom,
        };
        Alignment { horiz, vert }
    }
}

/// Position the label with respect to an appropriate location on its bounding
/// box, so that the label remains within the tile.
/// This typically means select the point that is closest to the specified hex
/// face or corner.
pub fn get_origin(hex: &Hex, pos: &HexPosition, size: &Size) -> Coord {
    // Obtain the label position coordinates, before selecting an appropriate
    // anchor.
    // Note that this will include any "nudge" (delta) in pos.
    let mut coord = pos.coord(hex);
    // Negate the dx and dy offsets.
    coord.x -= size.dx;
    coord.y -= size.dy;

    // Adjust the coordinates to align the label with the appropriate anchor.
    let alignment: Alignment = pos.into();
    alignment.align_origin(size, coord)
}

/// The different types of labels that may appear on a tile.
#[derive(Clone, Debug, PartialEq)]
enum LabelType {
    Text {
        layout: pango::Layout,
        colour: (f64, f64, f64),
    },
    CircledText {
        layout: pango::Layout,
        radius: f64,
        ratio: f64,
    },
    HorizBoxes {
        layout: pango::Layout,
        boxes: Vec<(HexColour, String)>,
        active_box: usize,
        box_width: f64,
        box_height: f64,
    },
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
        let matrix = ctx.get_matrix();
        let scale = 1.0 / ratio;
        ctx.scale(1.0, scale);
        ctx.arc(centre.x, centre.y / scale, radius, 0.0, 2.0 * PI);
        ctx.set_matrix(matrix);
    } else {
        ctx.arc(centre.x, centre.y, radius, 0.0, 2.0 * PI);
    };
}

impl LabelType {
    /// Return the width and height of label's contents.
    pub fn size(self: &Self) -> Size {
        match self {
            LabelType::Text { layout, .. } => layout_size(layout),
            LabelType::CircledText { radius, ratio, .. } => {
                let width = 2.0 * radius;
                let height = ellipse_height(*radius, *ratio);
                Size {
                    width,
                    height,
                    dx: 0.0,
                    dy: 0.0,
                }
            }
            LabelType::HorizBoxes {
                boxes,
                box_width,
                box_height,
                ..
            } => {
                let width = box_width * boxes.len() as f64;
                let height = *box_height;
                Size {
                    width,
                    height,
                    dx: 0.0,
                    dy: 0.0,
                }
            }
        }
    }

    /// Draw the label from the specified origin.
    pub fn draw(self: &Self, ctx: &Context, hex: &Hex, origin: Coord) {
        match self {
            LabelType::Text { layout, colour } => {
                ctx.move_to(origin.x, origin.y);
                ctx.set_source_rgb(colour.0, colour.1, colour.2);
                pangocairo::update_layout(ctx, &layout);
                pangocairo::show_layout(ctx, &layout);
            }
            LabelType::CircledText {
                layout,
                radius,
                ratio,
            } => {
                let net_size = self.size();

                // Note: arcs are drawn with respect to the centre coordinate.
                let translate: Coord =
                    (0.5 * net_size.width, 0.5 * net_size.height).into();
                let centre = &origin + &translate;
                define_ellipse(ctx, *radius, *ratio, centre);
                ctx.set_source_rgb(1.0, 1.0, 1.0);
                ctx.fill_preserve();
                ctx.set_line_width(hex.max_d * 0.01);
                ctx.set_source_rgb(0.0, 0.0, 0.0);
                ctx.stroke();

                // Draw the text in the centre of the circle.
                let text_size = layout_size(&layout);
                ctx.move_to(
                    origin.x - text_size.dx
                        + 0.5 * (net_size.width - text_size.width),
                    origin.y - text_size.dy
                        + 0.5 * (net_size.height - text_size.height),
                );
                pangocairo::update_layout(ctx, &layout);
                pangocairo::show_layout(ctx, &layout);
            }
            LabelType::HorizBoxes {
                layout,
                boxes,
                active_box,
                box_width,
                box_height,
            } => {
                // Draw the background and text for each box.
                boxes.iter().enumerate().for_each(
                    |(ix, (bg_colour, text))| {
                        let dx = ix as f64 * box_width;
                        let x0 = origin.x + dx;
                        let y0 = origin.y;
                        // Draw the background.
                        ctx.rectangle(x0, y0, *box_width, *box_height);
                        bg_colour.set_source_rgb(ctx);
                        ctx.fill();
                        // Draw the label text.
                        layout.set_text(text);
                        let size = layout_size(&layout);
                        // Centre the label in the box.
                        let x = x0 - size.dx + 0.5 * (box_width - size.width);
                        let y =
                            y0 - size.dy + 0.5 * (box_height - size.height);
                        ctx.move_to(x, y);
                        ctx.set_source_rgb(0.0, 0.0, 0.0);
                        pangocairo::update_layout(ctx, &layout);
                        pangocairo::show_layout(ctx, &layout);
                        ctx.new_path();
                    },
                );
                // Draw a border around the active box.
                let dx = *active_box as f64 * box_width;
                ctx.rectangle(
                    origin.x + dx,
                    origin.y,
                    *box_width,
                    *box_height,
                );
                ctx.set_source_rgb(0.0, 0.0, 0.0);
                ctx.stroke();
            }
        }
    }
}
