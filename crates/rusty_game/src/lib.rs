use rusty_hex::Hex;
use rusty_map::Map;
use rusty_route::{Bonus, Train};
use rusty_tile::Tile;
use rusty_token::Tokens;

pub mod _1867;

/// The methods that are required for a specific 18xx game implementation.
pub trait Game {
    /// The name of this game.
    fn name(&self) -> &str;

    fn create_map(&self, hex: &Hex) -> Map;

    // NOTE: this returns a Vec and not a HashMap<Train, &str> so that there
    // is a fixed ordering.
    fn train_types(&self) -> Vec<Train>;

    fn train_name(&self, train: &Train) -> Option<&str>;

    /// Optional route bonuses that a company may hold.
    fn bonus_options(&self) -> Vec<&str>;

    /// Return the bonuses that may apply to the routes being operated by a
    /// company, given the bonus options (e.g., private company bonuses) that
    /// the company currently owns.
    fn get_bonuses(&self, bonus_options: &Vec<bool>) -> Vec<Bonus>;

    /// Return the tiles that players are allowed to place on the map.
    fn player_tiles(&self) -> &[Tile];

    /// Return the unique tokens (one per company).
    fn company_tokens(&self) -> &Tokens;

    /// Return the number of game phases.
    fn phase_count(&self) -> usize;

    /// Return the current game phase.
    fn get_phase(&self) -> usize;

    /// Change the current game phase, which may update the map.
    fn set_phase(&mut self, map: &mut Map, phase: usize);

    /// Return the name of a game phase.
    fn phase_name(&self, phase: usize) -> Option<&str>;

    /// Return the name of each game phase.
    fn phase_names(&self) -> &[&str];
}
