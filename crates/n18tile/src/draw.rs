use cairo::Context;
use n18hex::Hex;

pub trait Draw {
    // NOTE: will be used to determine track / city connectivity using
    // ctx.in_fill(x, y) for (x,y) along track segment
    // --- how will this be handled with translate/rotate stuff?!?
    fn define_boundary(&self, hex: &Hex, ctx: &Context);
    fn draw_bg(&self, hex: &Hex, ctx: &Context);
    fn draw_fg(&self, hex: &Hex, ctx: &Context);
}
