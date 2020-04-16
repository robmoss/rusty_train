use crate::prelude::{Hex, Map, Train};

pub mod _1867;

/// The methods that are required for a specific 18xx game implementation.
pub trait Game {
    fn create_map(&self, hex: &Hex) -> Map;

    // NOTE: this returns a Vec and not a HashMap<Train, &str> so that there
    // is a fixed ordering.
    fn train_types(&self) -> Vec<Train>;

    fn train_name(&self, train: &Train) -> Option<&str>;

    fn bonus_options(&self) -> Vec<&str>;
}
