//! Draw a range of token styles.

use cairo::{Context, Format, ImageSurface};
use navig18xx::prelude::*;

mod output;

/// The hex size, used by Token to determine the text size.
static HEX_DIAMETER: f64 = 150.0;

/// The radius of each token.
static TOKEN_RADIUS: f64 = 20.0;

/// The radius of the space reserved for each token, including a margin.
static TOKEN_RADIUS_MARGIN: f64 = TOKEN_RADIUS * 1.125;

fn new_context(width: i32, height: i32) -> (Context, ImageSurface) {
    let surface = ImageSurface::create(Format::ARgb32, width, height)
        .expect("Can't create surface");
    (Context::new(&surface), surface)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_file = output::Dir::Examples.join("draw_tokens.png");

    let rows = 3;
    let cols = 8;

    let width = cols as f64 * 2.0 * TOKEN_RADIUS_MARGIN;
    let height = rows as f64 * 2.0 * TOKEN_RADIUS_MARGIN;
    let (ctx, surf) = new_context(width as i32, height as i32);

    // Background colours for minor (yellow) and major (green) companies.
    let bg_yellow = Colour::from((223, 223, 0));
    let bg_green = Colour::from((0, 153, 63));
    let bg_iter = std::iter::repeat(bg_yellow)
        .take(16)
        .chain(std::iter::repeat(bg_green).take(8));

    // Foreground colours.
    let aqua = Colour::from((0, 204, 204));
    let blue = Colour::from((0, 63, 204));
    let red = Colour::from((223, 0, 0));
    let purple = Colour::from((127, 0, 223));
    let fg_colours = vec![aqua, blue, red, purple];
    let fg_count = fg_colours.len();
    let fg_iter = fg_colours.into_iter().cycle();

    // Define token styles and create tokens.
    let tokens: Vec<Token> = bg_iter
        .zip(fg_iter)
        .enumerate()
        .map(|(ix, (bg, fg))| {
            // Use black text on yellow, and white text on green.
            let text = if bg == bg_yellow {
                Colour::from((0, 0, 0))
            } else {
                Colour::from((255, 255, 255))
            };
            // Cycle through token styles, repeating each style in turn so
            // that it is paired with all of the foreground colours.
            match ix / fg_count {
                0 => TokenStyle::TopLines { bg, fg, text },
                1 => TokenStyle::TopTriangles { bg, fg, text },
                2 => TokenStyle::TopArcs { bg, fg, text },
                3 => TokenStyle::TripleTriangles { bg, fg, text },
                4 => TokenStyle::TopLines { bg, fg, text },
                _ => TokenStyle::TopTriangles { bg, fg, text },
            }
        })
        .map(|style| Token::new(style))
        .collect();

    // Define the token names
    let names = vec![
        "BBG", "BO", "CV", "CS", "KP", "LPS", "OP", "SLA", "TGB", "TN", "AE",
        "CA", "NYO", "PM", "QLS", "THB", "CNR", "CPR", "C&O", "GT", "GW",
        "IRC", "NTR", "NYC",
    ];

    let hex = Hex::new(HEX_DIAMETER);
    let rotn = 0.0;

    let mut tok_ix = 0;
    for row in 0..rows {
        for col in 0..cols {
            // Define the token boundary.
            let x = TOKEN_RADIUS_MARGIN * (1.0 + 2.0 * col as f64);
            let y = TOKEN_RADIUS_MARGIN * (1.0 + 2.0 * row as f64);
            ctx.new_path();
            ctx.arc(x, y, TOKEN_RADIUS, 0.0, 2.0 * PI);

            // Draw the token.
            tokens[tok_ix].draw(&hex, &ctx, names[tok_ix], rotn);

            tok_ix += 1;
        }
    }

    let mut file = std::fs::File::create(output_file)
        .expect("Couldn't create output PNG file");
    surf.write_to_png(&mut file)
        .expect("Couldn't write to output PNG file");

    Ok(())
}
