use cairo::{Context, FontSlant, FontWeight};
use n18hex::consts::*;
use n18hex::Hex;

/// The collection of tokens associated with each company.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tokens {
    names: Vec<String>,
    tokens: Vec<Token>,
    count: usize,
}

impl From<Vec<(String, Token)>> for Tokens {
    fn from(src: Vec<(String, Token)>) -> Self {
        let mut names = vec![];
        let mut tokens = vec![];
        let count = src.len();
        for (name, token) in src.into_iter() {
            names.push(name);
            tokens.push(token);
        }
        Self {
            names,
            tokens,
            count,
        }
    }
}

impl From<Tokens> for Vec<(String, Token)> {
    fn from(src: Tokens) -> Self {
        src.names.into_iter().zip(src.tokens.into_iter()).collect()
    }
}

impl Tokens {
    pub fn new(src: Vec<(String, Token)>) -> Self {
        src.into()
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn first_token(&self) -> Token {
        self.tokens[0]
    }

    pub fn last_token(&self) -> Token {
        self.tokens[self.count - 1]
    }

    pub fn prev_token(&self, token: &Token) -> Option<Token> {
        self.tokens
            .iter()
            .enumerate()
            .find(|(_ix, tok)| tok == &token)
            .map(|(ix, _tok)| if ix > 0 { ix - 1 } else { self.count - 1 })
            .map(|ix| self.tokens[ix])
    }

    pub fn next_token(&self, token: &Token) -> Option<Token> {
        self.tokens
            .iter()
            .enumerate()
            .find(|(_ix, tok)| tok == &token)
            .map(|(ix, _tok)| if ix < self.count - 1 { ix + 1 } else { 0 })
            .map(|ix| self.tokens[ix])
    }

    pub fn get_token(&self, name: &str) -> Option<&Token> {
        self.names
            .iter()
            .enumerate()
            .find(|(_ix, n)| n == &name)
            .map(|(ix, _n)| &self.tokens[ix])
    }

    pub fn get_name(&self, token: &Token) -> Option<&str> {
        self.tokens
            .iter()
            .enumerate()
            .find(|(_ix, t)| t == &token)
            .map(|(ix, _t)| self.names[ix].as_str())
    }
}

/// A token that may occupy a token space on a `Tile`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Token {
    pub style: TokenStyle,
    pub x_pcnt: usize,
    pub y_pcnt: usize,
}

/// Define the colour palette for each token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Colour {
    /// The amount of red, must be between 0 and 255 (inclusive).
    pub red: usize,
    /// The amount of green, must be between 0 and 255 (inclusive).
    pub green: usize,
    /// The amount of blue, must be between 0 and 255 (inclusive).
    pub blue: usize,
    /// The alpha transparency, must be between 0 and 100 (inclusive).
    pub alpha: Option<usize>,
}

impl Colour {
    pub fn apply_to(&self, ctx: &Context) {
        let r = self.red as f64 / 255.0;
        let g = self.green as f64 / 255.0;
        let b = self.blue as f64 / 255.0;
        match self.alpha {
            Some(a) => ctx.set_source_rgba(r, g, b, a as f64 / 100.0),
            None => ctx.set_source_rgb(r, g, b),
        }
    }

    pub fn with_alpha(mut self, alpha: usize) -> Self {
        self.alpha = Some(alpha);
        self
    }
}

impl From<(usize, usize, usize)> for Colour {
    fn from(src: (usize, usize, usize)) -> Self {
        Colour {
            red: src.0,
            green: src.1,
            blue: src.2,
            alpha: None,
        }
    }
}

impl From<(usize, usize, usize, usize)> for Colour {
    fn from(src: (usize, usize, usize, usize)) -> Self {
        Colour {
            red: src.0,
            green: src.1,
            blue: src.2,
            alpha: Some(src.3),
        }
    }
}

/// Define the appearance of each token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TokenStyle {
    // TODO: add TopArcs, SideSquares, TopSquares, SideLines, TopLines, etc.
    // See https://boardgamegeek.com/image/5033873/18611867 for ideas.
    SideArcs {
        bg: Colour,
        fg: Colour,
        text: Colour,
    },
    TopArcs {
        bg: Colour,
        fg: Colour,
        text: Colour,
    },
    TopSquares {
        bg: Colour,
        fg: Colour,
        text: Colour,
    },
    TopLines {
        bg: Colour,
        fg: Colour,
        text: Colour,
    },
    TopTriangles {
        bg: Colour,
        fg: Colour,
        text: Colour,
    },
    TripleTriangles {
        bg: Colour,
        fg: Colour,
        text: Colour,
    },
    TribandV {
        sides: Colour,
        middle: Colour,
        text: Colour,
    },
    TribandH {
        sides: Colour,
        middle: Colour,
        text: Colour,
    },
    TricolourV {
        left: Colour,
        middle: Colour,
        right: Colour,
        text: Colour,
    },
    TricolourH {
        top: Colour,
        middle: Colour,
        bottom: Colour,
        text: Colour,
    },
}

impl TokenStyle {
    fn draw_background(&self, hex: &Hex, ctx: &Context) {
        use TokenStyle::*;

        let radius = hex.max_d * 0.125;
        let rmax = 1.1 * radius;
        let dx = 0.45 * radius;

        match self {
            SideArcs { fg, bg, .. } => {
                bg.apply_to(ctx);
                ctx.fill_preserve();

                ctx.clip_preserve();
                ctx.new_path();
                ctx.arc(-1.5 * radius, 0.0, 1.0 * radius, 0.0, 2.0 * PI);
                ctx.arc(1.5 * radius, 0.0, 1.0 * radius, 0.0, 2.0 * PI);

                fg.apply_to(ctx);
                ctx.fill();
            }
            TopArcs { fg, bg, .. } => {
                bg.apply_to(ctx);
                ctx.fill_preserve();

                ctx.clip_preserve();
                ctx.new_path();
                ctx.arc(0.0, -1.5 * radius, 1.0 * radius, 0.0, 2.0 * PI);
                ctx.arc(0.0, 1.5 * radius, 1.0 * radius, 0.0, 2.0 * PI);

                fg.apply_to(ctx);
                ctx.fill();
            }
            TopSquares { fg, bg, .. } => {
                bg.apply_to(ctx);
                ctx.fill_preserve();
                ctx.clip_preserve();
                ctx.new_path();
                ctx.move_to(-0.4 * radius, -dx);
                ctx.line_to(0.4 * radius, -dx);
                ctx.line_to(0.4 * radius, -rmax);
                ctx.line_to(-0.4 * radius, -rmax);
                fg.apply_to(ctx);
                ctx.fill();
                ctx.new_path();
                ctx.move_to(-0.4 * radius, dx);
                ctx.line_to(0.4 * radius, dx);
                ctx.line_to(0.4 * radius, rmax);
                ctx.line_to(-0.4 * radius, rmax);
                fg.apply_to(ctx);
                ctx.fill();
            }
            TopLines { fg, bg, .. } => {
                bg.apply_to(ctx);
                ctx.fill_preserve();
                ctx.clip_preserve();
                ctx.new_path();
                ctx.move_to(-0.5 * radius, -dx);
                ctx.line_to(-0.3 * radius, -dx);
                ctx.line_to(-0.3 * radius, -rmax);
                ctx.line_to(-0.5 * radius, -rmax);
                ctx.move_to(-0.1 * radius, -dx);
                ctx.line_to(0.1 * radius, -dx);
                ctx.line_to(0.1 * radius, -rmax);
                ctx.line_to(-0.1 * radius, -rmax);
                ctx.move_to(0.5 * radius, -dx);
                ctx.line_to(0.3 * radius, -dx);
                ctx.line_to(0.3 * radius, -rmax);
                ctx.line_to(0.5 * radius, -rmax);
                fg.apply_to(ctx);
                ctx.fill();
                ctx.new_path();
                ctx.move_to(-0.5 * radius, dx);
                ctx.line_to(-0.3 * radius, dx);
                ctx.line_to(-0.3 * radius, rmax);
                ctx.line_to(-0.5 * radius, rmax);
                ctx.move_to(-0.1 * radius, dx);
                ctx.line_to(0.1 * radius, dx);
                ctx.line_to(0.1 * radius, rmax);
                ctx.line_to(-0.1 * radius, rmax);
                ctx.move_to(0.5 * radius, dx);
                ctx.line_to(0.3 * radius, dx);
                ctx.line_to(0.3 * radius, rmax);
                ctx.line_to(0.5 * radius, rmax);
                fg.apply_to(ctx);
                ctx.fill();
            }
            TopTriangles { fg, bg, .. } => {
                bg.apply_to(ctx);
                ctx.fill_preserve();
                ctx.clip_preserve();
                ctx.new_path();
                ctx.move_to(-dx, -rmax);
                ctx.line_to(0.0, -dx);
                ctx.line_to(dx, -rmax);
                fg.apply_to(ctx);
                ctx.fill();
                ctx.new_path();
                ctx.move_to(-dx, rmax);
                ctx.line_to(0.0, dx);
                ctx.line_to(dx, rmax);
                fg.apply_to(ctx);
                ctx.fill();
            }
            TripleTriangles { fg, bg, .. } => {
                bg.apply_to(ctx);
                ctx.fill_preserve();
                ctx.clip_preserve();
                ctx.new_path();
                ctx.move_to(-1.5 * radius, -rmax);
                ctx.line_to(-0.3 * radius, -dx);
                ctx.line_to(-0.3 * radius, -rmax);
                ctx.line_to(0.0, -0.5 * radius);
                ctx.line_to(0.3 * radius, -rmax);
                ctx.line_to(0.3 * radius, -dx);
                ctx.line_to(1.5 * radius, -rmax);
                fg.apply_to(ctx);
                ctx.fill();
                ctx.new_path();
                ctx.move_to(-1.5 * radius, rmax);
                ctx.line_to(-0.3 * radius, dx);
                ctx.line_to(-0.3 * radius, rmax);
                ctx.line_to(0.0, 0.5 * radius);
                ctx.line_to(0.3 * radius, rmax);
                ctx.line_to(0.3 * radius, dx);
                ctx.line_to(1.5 * radius, rmax);
                fg.apply_to(ctx);
                ctx.fill();
            }
            TribandV { sides, middle, .. } => {
                sides.apply_to(ctx);
                ctx.fill_preserve();
                ctx.clip_preserve();
                ctx.new_path();
                ctx.move_to(-dx, rmax);
                ctx.line_to(-dx, -rmax);
                ctx.line_to(dx, -rmax);
                ctx.line_to(dx, rmax);
                ctx.line_to(-dx, rmax);
                middle.apply_to(ctx);
                ctx.fill();
            }
            TribandH { sides, middle, .. } => {
                sides.apply_to(ctx);
                ctx.fill_preserve();
                ctx.clip_preserve();
                ctx.new_path();
                ctx.move_to(rmax, -dx);
                ctx.line_to(-rmax, -dx);
                ctx.line_to(-rmax, dx);
                ctx.line_to(rmax, dx);
                ctx.line_to(rmax, -dx);
                middle.apply_to(ctx);
                ctx.fill();
            }
            TricolourV {
                left,
                middle,
                right,
                ..
            } => {
                // Fill the entire region with the middle colour.
                middle.apply_to(ctx);
                ctx.fill_preserve();
                ctx.clip_preserve();
                // Define the left region and fill with the left colour.
                ctx.new_path();
                ctx.move_to(-rmax, -rmax);
                ctx.line_to(-rmax, rmax);
                ctx.line_to(-dx, rmax);
                ctx.line_to(-dx, -rmax);
                left.apply_to(ctx);
                ctx.fill();
                // Define the right region and fill with the right colour.
                ctx.new_path();
                ctx.move_to(rmax, -rmax);
                ctx.line_to(rmax, rmax);
                ctx.line_to(dx, rmax);
                ctx.line_to(dx, -rmax);
                right.apply_to(ctx);
                ctx.fill();
            }
            TricolourH {
                top,
                middle,
                bottom,
                ..
            } => {
                // Fill the entire region with the middle colour.
                middle.apply_to(ctx);
                ctx.fill_preserve();
                ctx.clip_preserve();
                // Define the top region and fill with the top colour.
                ctx.new_path();
                ctx.move_to(-rmax, -dx);
                ctx.line_to(-rmax, -rmax);
                ctx.line_to(rmax, -rmax);
                ctx.line_to(rmax, -dx);
                top.apply_to(ctx);
                ctx.fill();
                // Define the bottom region and fill with the bottom colour.
                ctx.new_path();
                ctx.move_to(-rmax, dx);
                ctx.line_to(-rmax, rmax);
                ctx.line_to(rmax, rmax);
                ctx.line_to(rmax, dx);
                bottom.apply_to(ctx);
                ctx.fill();
            }
        }
    }

    pub fn text_colour(&self) -> &Colour {
        use TokenStyle::*;

        match self {
            SideArcs { text, .. } => &text,
            TopArcs { text, .. } => &text,
            TopSquares { text, .. } => &text,
            TopLines { text, .. } => &text,
            TopTriangles { text, .. } => &text,
            TripleTriangles { text, .. } => &text,
            TribandV { text, .. } => &text,
            TribandH { text, .. } => &text,
            TricolourV { text, .. } => &text,
            TricolourH { text, .. } => &text,
        }
    }
}

impl Token {
    pub fn new(style: TokenStyle) -> Self {
        Self {
            style,
            x_pcnt: 50,
            y_pcnt: 50,
        }
    }

    pub fn shift_text(mut self, x_pcnt: usize, y_pcnt: usize) -> Self {
        self.x_pcnt = x_pcnt;
        self.y_pcnt = y_pcnt;
        self
    }

    fn draw_text(&self, hex: &Hex, ctx: &Context, text: &str) {
        // NOTE: scale font size relative to hex diameter.
        ctx.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
        let scale = hex.max_d / 125.0;
        ctx.set_font_size(10.0 * scale);

        // NOTE: use pango to draw the label on tokens, so that we can specify
        // a maximum width and wrap the text, and support line breaks for
        // token names that are deliberately split over multiple lines.
        let mut font_descr = pango::FontDescription::new();
        font_descr.set_family("Sans");
        font_descr.set_absolute_size(10.0 * scale * pango::SCALE as f64);
        font_descr.set_style(pango::Style::Normal);
        font_descr.set_weight(pango::Weight::Bold);

        let layout = pangocairo::create_layout(ctx)
            .expect("Could not create Pango layout");
        layout.set_font_description(Some(&font_descr));
        layout.set_text(text);

        // Determine the logical extents of the token text.
        let size = layout.get_pixel_extents().1;

        // Move to the appropriate starting location so that the text is
        // centred at the desired location.
        let dx = -0.5 * size.width as f64 - size.x as f64;
        let dy = -0.5 * size.height as f64 - size.y as f64;

        // Shift the text according to the values of `x_pcnt` and `y_pcnt`.
        let radius = hex.max_d * 0.125;
        let x = (radius + dx) * ((self.x_pcnt as f64 - 50.0) / 50.0);
        let y = (radius + dy) * ((self.y_pcnt as f64 - 50.0) / 50.0);

        // Draw the token label.
        self.style.text_colour().apply_to(ctx);
        layout.set_width((30.0 * scale) as i32 * pango::SCALE);
        layout.set_alignment(pango::Alignment::Center);
        layout.set_wrap(pango::WrapMode::Word);
        pangocairo::update_layout(ctx, &layout);
        let (_ink_rect, logical_rect) = layout.get_pixel_extents();
        let (dx, dy) = layout.get_pixel_size();
        let nudge_fac = 0.5;
        ctx.move_to(
            x - nudge_fac * dx as f64 - logical_rect.x as f64,
            y - nudge_fac * dy as f64 - logical_rect.y as f64,
        );
        pangocairo::show_layout(ctx, &layout);
    }

    /// Draws the token so that it fills the current path.
    ///
    /// Define the token boundary before calling this function.
    pub fn draw(&self, hex: &Hex, ctx: &Context, text: &str, rotn: f64) {
        // Locate the centre of the token.
        let (x0, y0, x1, y1) = ctx.fill_extents();
        let x = 0.5 * (x0 + x1);
        let y = 0.5 * (y0 + y1);

        let m = ctx.get_matrix();
        ctx.save();

        // NOTE: move to the token centre and apply the inverse rotation.
        ctx.translate(x, y);
        ctx.rotate(-rotn);

        let stroke_path = ctx.copy_path();
        self.style.draw_background(hex, ctx);
        self.draw_text(hex, ctx, text);

        // Redraw the outer black circle.
        ctx.new_path();
        ctx.append_path(&stroke_path);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.set_line_width(hex.max_d * 0.01);
        ctx.stroke_preserve();

        ctx.restore();
        ctx.set_matrix(m);
    }
}
