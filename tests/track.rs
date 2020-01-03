use cairo::{Context, Format, ImageSurface};
use rusty_train::prelude::*;

static HEX_DIAMETER: f64 = 150.0;

fn new_context(width: i32, height: i32) -> (Context, ImageSurface) {
    let surface = ImageSurface::create(Format::ARgb32, width, height)
        .expect("Can't create surface");
    (Context::new(&surface), surface)
}

// TODO: test track catalogue, every city should intersect with at least one
// track segment!

#[test]
fn two_straights_cross() {
    let hex = Hex::new(HEX_DIAMETER);
    let ctx = hex.context();
    let dt = 0.1;
    let t1 = Track::straight(HexFace::Top);
    let t2 = Track::straight(HexFace::UpperLeft);
    assert!(t1.crosses(&t2, &hex, dt, &ctx));
    assert!(t2.crosses(&t1, &hex, dt, &ctx));
    assert!(!t1.connected(&t2, &hex, &ctx));
    assert!(!t2.connected(&t1, &hex, &ctx));
}

fn no_escape(track: &Track, hex: &Hex, dt: f64, ctx: &Context) -> bool {
    hex.define_boundary(ctx);
    track
        .coords(hex, dt)
        .all(|coord| ctx.in_fill(coord.x, coord.y))
}

#[test]
fn track_contained_in_hex() {
    let dim = HEX_DIAMETER * 1.1;
    let (ctx, surf) = new_context(dim as i32, dim as i32);
    let hex = Hex::new(HEX_DIAMETER);
    let dt = 0.1;

    use HexFace::*;
    use TrackCurve::*;

    hex.define_boundary(&ctx);
    let (hex_x0, hex_y0, hex_x1, hex_y1) = ctx.path_extents();

    // TODO: if we switch the clockwise flag, the track segments are still
    // contained within the hex?!?
    // NO! But the coords are!!
    // TODO: HOW CAN WE DETECT IF THE PATH ESCAPES THE HEXAGON?!?
    // It's not robust, but can compare ctx.path_extents() for hex and track?
    // Should we try saving each image as a separate PNG file?

    let mut counter = 0;

    for face in vec![Top]
    //, UpperRight, LowerRight, Bottom, LowerLeft, UpperLeft]
    {
        for curve in vec![Straight, GentleL, HardL, GentleR, HardR] {
            for x0 in vec![0.0, 0.25] {
                //, 0.5, 0.75] {
                for x1 in vec![
                    1.0,
                    1.0 - 0.25 * (1.0 - x0),
                    // 1.0 - 0.5 * (1.0 - x0),
                    // 1.0 - 0.75 * (1.0 - x0),
                ] {
                    let t = Track::new(face, curve, x0, x1, None, None);
                    assert!(no_escape(&t, &hex, dt, &ctx));

                    // TODO: check that track intersects hex boundary if
                    // x0 is 0.0 and that it doesn't if x0 is not 0.0.
                    let start = t.start(&hex);
                    if x0 == 0.0 {
                        assert!(ctx.in_stroke(start.x, start.y));
                        // NOTE: this check causes the invalid clockwise
                        // setting to fail!
                        let diff =
                            (&start - &hex.midpoint(&face)).magnitude();
                        assert!(diff < 1e-8);
                    } else {
                        assert!(!ctx.in_stroke(start.x, start.y));
                    }
                    let end = t.end(&hex);
                    if x1 == 1.0 {
                        assert!(ctx.in_stroke(end.x, end.y));
                    // TODO: check that it intersects the correct face!
                    } else {
                        assert!(!ctx.in_stroke(end.x, end.y));
                    }

                    ctx.save();
                    t.define_boundary(&hex, &ctx);
                    let (t_x0, t_y0, t_x1, t_y1) = ctx.path_extents();
                    assert!(t_x0 >= hex_x0);
                    assert!(t_y0 >= hex_y0);
                    assert!(t_x1 <= hex_x1);
                    assert!(t_y1 <= hex_y1);
                    ctx.restore();

                    ctx.save();
                    ctx.translate(dim / 2.0, dim / 2.0);

                    // Clear the surface.
                    // https://www.cairographics.org/FAQ/#clear_a_surface
                    let operator = ctx.get_operator();
                    ctx.set_operator(cairo::Operator::Clear);
                    ctx.paint();
                    ctx.set_operator(operator);

                    hex.draw_background(HexColour::Green, &ctx);
                    t.draw_bg(&hex, &ctx);
                    t.draw_fg(&hex, &ctx);
                    ctx.set_source_rgb(1.0, 0.0, 0.0);
                    let line_cap = ctx.get_line_cap();
                    ctx.set_line_cap(cairo::LineCap::Round);
                    for coord in t.coords(&hex, 0.1) {
                        ctx.new_path();
                        ctx.move_to(coord.x, coord.y);
                        ctx.line_to(coord.x, coord.y);
                        ctx.stroke();
                    }
                    ctx.set_line_cap(line_cap);

                    let name = format!("({:0.2}, {:0.2})", x0, x1);
                    hex.draw_tile_name(&name, &ctx);

                    let filename = format!("test-tcih-{:04}.png", counter);
                    let mut file = std::fs::File::create(filename)
                        .expect("Couldn't create output PNG file");
                    surf.write_to_png(&mut file)
                        .expect("Couldn't write to output PNG file");
                    ctx.translate(-dim / 2.0, -dim / 2.0);
                    ctx.restore();
                    counter += 1;
                }
            }
        }
    }
}

#[test]
fn invalid_span_escapes_hex() {
    let hex = Hex::new(HEX_DIAMETER);
    let ctx = hex.context();
    let dt = 0.1;

    let t = Track::gentle_r(HexFace::LowerLeft).with_span(-0.5, 0.5);
    assert!(!no_escape(&t, &hex, dt, &ctx));
}

#[test]
fn coords_contained_in_track() {
    let hex = Hex::new(HEX_DIAMETER);
    let ctx = hex.context();
    let dt = 0.1;

    use HexFace::*;
    use TrackCurve::*;

    for face in
        vec![Top, UpperRight, LowerRight, Bottom, LowerLeft, UpperLeft]
    {
        for curve in vec![Straight, GentleL, HardL, GentleR, HardR] {
            for x0 in vec![0.0, 0.25, 0.5, 0.75] {
                for x1 in vec![
                    1.0,
                    1.0 - 0.25 * x0,
                    1.0 - 0.5 * x0,
                    1.0 - 0.75 * x0,
                ] {
                    let t = Track::new(face, curve, x0, x1, None, None);
                    t.define_boundary(&hex, &ctx);
                    assert!(t
                        .coords(&hex, dt)
                        .all(|coord| ctx.in_stroke(coord.x, coord.y)))
                }
            }
        }
    }
}
