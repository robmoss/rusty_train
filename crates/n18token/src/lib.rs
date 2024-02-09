use cairo::Context;
use n18hex::consts::*;
use n18hex::{Colour, Hex};

/// The collection of tokens associated with each company.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
        src.names.into_iter().zip(src.tokens).collect()
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

    pub fn token(&self, name: &str) -> Option<&Token> {
        self.names
            .iter()
            .enumerate()
            .find(|(_ix, n)| n == &name)
            .map(|(ix, _n)| &self.tokens[ix])
    }

    pub fn name(&self, token: &Token) -> Option<&str> {
        self.tokens
            .iter()
            .enumerate()
            .find(|(_ix, t)| t == &token)
            .map(|(ix, _t)| self.names[ix].as_str())
    }
}

/// A token that may occupy a token space on a `Tile`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
    pub style: TokenStyle,
    pub x_pcnt: usize,
    pub y_pcnt: usize,
}

/// Define the appearance of each token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

        let radius = hex.theme.token_space_radius.absolute(hex);
        let rmax = 1.1 * radius;
        let dx = 0.45 * radius;

        match self {
            SideArcs { fg, bg, .. } => {
                bg.apply_colour(ctx);
                ctx.fill_preserve().unwrap();

                ctx.clip_preserve();
                ctx.new_path();
                ctx.arc(-1.5 * radius, 0.0, 1.0 * radius, 0.0, 2.0 * PI);
                ctx.arc(1.5 * radius, 0.0, 1.0 * radius, 0.0, 2.0 * PI);

                fg.apply_colour(ctx);
                ctx.fill().unwrap();
            }
            TopArcs { fg, bg, .. } => {
                bg.apply_colour(ctx);
                ctx.fill_preserve().unwrap();

                ctx.clip_preserve();
                ctx.new_path();
                ctx.arc(0.0, -1.5 * radius, 1.0 * radius, 0.0, 2.0 * PI);
                ctx.arc(0.0, 1.5 * radius, 1.0 * radius, 0.0, 2.0 * PI);

                fg.apply_colour(ctx);
                ctx.fill().unwrap();
            }
            TopSquares { fg, bg, .. } => {
                bg.apply_colour(ctx);
                ctx.fill_preserve().unwrap();
                ctx.clip_preserve();
                ctx.new_path();
                ctx.move_to(-0.4 * radius, -dx);
                ctx.line_to(0.4 * radius, -dx);
                ctx.line_to(0.4 * radius, -rmax);
                ctx.line_to(-0.4 * radius, -rmax);
                fg.apply_colour(ctx);
                ctx.fill().unwrap();
                ctx.new_path();
                ctx.move_to(-0.4 * radius, dx);
                ctx.line_to(0.4 * radius, dx);
                ctx.line_to(0.4 * radius, rmax);
                ctx.line_to(-0.4 * radius, rmax);
                fg.apply_colour(ctx);
                ctx.fill().unwrap();
            }
            TopLines { fg, bg, .. } => {
                bg.apply_colour(ctx);
                ctx.fill_preserve().unwrap();
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
                fg.apply_colour(ctx);
                ctx.fill().unwrap();
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
                fg.apply_colour(ctx);
                ctx.fill().unwrap();
            }
            TopTriangles { fg, bg, .. } => {
                bg.apply_colour(ctx);
                ctx.fill_preserve().unwrap();
                ctx.clip_preserve();
                ctx.new_path();
                ctx.move_to(-dx, -rmax);
                ctx.line_to(0.0, -dx);
                ctx.line_to(dx, -rmax);
                fg.apply_colour(ctx);
                ctx.fill().unwrap();
                ctx.new_path();
                ctx.move_to(-dx, rmax);
                ctx.line_to(0.0, dx);
                ctx.line_to(dx, rmax);
                fg.apply_colour(ctx);
                ctx.fill().unwrap();
            }
            TripleTriangles { fg, bg, .. } => {
                bg.apply_colour(ctx);
                ctx.fill_preserve().unwrap();
                ctx.clip_preserve();
                ctx.new_path();
                ctx.move_to(-1.5 * radius, -rmax);
                ctx.line_to(-0.3 * radius, -dx);
                ctx.line_to(-0.3 * radius, -rmax);
                ctx.line_to(0.0, -0.5 * radius);
                ctx.line_to(0.3 * radius, -rmax);
                ctx.line_to(0.3 * radius, -dx);
                ctx.line_to(1.5 * radius, -rmax);
                fg.apply_colour(ctx);
                ctx.fill().unwrap();
                ctx.new_path();
                ctx.move_to(-1.5 * radius, rmax);
                ctx.line_to(-0.3 * radius, dx);
                ctx.line_to(-0.3 * radius, rmax);
                ctx.line_to(0.0, 0.5 * radius);
                ctx.line_to(0.3 * radius, rmax);
                ctx.line_to(0.3 * radius, dx);
                ctx.line_to(1.5 * radius, rmax);
                fg.apply_colour(ctx);
                ctx.fill().unwrap();
            }
            TribandV { sides, middle, .. } => {
                sides.apply_colour(ctx);
                ctx.fill_preserve().unwrap();
                ctx.clip_preserve();
                ctx.new_path();
                ctx.move_to(-dx, rmax);
                ctx.line_to(-dx, -rmax);
                ctx.line_to(dx, -rmax);
                ctx.line_to(dx, rmax);
                ctx.line_to(-dx, rmax);
                middle.apply_colour(ctx);
                ctx.fill().unwrap();
            }
            TribandH { sides, middle, .. } => {
                sides.apply_colour(ctx);
                ctx.fill_preserve().unwrap();
                ctx.clip_preserve();
                ctx.new_path();
                ctx.move_to(rmax, -dx);
                ctx.line_to(-rmax, -dx);
                ctx.line_to(-rmax, dx);
                ctx.line_to(rmax, dx);
                ctx.line_to(rmax, -dx);
                middle.apply_colour(ctx);
                ctx.fill().unwrap();
            }
            TricolourV {
                left,
                middle,
                right,
                ..
            } => {
                // Fill the entire region with the middle colour.
                middle.apply_colour(ctx);
                ctx.fill_preserve().unwrap();
                ctx.clip_preserve();
                // Define the left region and fill with the left colour.
                ctx.new_path();
                ctx.move_to(-rmax, -rmax);
                ctx.line_to(-rmax, rmax);
                ctx.line_to(-dx, rmax);
                ctx.line_to(-dx, -rmax);
                left.apply_colour(ctx);
                ctx.fill().unwrap();
                // Define the right region and fill with the right colour.
                ctx.new_path();
                ctx.move_to(rmax, -rmax);
                ctx.line_to(rmax, rmax);
                ctx.line_to(dx, rmax);
                ctx.line_to(dx, -rmax);
                right.apply_colour(ctx);
                ctx.fill().unwrap();
            }
            TricolourH {
                top,
                middle,
                bottom,
                ..
            } => {
                // Fill the entire region with the middle colour.
                middle.apply_colour(ctx);
                ctx.fill_preserve().unwrap();
                ctx.clip_preserve();
                // Define the top region and fill with the top colour.
                ctx.new_path();
                ctx.move_to(-rmax, -dx);
                ctx.line_to(-rmax, -rmax);
                ctx.line_to(rmax, -rmax);
                ctx.line_to(rmax, -dx);
                top.apply_colour(ctx);
                ctx.fill().unwrap();
                // Define the bottom region and fill with the bottom colour.
                ctx.new_path();
                ctx.move_to(-rmax, dx);
                ctx.line_to(-rmax, rmax);
                ctx.line_to(rmax, rmax);
                ctx.line_to(rmax, dx);
                bottom.apply_colour(ctx);
                ctx.fill().unwrap();
            }
        }
    }

    pub fn text_colour(&self) -> &Colour {
        use TokenStyle::*;

        match self {
            SideArcs { text, .. } => text,
            TopArcs { text, .. } => text,
            TopSquares { text, .. } => text,
            TopLines { text, .. } => text,
            TopTriangles { text, .. } => text,
            TripleTriangles { text, .. } => text,
            TribandV { text, .. } => text,
            TribandH { text, .. } => text,
            TricolourV { text, .. } => text,
            TricolourH { text, .. } => text,
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
        // Draw the token text using the appropriate theme settings.
        let mut labeller = hex.theme.token_label.labeller(ctx, hex);

        // Ensure the text is centred and has the desired colour.
        labeller.halign(n18hex::theme::AlignH::Centre);
        labeller.valign(n18hex::theme::AlignV::Middle);
        labeller.colour(*self.style.text_colour());

        // Identify the location of the text centre, noting that the current
        // point is the token centre.
        let (x, y) = ctx.current_point().unwrap();
        let radius = hex.theme.token_space_radius.absolute(hex);
        let dx = radius * ((self.x_pcnt as f64 - 50.0) / 50.0);
        let dy = radius * ((self.y_pcnt as f64 - 50.0) / 50.0);
        let text_centre = n18hex::Coord::from((x + dx, y + dy));

        labeller.draw(text, text_centre);
    }

    /// Draws the token so that it fills the current path.
    ///
    /// Define the token boundary before calling this function.
    pub fn draw(&self, hex: &Hex, ctx: &Context, text: &str, rotn: f64) {
        // Locate the centre of the token.
        let (x0, y0, x1, y1) = ctx.fill_extents().unwrap();
        let x = 0.5 * (x0 + x1);
        let y = 0.5 * (y0 + y1);

        let m = ctx.matrix();
        ctx.save().unwrap();

        // NOTE: move to the token centre and apply the inverse rotation.
        ctx.translate(x, y);
        ctx.rotate(-rotn);

        let stroke_path = ctx.copy_path().unwrap();
        self.style.draw_background(hex, ctx);
        self.draw_text(hex, ctx, text);

        // Redraw the outer black circle.
        ctx.new_path();
        ctx.append_path(&stroke_path);
        hex.theme.token_space_inner.apply_line_and_stroke(ctx, hex);
        ctx.stroke_preserve().unwrap();

        ctx.restore().unwrap();
        ctx.set_matrix(m);
    }
}
