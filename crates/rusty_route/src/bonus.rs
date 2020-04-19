//! Route bonuses that can increase revenue.

use rusty_map::HexAddress;

/// The different types of route bonus that may be applied.
pub enum Bonus {
    /// A bonus for visiting a specific location.
    VisitBonus { locn: HexAddress, bonus: usize },
    /// A bonus for connecting one location to another location.
    ConnectionBonus {
        from: HexAddress,
        to_any: Vec<HexAddress>,
        bonus: usize,
    },
}
