use cairo::{Context, FontSlant, FontWeight};
use rusty_hex::consts::*;
use rusty_hex::Hex;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    pub style: TokenStyle,
    pub x_pcnt: usize,
    pub y_pcnt: usize,
}

/// Define the colour palette for each token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Define the appearance of each token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenStyle {
    // TODO: add TopArcs, SizeSquares, TopSquares, SideLines, TopLines, etc.
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
}

impl TokenStyle {
    pub fn draw_background(&self, hex: &Hex, ctx: &Context) {
        use TokenStyle::*;

        // Locate the centre of the token.
        let (x0, y0, x1, y1) = ctx.fill_extents();
        let x = 0.5 * (x0 + x1);
        let y = 0.5 * (y0 + y1);

        match self {
            SideArcs { fg, bg, .. } => {
                bg.apply_to(ctx);
                ctx.fill_preserve();

                ctx.clip_preserve();
                let radius = hex.max_d * 0.125;
                ctx.new_path();
                ctx.arc(x - 1.5 * radius, y, 1.0 * radius, 0.0, 2.0 * PI);
                ctx.arc(x + 1.5 * radius, y, 1.0 * radius, 0.0, 2.0 * PI);

                fg.apply_to(ctx);
                ctx.fill_preserve();
            }
            TopArcs { fg, bg, .. } => {
                bg.apply_to(ctx);
                ctx.fill_preserve();

                ctx.clip_preserve();
                let radius = hex.max_d * 0.125;
                ctx.new_path();
                ctx.arc(x, y - 1.5 * radius, 1.0 * radius, 0.0, 2.0 * PI);
                ctx.arc(x, y + 1.5 * radius, 1.0 * radius, 0.0, 2.0 * PI);

                fg.apply_to(ctx);
                ctx.fill_preserve();
            }
        }
    }

    pub fn text_colour(&self) -> &Colour {
        use TokenStyle::*;

        match self {
            SideArcs { text, .. } => &text,
            TopArcs { text, .. } => &text,
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

    fn draw_text(&self, hex: &Hex, ctx: &Context, text: &str) {
        // Locate the centre of the token.
        let (x0, y0, x1, y1) = ctx.fill_extents();
        let x = 0.5 * (x0 + x1);
        let y = 0.5 * (y0 + y1);

        // NOTE: allow the text position to be adjusted.
        let x = x * (self.x_pcnt as f64 / 50.0);
        let y = y * (self.y_pcnt as f64 / 50.0);

        // NOTE: scale font size relative to hex diameter.
        ctx.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
        let scale = hex.max_d / 125.0;
        ctx.set_font_size(10.0 * scale);
        let exts = ctx.text_extents(text);
        let x = x - 0.5 * exts.width;
        let y = y + 0.5 * exts.height;

        // Draw the token label.
        self.style.text_colour().apply_to(ctx);
        ctx.move_to(x, y);
        ctx.show_text(text);
    }

    pub fn draw(&self, hex: &Hex, ctx: &Context, text: &str) {
        let stroke_path = ctx.copy_path();
        ctx.save();
        self.style.draw_background(hex, ctx);
        ctx.restore();

        ctx.save();
        self.draw_text(hex, ctx, text);
        ctx.restore();

        // Redraw the outer black circle.
        ctx.save();
        ctx.new_path();
        ctx.append_path(&stroke_path);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.set_line_width(hex.max_d * 0.01);
        ctx.stroke_preserve();
        ctx.restore();
    }
}
