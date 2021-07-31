//!
//! Every [Hex] contains a [Theme] that defines the visual appearance of the
//! hexagon, tiles, and tile elements such as track segments and token spaces.
//! Each theme element is one of the following:
//!
//! - [Colour](theme::Colour): defines individual colours;
//! - [Draw](theme::Draw): line and fill properties;
//! - [Length](theme::Length): defines sizes of tile elements; and
//! - [Text](theme::Text): defines text styling for tile labels and
//!   annotations.
//!
//! [Colour](theme::Colour) and [Draw](theme::Draw) provide methods that apply
//! their properties to a `cairo::Context`.
//!
//! [Length::absolute()](theme::Length::absolute) returns lengths (in pixels)
//! for the provided [Hex].
//!
//! [Text::labeller()](theme::Text::labeller) creates
//! [Labeller](theme::Labeller) values that can be used to draw text on a
//! `cairo::Context`.

use std::collections::BTreeMap;

use crate::{Coord, Hex, HexColour};
use cairo::Context;

/// Defines relative and absolute lengths.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    /// Lengths that are relative to the hexagon maximal diameter.
    Relative(f64),
    /// Lengths that are absolute (in pixels).
    Absolute(f64),
}

impl Length {
    /// Returns the absolute length in pixels.
    pub fn absolute(&self, hex: &Hex) -> f64 {
        match self {
            Self::Relative(frac) => frac * hex.max_d,
            Self::Absolute(length) => *length,
        }
    }
}

/// Defines the drawing properties for strokes and fills.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Draw {
    /// The width of this line.
    pub width: Length,
    /// The cap style of this line.
    pub cap: cairo::LineCap,
    /// The join style of this line.
    pub join: cairo::LineJoin,
    /// The colour of this line.
    pub stroke: Colour,
    /// The colour of the filled path interior, for lines that are used to
    /// define and fill closed regions.
    pub fill: Colour,
}

impl Draw {
    /// Applies the line style and stroke colour to the provided context.
    pub fn apply_line_and_stroke(&self, ctx: &Context, hex: &Hex) {
        self.apply_line(ctx, hex);
        self.apply_stroke(ctx);
    }

    /// Applies the line style to the provided context.
    pub fn apply_line(&self, ctx: &Context, hex: &Hex) {
        ctx.set_line_width(self.width.absolute(hex));
        ctx.set_line_cap(self.cap);
        ctx.set_line_join(self.join);
    }

    /// Applies the fill colour to the provided context.
    pub fn apply_fill(&self, ctx: &Context) {
        self.fill.apply_colour(ctx);
    }

    /// Applies the stroke colour to the provided context.
    pub fn apply_stroke(&self, ctx: &Context) {
        self.stroke.apply_colour(ctx);
    }
}

impl Default for Draw {
    fn default() -> Self {
        Draw {
            width: Length::Relative(0.01),
            cap: cairo::LineCap::Butt,
            join: cairo::LineJoin::Round,
            stroke: Colour::BLACK,
            fill: Colour::BLACK,
        }
    }
}

/// The supported horizontal alignment options.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlignH {
    Left,
    Centre,
    Right,
    /// Fractional alignment; `0.0` corresponds to [AlignH::Left], `0.5` to
    /// [AlignH::Centre], and `1.0` to [AlignH::Right].
    /// Values outside of this range are accepted.
    Frac(f64),
}

impl AlignH {
    /// Returns the translation needed for horizontal alignment of an object,
    /// expressed as multiples of that object's width.
    pub fn hjust(&self) -> f64 {
        use AlignH::*;
        match self {
            Left => 0.0,
            Centre => -0.5,
            Right => -1.0,
            Frac(frac) => -frac,
        }
    }
}

/// The supported vertical alignment options.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlignV {
    Top,
    Middle,
    Bottom,
    /// Fractional alignment; `0.0` corresponds to [AlignV::Top], `0.5` to
    /// [AlignV::Middle], and `1.0` to [AlignV::Bottom].
    /// Values outside of this range are accepted.
    Frac(f64),
}

impl AlignV {
    /// Returns the translation needed for vertical alignment of an object,
    /// expressed as multiples of that object's height.
    pub fn vjust(&self) -> f64 {
        use AlignV::*;
        match self {
            Top => 0.0,
            Middle => -0.5,
            Bottom => -1.0,
            Frac(frac) => -frac,
        }
    }
}

/// The supported font families.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontFamily {
    Sans,
    Serif,
    Monospace,
}

impl FontFamily {
    fn set_family(&self, descr: &mut pango::FontDescription) {
        use FontFamily::*;
        match self {
            Sans => descr.set_family("Sans"),
            Serif => descr.set_family("Serif"),
            Monospace => descr.set_family("Monospace"),
        }
    }
}

/// Describes text extents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
    pub dx: f64,
    pub dy: f64,
}

impl Size {
    /// Returns the top-left corner coordinates of this bounding box, for the
    /// provided horizontal and vertical alignment with respect to the given
    /// reference coordinates.
    pub fn top_left(
        &self,
        coord: &Coord,
        horiz: AlignH,
        vert: AlignV,
    ) -> Coord {
        let mut origin = *coord;
        origin.x -= self.dx;
        origin.y -= self.dy;
        // Align content with the appropriate anchor.
        origin.x += horiz.hjust() * self.width;
        origin.y += vert.vjust() * self.height;
        origin
    }
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

/// Draws labels with a specific text style and layout.
///
/// These are created from [Text] styles; see [Text::labeller()] for details.
pub struct Labeller<'a> {
    layout: pango::Layout,
    context: &'a cairo::Context,
    colour: Colour,
    horiz: AlignH,
    vert: AlignV,
}

impl<'a> Labeller<'a> {
    /// Draws the text relative to the provided coordinates, with respect to
    /// the specified horizontal and vertical alignments.
    pub fn draw(&self, text: &str, coord: Coord) {
        let size = self.size(text);
        let coord = size.top_left(&coord, self.horiz, self.vert);
        // Draw the text at the appropriate coordinates.
        self.context.new_path();
        self.context.move_to(coord.x, coord.y);
        self.colour.apply_colour(self.context);
        self.layout.set_text(text);
        pangocairo::update_layout(self.context, &self.layout);
        pangocairo::show_layout(self.context, &self.layout);
        self.context.new_path();
    }

    /// Draws the text relative to the current point, with respect to the
    /// specified horizontal and vertical alignments.
    ///
    /// If there is no current point, the text will be drawn relative to the
    /// origin `(0.0, 0.0)`.
    pub fn draw_at_current_point(&self, text: &str) {
        let size = self.size(text);
        let coord = Coord::from((0.0, 0.0));
        let coord = size.top_left(&coord, self.horiz, self.vert);
        let has_current_point = self
            .context
            .has_current_point()
            .expect("Context::has_current_point() failed");
        if !has_current_point {
            self.context.move_to(0.0, 0.0)
        }
        self.context.rel_move_to(coord.x, coord.y);
        self.colour.apply_colour(self.context);
        self.layout.set_text(text);
        pangocairo::update_layout(self.context, &self.layout);
        pangocairo::show_layout(self.context, &self.layout);
        self.context.new_path();
    }

    /// Returns the width and height of the space occupied by the layout.
    ///
    /// This currently uses the logical extents (which define the bounding box
    /// for layout purposes) rather than the ink extents (which define the
    /// bounding box of the area that will contain ink as a result of drawing
    /// the text).
    /// For example, consider the extents for each letter in the word "Dog".
    pub fn size(&self, text: &str) -> Size {
        let use_logical = true;
        self.layout.set_text(text);
        let (ink, logical) = self.layout.pixel_extents();
        let rect = if use_logical { logical } else { ink };
        rect.into()
    }

    /// Sets the text colour.
    pub fn colour(&mut self, colour: Colour) -> &mut Self {
        self.colour = colour;
        self
    }

    /// Sets the text horizontal alignment.
    pub fn halign(&mut self, align: AlignH) -> &mut Self {
        self.horiz = align;
        self
    }

    /// Sets the text vertical alignment.
    pub fn valign(&mut self, align: AlignV) -> &mut Self {
        self.vert = align;
        self
    }
}

/// Defines the drawing properties for text, such as font family, font size,
/// and alignment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Text {
    family: FontFamily,
    align: pango::Alignment, // Text alignment within the bounding box.
    wrap: pango::WrapMode,
    colour: Colour,
    font_size: f64,
    style: pango::Style,
    weight: pango::Weight,
    variant: pango::Variant,
    horiz: AlignH, // Bounding box alignment.
    vert: AlignV,  // Bounding box alignment.
    max_width: Option<f64>,
}

impl Default for Text {
    fn default() -> Self {
        Text {
            family: FontFamily::Serif,
            align: pango::Alignment::Left,
            wrap: pango::WrapMode::WordChar,
            colour: Colour::BLACK,
            font_size: 12.0,
            style: pango::Style::Normal,
            weight: pango::Weight::Normal,
            variant: pango::Variant::Normal,
            horiz: AlignH::Left,
            vert: AlignV::Top,
            max_width: None,
        }
    }
}

impl Text {
    /// Returns the default text properties.
    ///
    /// These currently include: size 12 serif font, left-aligned, black, and
    /// positioned with respect to the top-left corner.
    pub fn new() -> Self {
        Self::default()
    }

    fn describe(&self, scale: f64) -> pango::FontDescription {
        let font_size = self.font_size * scale * pango::SCALE as f64;
        let mut font_descr = pango::FontDescription::new();
        font_descr.set_style(self.style);
        font_descr.set_weight(self.weight);
        font_descr.set_absolute_size(font_size);
        font_descr.set_variant(self.variant);
        self.family.set_family(&mut font_descr);
        font_descr
    }

    /// Returns a [Labeller] that can be used to draw text.
    pub fn labeller<'a>(&self, ctx: &'a Context, hex: &Hex) -> Labeller<'a> {
        let scale = hex.max_d / 125.0;
        let font_descr = self.describe(scale);
        let layout = pangocairo::create_layout(ctx)
            .expect("Could not create Pango layout");
        layout.set_font_description(Some(&font_descr));
        layout.set_alignment(self.align);
        layout.set_wrap(self.wrap);
        if let Some(width) = self.max_width {
            let abs_width = (width * scale) as i32 * pango::SCALE;
            layout.set_width(abs_width);
        }
        Labeller {
            layout,
            context: ctx,
            colour: self.colour,
            horiz: self.horiz,
            vert: self.vert,
        }
    }

    /// Makes text left-aligned.
    pub fn text_left(&mut self) -> &mut Self {
        self.align = pango::Alignment::Left;
        self
    }

    /// Makes text centred.
    pub fn text_centre(&mut self) -> &mut Self {
        self.align = pango::Alignment::Center;
        self
    }

    /// Makes text right-aligned.
    pub fn text_right(&mut self) -> &mut Self {
        self.align = pango::Alignment::Right;
        self
    }

    /// Wraps lines at character boundaries.
    pub fn wrap_char(&mut self) -> &mut Self {
        self.wrap = pango::WrapMode::Char;
        self
    }

    /// Wraps lines at word boundaries (but falls back to character boundaries
    /// when there is insufficient space).
    pub fn wrap_word(&mut self) -> &mut Self {
        self.wrap = pango::WrapMode::WordChar;
        self
    }

    /// Sets the text colour.
    pub fn colour(&mut self, colour: Colour) -> &mut Self {
        self.colour = colour;
        self
    }

    /// Sets the font size (in points).
    pub fn font_size(&mut self, size: f64) -> &mut Self {
        self.font_size = size;
        self
    }

    /// Sets the bounding box horizontal alignment, relative to the specified
    /// anchor coordinates.
    pub fn halign(&mut self, align: AlignH) -> &mut Self {
        self.horiz = align;
        self
    }

    /// Sets the bounding box vertical alignment, relative to the specified
    /// anchor coordinates.
    pub fn valign(&mut self, align: AlignV) -> &mut Self {
        self.vert = align;
        self
    }

    /// Horizontally aligns bounding boxes with respect to their left edges.
    pub fn halign_left(&mut self) -> &mut Self {
        self.halign(AlignH::Left)
    }

    /// Horizontally aligns bounding boxes with respect to their (horizontal)
    /// centres.
    pub fn halign_centre(&mut self) -> &mut Self {
        self.halign(AlignH::Centre)
    }

    /// Horizontally aligns bounding boxes with respect to their right edges.
    pub fn halign_right(&mut self) -> &mut Self {
        self.halign(AlignH::Right)
    }

    /// Vertically aligns bounding boxes with respect to their top edges.
    pub fn valign_top(&mut self) -> &mut Self {
        self.valign(AlignV::Top)
    }

    /// Vertically aligns bounding boxes with respect to their (vertical)
    /// middle.
    pub fn valign_middle(&mut self) -> &mut Self {
        self.valign(AlignV::Middle)
    }

    /// Vertically aligns bounding boxes with respect to their bottom edges.
    pub fn valign_bottom(&mut self) -> &mut Self {
        self.valign(AlignV::Bottom)
    }

    /// Sets the font family.
    pub fn font(&mut self, family: FontFamily) -> &mut Self {
        self.family = family;
        self
    }

    /// Sets the font family to sans-serif.
    pub fn font_sans(&mut self) -> &mut Self {
        self.font(FontFamily::Sans)
    }

    /// Sets the font family to serif.
    pub fn font_serif(&mut self) -> &mut Self {
        self.font(FontFamily::Serif)
    }

    /// Sets the font family to monospace.
    pub fn font_monospace(&mut self) -> &mut Self {
        self.font(FontFamily::Monospace)
    }

    /// Sets the font style to roman (upright).
    pub fn roman(&mut self) -> &mut Self {
        self.style = pango::Style::Normal;
        self
    }

    /// Sets the font style to italic.
    pub fn italic(&mut self) -> &mut Self {
        self.style = pango::Style::Italic;
        self
    }

    /// Sets the font weight to normal.
    pub fn normal(&mut self) -> &mut Self {
        self.weight = pango::Weight::Normal;
        self
    }

    /// Sets the font weight to bold.
    pub fn bold(&mut self) -> &mut Self {
        self.weight = pango::Weight::Bold;
        self
    }

    /// Uses lowercase characters that resemble shrunken capital characters.
    pub fn small_caps(&mut self) -> &mut Self {
        self.variant = pango::Variant::SmallCaps;
        self
    }

    /// Uses regular lowercase characters.
    pub fn lowercase(&mut self) -> &mut Self {
        self.variant = pango::Variant::Normal;
        self
    }

    /// Sets the text maximum width, after which the text will be wrapped to
    /// span multiple lines.
    pub fn max_width(&mut self, width: f64) -> &mut Self {
        self.max_width = Some(width);
        self
    }

    /// Places no limit on the text width, ensuring that the text will be
    /// displayed in a single line.
    pub fn no_max_width(&mut self) -> &mut Self {
        self.max_width = None;
        self
    }
}

/// Defines all of the drawing properties used to draw hexagons, tiles, and
/// tile elements.
///
/// Note that many quantities with dimension distance, such as lengths,
/// widths, and font sizes, can be defined relative to the hexagon size.
///
/// # Panics
///
/// It is possible to cause panics by changing the theme settings, since they
/// affect how tiles are drawn, which in turn can affect track connectivity.
///
/// For example, a sufficiently large token-space radius will cause token
/// spaces to reach the tile edge and be connected to both ends of a track
/// segment, which results in a panic.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// The mapping from HexColour variants to specific colours.
    hex_colours: BTreeMap<HexColour, Colour>,
    /// The default hexagon border.
    ///
    /// Note that the fill colour is used by `n18brush` to fade non-active
    /// tiles, and should be partially transparent.
    pub hex_border: Draw,
    /// Barriers between hexes that affect track-building.
    pub hex_barrier: Draw,
    /// The border for highlighted hexagons.
    pub hex_highlight: Draw,
    /// The outer (background) style for track segments.
    pub track_outer: Draw,
    /// The inner (foreground) style for track segments.
    pub track_inner: Draw,
    /// The outer (background) style for token spaces.
    pub token_space_outer: Draw,
    /// The inner (foreground) style for token spaces.
    pub token_space_inner: Draw,
    /// The border for highlighted token spaces **and** highlighted cities.
    pub token_space_highlight: Draw,
    /// The border for circular dits.
    pub dit_circle: Draw,
    /// The outer (background) style for linear dits.
    pub dit_outer: Draw,
    /// The inner (foreground) style for linear dits.
    pub dit_inner: Draw,
    /// The outer (background) length of (linear) dits.
    pub dit_outer_length: Length,
    /// The inner (foreground) length of (linear) dits.
    pub dit_inner_length: Length,
    /// The radius of gentle track curves.
    pub track_gentle_radius: Length,
    /// The radius of hard track curves.
    pub track_hard_radius: Length,
    /// The radius of circular dits.
    pub dit_circle_radius: Length,
    /// The radius of token spaces.
    pub token_space_radius: Length,
    /// The border for circular labels (e.g., revenue labels).
    pub label_circle: Draw,
    /// A sequence of colours, intended for highlighting features such as
    /// routes on a map.
    /// Cycle through these colours with
    /// [`highlight_colours()`][Self::highlight_colours()].
    pub highlight_colours: Vec<Colour>,
    /// The margin when drawing a single tile by itself.
    pub tile_margin: Length,
    /// The border around the map edges (i.e., hexagon edges that are not
    /// adjacent to other hexagons).
    pub map_border: Draw,
    /// The width of the margin around each map edge.
    pub map_margin: Length,
    /// The text settings for tile name labels.
    pub tile_label: Text,
    /// The text settings for city name labels.
    pub city_label: Text,
    /// The text settings for Y labels.
    pub y_label: Text,
    /// The text settings for location labels.
    pub location_label: Text,
    /// The text settings for revenue labels.
    pub revenue_label: Text,
    /// The text settings for phase revenue labels.
    pub phase_revenue_label: Text,
    /// The text settings for token labels.
    pub token_label: Text,
    /// The horizontal margin for phase revenue labels.
    pub phase_revenue_margin_x: Length,
    /// The vertical margin for phase revenue labels.
    pub phase_revenue_margin_y: Length,
}

impl Default for Theme {
    fn default() -> Self {
        let hex_colours: BTreeMap<HexColour, Colour> = vec![
            // #dcbf11
            (HexColour::Yellow, Colour::from((220, 191, 17))),
            // #33b764
            (HexColour::Green, Colour::from((51, 183, 100))),
            // #ac6b3e
            (HexColour::Brown, Colour::from((172, 107, 62))),
            // #bdbcbc
            (HexColour::Grey, Colour::from((189, 188, 188))),
            // #dc3e3e
            (HexColour::Red, Colour::from((220, 62, 62))),
            // #0080f5
            (HexColour::Blue, Colour::from((0, 128, 245))),
            // #bddcbd
            (HexColour::Empty, Colour::from((189, 220, 189))),
        ]
        .into_iter()
        .collect();
        let highlight_colours = vec![
            Colour::from((179, 25, 25)),
            Colour::from((25, 179, 25)),
            Colour::from((25, 25, 179)),
        ];
        Theme {
            hex_colours,
            hex_border: Draw {
                width: Length::Relative(0.01),
                stroke: Colour::from((179, 179, 179)),
                fill: Colour::WHITE.with_alpha(63),
                ..Default::default()
            },
            hex_barrier: Draw {
                width: Length::Relative(0.05),
                stroke: Colour::from((25, 25, 153)),
                cap: cairo::LineCap::Round,
                ..Default::default()
            },
            hex_highlight: Draw {
                width: Length::Relative(0.02),
                stroke: Colour::from((255, 0, 0)),
                ..Default::default()
            },
            track_outer: Draw {
                width: Length::Relative(0.10),
                stroke: Colour::WHITE,
                ..Default::default()
            },
            track_inner: Draw {
                width: Length::Relative(0.08),
                stroke: Colour::BLACK,
                ..Default::default()
            },
            token_space_outer: Draw {
                width: Length::Relative(0.03),
                stroke: Colour::WHITE,
                fill: Colour::WHITE,
                ..Default::default()
            },
            token_space_inner: Draw {
                width: Length::Relative(0.01),
                stroke: Colour::BLACK,
                fill: Colour::WHITE,
                ..Default::default()
            },
            token_space_highlight: Draw {
                width: Length::Relative(0.025),
                stroke: Colour::from((255, 0, 0)),
                fill: Colour::WHITE,
                ..Default::default()
            },
            dit_circle: Draw {
                width: Length::Relative(0.01),
                stroke: Colour::WHITE,
                fill: Colour::BLACK,
                ..Default::default()
            },
            dit_outer: Draw {
                width: Length::Relative(0.10),
                stroke: Colour::WHITE,
                ..Default::default()
            },
            dit_inner: Draw {
                width: Length::Relative(0.08),
                stroke: Colour::BLACK,
                ..Default::default()
            },
            dit_outer_length: Length::Relative(0.11),
            dit_inner_length: Length::Relative(0.10),
            track_gentle_radius: Length::Relative(0.75),
            track_hard_radius: Length::Relative(0.25),
            dit_circle_radius: Length::Relative(0.085),
            token_space_radius: Length::Relative(0.125),
            label_circle: Draw {
                width: Length::Relative(0.01),
                stroke: Colour::BLACK,
                fill: Colour::WHITE,
                ..Default::default()
            },
            highlight_colours,
            tile_margin: Length::Relative(0.025),
            map_border: Draw {
                width: Length::Relative(0.05),
                stroke: Colour::BLACK,
                ..Default::default()
            },
            map_margin: Length::Absolute(10.0),
            tile_label: Text {
                family: FontFamily::Sans,
                font_size: 8.0,
                ..Default::default()
            },
            city_label: Text {
                family: FontFamily::Serif,
                font_size: 14.0,
                weight: pango::Weight::Bold,
                ..Default::default()
            },
            y_label: Text {
                family: FontFamily::Serif,
                font_size: 12.0,
                weight: pango::Weight::Bold,
                ..Default::default()
            },
            location_label: Text {
                family: FontFamily::Serif,
                font_size: 12.0,
                weight: pango::Weight::Bold,
                align: pango::Alignment::Center,
                horiz: AlignH::Centre,
                vert: AlignV::Top,
                max_width: Some(80.0),
                ..Default::default()
            },
            revenue_label: Text {
                family: FontFamily::Sans,
                font_size: 10.0,
                align: pango::Alignment::Center,
                ..Default::default()
            },
            phase_revenue_label: Text {
                family: FontFamily::Sans,
                font_size: 10.0,
                ..Default::default()
            },
            token_label: Text {
                family: FontFamily::Sans,
                font_size: 10.0,
                weight: pango::Weight::Bold,
                align: pango::Alignment::Center,
                horiz: AlignH::Centre,
                vert: AlignV::Middle,
                max_width: Some(30.0),
                ..Default::default()
            },
            phase_revenue_margin_x: Length::Absolute(2.0),
            phase_revenue_margin_y: Length::Absolute(1.0),
        }
    }
}

impl Theme {
    /// Sets a hexagon colour as the source pattern for the provided context.
    pub fn apply_hex_colour(&self, ctx: &Context, hc: HexColour) {
        let colour = self
            .hex_colour(hc)
            .unwrap_or_else(|| panic!("No colour defined for {:?}", hc));
        self.apply_colour(ctx, colour)
    }

    /// Returns an iterator that provides a sequence of colours, intended for
    /// highlighting features such as routes on a map.
    ///
    /// This iterator repeats without end, so it will always return a valid
    /// (but possibly repeated) colour.
    pub fn highlight_colours(&self) -> impl Iterator<Item = &Colour> {
        self.highlight_colours.iter().cycle()
    }

    /// Returns the `n`th colour in the (endless)
    /// [`highlight_colours()`](fn@Self::highlight_colours) series.
    pub fn nth_highlight_colour(&self, n: usize) -> Colour {
        let ix = n % self.highlight_colours.len();
        self.highlight_colours[ix]
    }

    /// Retrieves the colour associated with the provided hexagon background.
    pub fn hex_colour(&self, hc: HexColour) -> Option<Colour> {
        self.hex_colours.get(&hc).copied()
    }

    /// Defines the colour associated with the provided hexagon background.
    pub fn set_hex_colour(
        &mut self,
        hc: HexColour,
        colour: Colour,
    ) -> &mut Self {
        self.hex_colours.insert(hc, colour);
        self
    }

    /// Sets the source colour for the provided context.
    fn apply_colour<C>(&self, ctx: &Context, colour: C)
    where
        C: Into<Colour>,
    {
        colour.into().apply_colour(ctx)
    }
}

/// The scaling factor for converting u8 values to the unit interval.
static SCALE_U8_COLOUR: f64 = 1.0 / u8::MAX as f64;

/// Defines colours for drawing strokes and fills.
///
/// Each colour comprises four channels: red, green, blue, and alpha.
/// These values correspond to Cairo's 8-bit RGB colour space.
///
/// Colours can be constructed from 3-tuples (red, green, blue) and 4-tuples
/// (red, green, blue, alpha) of `u8` values (`0..=255`) or `f64` values
/// (`0.0..=1.0`).
/// When constructed from 3-tuples, the alpha channel is set to 255 (opaque).
///
/// ```
/// # use n18hex::theme::Colour;
/// let red = Colour::from((255, 0, 0));
/// let green = Colour::from((0.0, 1.0, 0.0));
/// let blue = Colour::from((0, 0, 255, 255));
/// let black = Colour::from((0.0, 0.0, 0.0, 1.0));
/// ```
///
/// Colours can also be constructed from valid hexadecimal strings:
///
/// ```
/// # use n18hex::theme::Colour;
/// let red = "#ff0000".parse::<Colour>().unwrap();
/// let translucent_blue = "#0000ff7f".parse::<Colour>().unwrap();
/// ```
///
/// Attempts to parse invalid strings will return a [ParseColourError] value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Colour {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Colour {
    /// The colour white ("#FFFFFF").
    pub const WHITE: Self = Self {
        red: u8::MAX,
        green: u8::MAX,
        blue: u8::MAX,
        alpha: u8::MAX,
    };

    /// The colour black ("#000000").
    pub const BLACK: Self = Self {
        red: 0,
        green: 0,
        blue: 0,
        alpha: u8::MAX,
    };

    /// Entirely transparent.
    pub const TRANSPARENT: Self = Self {
        red: u8::MAX,
        green: u8::MAX,
        blue: u8::MAX,
        alpha: 0,
    };

    /// Adjust the transparency of this colour.
    pub fn with_alpha(mut self, alpha: u8) -> Self {
        self.alpha = alpha;
        self
    }

    /// Adjust the transparency of this colour (`0.0..=1.0`).
    pub fn with_alpha_f64(mut self, alpha: f64) -> Self {
        self.alpha = (alpha.clamp(0.0, 1.0) * u8::MAX as f64).round() as u8;
        self
    }

    /// Use this colour as the source for the provided context.
    pub fn apply_colour(&self, ctx: &Context) {
        let r = self.red as f64 * SCALE_U8_COLOUR;
        let g = self.green as f64 * SCALE_U8_COLOUR;
        let b = self.blue as f64 * SCALE_U8_COLOUR;
        let a = self.alpha as f64 * SCALE_U8_COLOUR;
        ctx.set_source_rgba(r, g, b, a)
    }

    /// Returns the hexadecimal representation of this colour, without the
    /// alpha channel.
    pub fn as_rgb(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }

    /// Returns the hexadecimal representation of this colour, including the
    /// alpha channel.
    pub fn as_rgba(&self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            self.red, self.green, self.blue, self.alpha
        )
    }
}

impl std::fmt::Display for Colour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_rgba())
    }
}

impl From<&Colour> for Colour {
    fn from(src: &Colour) -> Self {
        *src
    }
}

impl From<(f64, f64, f64)> for Colour {
    fn from(src: (f64, f64, f64)) -> Self {
        let fmax = u8::MAX as f64;
        let r = (src.0 * fmax).round() as u8;
        let g = (src.1 * fmax).round() as u8;
        let b = (src.2 * fmax).round() as u8;
        Colour::from((r, g, b))
    }
}

impl From<(f64, f64, f64, f64)> for Colour {
    fn from(src: (f64, f64, f64, f64)) -> Self {
        let fmax = u8::MAX as f64;
        let r = (src.0 * fmax).round() as u8;
        let g = (src.1 * fmax).round() as u8;
        let b = (src.2 * fmax).round() as u8;
        let a = (src.3 * fmax).round() as u8;
        Colour::from((r, g, b, a))
    }
}

impl From<(u8, u8, u8)> for Colour {
    fn from(src: (u8, u8, u8)) -> Self {
        Colour {
            red: src.0,
            green: src.1,
            blue: src.2,
            alpha: u8::MAX,
        }
    }
}

impl From<(u8, u8, u8, u8)> for Colour {
    fn from(src: (u8, u8, u8, u8)) -> Self {
        Self::from((src.0, src.1, src.2)).with_alpha(src.3)
    }
}

/// An error which can be returned when parsing colour strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseColourError {
    input: String,
}

impl std::error::Error for ParseColourError {}

impl std::fmt::Display for ParseColourError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid colour string: {:?}", self.input)
    }
}

impl From<std::num::ParseIntError> for ParseColourError {
    fn from(src: std::num::ParseIntError) -> Self {
        let input = format!("{}", src);
        ParseColourError { input }
    }
}

impl std::str::FromStr for Colour {
    type Err = ParseColourError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num_bytes = s.len();
        let invalid_len = num_bytes != 7 && num_bytes != 9;
        let invalid_start = &s[0..1] != "#";
        if invalid_len || invalid_start {
            return Err(ParseColourError {
                input: s.to_string(),
            });
        }
        let red = u8::from_str_radix(&s[1..3], 16)?;
        let green = u8::from_str_radix(&s[3..5], 16)?;
        let blue = u8::from_str_radix(&s[5..7], 16)?;
        let alpha = if num_bytes == 9 {
            u8::from_str_radix(&s[7..9], 16)?
        } else {
            u8::MAX
        };
        Ok(Colour::from((red, green, blue, alpha)))
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::error::Error;

    /// Converts a HexColour into a Colour and prints this Colour as a
    /// hexadecimal triplet.
    fn to_rgb(theme: &Theme, hc: HexColour) {
        let colour = theme
            .hex_colour(hc)
            .unwrap_or_else(|| panic!("No colour defined for {:?}", hc));
        let rgb_string = colour.as_rgb();
        println!("{:6} = {}", format!("{:?}", hc), rgb_string)
    }

    #[test]
    /// Prints the Colour associated with each HexColour variant.
    fn hex_colour_to_rgb() {
        use crate::HexColour::*;
        let theme = Theme::default();
        to_rgb(&theme, Yellow);
        to_rgb(&theme, Green);
        to_rgb(&theme, Brown);
        to_rgb(&theme, Grey);
        to_rgb(&theme, Red);
        to_rgb(&theme, Blue);
        to_rgb(&theme, Empty);
    }

    #[test]
    /// Tests the round-trip between RGBA strings and Colours.
    fn rgba_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let input = "#dbbf12aa";
        let colour: Colour = input.parse()?;
        let output = colour.as_rgba();
        assert_eq!(input, output);
        Ok(())
    }

    #[test]
    /// Tests the round-trip between RGBA strings and Colours.
    fn rgba_invalid_string() -> Result<(), Box<dyn Error>> {
        let input = "#dbbf12AAA";
        assert!(input.parse::<Colour>().is_err());
        Ok(())
    }

    #[test]
    /// Tests parsing a range of valid RGB and RGBA strings.
    ///
    /// The original version of this test case covered every valid input
    /// string, but this took 13 seconds to run when compiled in release mode.
    fn rgb_rgba_inputs() -> Result<(), Box<dyn Error>> {
        let in_vals: [u8; 8] = [0, 1, 63, 127, 128, 191, 254, 255];
        for r in &in_vals {
            for g in &in_vals {
                for b in &in_vals {
                    // Parse the #RRGGBB input string.
                    let input = format!("#{:02x}{:02x}{:02x}", r, g, b);
                    let parsed = input.parse::<Colour>();
                    assert!(parsed.is_ok());
                    let colour = parsed.unwrap();
                    // Ensure we obtain the expected colour values.
                    assert_eq!(*r, colour.red);
                    assert_eq!(*g, colour.green);
                    assert_eq!(*b, colour.blue);
                    for a in &in_vals {
                        // Parse the #RRGGBBAA input string.
                        let input_a = format!("{}{:02x}", input, a);
                        let parsed_a = input_a.parse::<Colour>();
                        assert!(parsed_a.is_ok());
                        let colour_a = parsed_a.unwrap();
                        // Ensure we obtain the expected colour values.
                        assert_eq!(colour.red, colour_a.red);
                        assert_eq!(colour.green, colour_a.green);
                        assert_eq!(colour.blue, colour_a.blue);
                        assert_eq!(*a, colour_a.alpha);
                    }
                }
            }
        }
        Ok(())
    }
}
