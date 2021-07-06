use n18hex::Hex;
use n18map::Map;
use n18route::{Bonus, Train};
use n18tile::Tile;
use n18token::{Token, Tokens};

pub mod _1867;

/// The details that characterise a company that can operate trains.
pub struct Company {
    /// The abbreviated company name, which must be unique for each company in
    /// the game, and which will appear on the company tokens.
    pub abbrev: String,
    /// The full company name, used for display purposes.
    pub full_name: String,
    /// The visual characteristics of the company tokens.
    pub token: Token,
}

/// The methods that are required for a specific 18xx game implementation.
///
/// Note that we do not use associated types to identify the companies,
/// trains, and game phases for each game, because each of these would need to
/// be provided as type parameters for other data structures, such as [Map],
/// in order for these types to known and usable.
/// We instead use string slices (`&str`) to identify these game-specific
/// entities, and provide methods to retrieve these entities with and without
/// panicking (`get_foo()` and `try_foo()`, respectively).
///
/// Also note that trains and game phases, as returned by [Game::trains],
/// [Game::train_names], [Game::train_types], and [Game::phase_names], are
/// returned as slices `&[T]` so that they retain their game-specific order.
///
/// While game phases are identified by name (`&str`), [Game::set_phase_ix]
/// instead uses an index (`usize`), because this avoids issues with having
/// concurrent mutable and immutable borrows of [Game] values in order to
/// retrieve the phase name (via, e.g., [Game::phase_names]) and to change the
/// game phase with [Game::set_phase_ix].
/// Alternatively, the caller can clone the phase names and use
/// [Game::set_phase_name].
pub trait Game {
    /// Returns the name of this game.
    fn name(&self) -> &str;

    /// Creates the initial map for this game.
    fn create_map(&self, hex: &Hex) -> Map;

    /// Returns the companies in this game.
    fn companies(&self) -> &[Company];

    /// Creates a [Tokens] collection, which is needed for creating the game [Map].
    fn create_tokens(&self) -> Tokens {
        self.companies()
            .iter()
            .map(|c| (c.abbrev.clone(), c.token))
            .collect::<Vec<_>>()
            .into()
    }

    /// Returns the token associated with the first company.
    fn first_token(&self) -> &Token {
        &self.companies()[0].token
    }

    /// Returns the token associated with the last company.
    fn last_token(&self) -> &Token {
        let companies = self.companies();
        let last_ix = companies.len() - 1;
        &companies[last_ix].token
    }

    /// Cycles forward through tokens in the company order, stopping at the
    /// end.
    fn next_token(&self, token: &Token) -> Option<&Token> {
        let companies = self.companies();
        let ix_opt = companies
            .iter()
            .enumerate()
            .find(|(_ix, tok)| &tok.token == token)
            .map(|(ix, _tok)| ix);
        ix_opt.and_then(|ix| {
            let next_ix = ix + 1;
            if next_ix < companies.len() {
                Some(&companies[next_ix].token)
            } else {
                None
            }
        })
    }

    /// Cycles backward through tokens in the company order, stopping at the
    /// beginning.
    fn prev_token(&self, token: &Token) -> Option<&Token> {
        let companies = self.companies();
        let ix_opt = companies
            .iter()
            .enumerate()
            .find(|(_ix, tok)| &tok.token == token)
            .map(|(ix, _tok)| ix);
        ix_opt.and_then(|ix| {
            let next_ix = ix + 1;
            if next_ix < companies.len() {
                Some(&companies[next_ix].token)
            } else {
                None
            }
        })
    }

    /// Returns the abbreviated name of each company in this game.
    fn company_abbrevs(&self) -> Vec<&str> {
        self.companies()
            .iter()
            .map(|c| &c.abbrev)
            .map(|a| a.as_str())
            .collect()
    }

    /// Returns the company with the given abbreviated name, if it exists.
    fn try_company(&self, abbrev: &str) -> Option<&Company> {
        self.companies().iter().find(|c| c.abbrev == abbrev)
    }

    /// Returns the company with the given abbreviated name.
    ///
    /// # Panics
    ///
    /// Panics if there is no company with the given name.
    fn get_company(&self, abbrev: &str) -> &Company {
        self.try_company(abbrev)
            .expect(&format!("No company named '{}'", abbrev))
    }

    /// Returns the token with the given abbreviated name, if it exists.
    fn try_token(&self, abbrev: &str) -> Option<&Token> {
        self.try_company(abbrev).map(|c| &c.token)
    }

    /// Returns the token with the given abbreviated name.
    ///
    /// # Panics
    ///
    /// Panics if there is no token with the given name.
    fn get_token(&self, abbrev: &str) -> &Token {
        self.try_token(abbrev)
            .expect(&format!("No company named '{}'", abbrev))
    }

    /// Returns the named train types in this game, in the order that they
    /// become available (where applicable).
    fn trains(&self) -> &[(&str, Train)];

    /// Returns the name of each train type, in the order that they become
    /// available (where applicable).
    fn train_names(&self) -> Vec<&str> {
        self.trains().iter().map(|t| t.0).collect()
    }

    /// Returns each train type, in the order that they become available
    /// (where applicable).
    fn train_types(&self) -> Vec<&Train> {
        self.trains().iter().map(|t| &t.1).collect()
    }

    /// Returns the train with the given name.
    ///
    /// # Panics
    ///
    /// Panics if there is no train with the given name.
    fn get_train(&self, name: &str) -> &Train {
        self.try_train(name)
            .expect(&format!("No train named '{}'", name))
    }

    /// Returns the train with the given name, if the train exists.
    fn try_train(&self, name: &str) -> Option<&Train> {
        self.trains().iter().find(|t| t.0 == name).map(|t| &t.1)
    }

    /// Returns the name associated with the given train, if the train exists
    /// in this game.
    fn train_name(&self, train: &Train) -> Option<&str> {
        self.trains().iter().find(|t| &t.1 == train).map(|t| t.0)
    }

    /// Optional route bonuses that a company may hold.
    fn bonus_options(&self) -> Vec<&str>;

    /// Return the bonuses that may apply to the routes being operated by a
    /// company, given the bonus options (e.g., private company bonuses) that
    /// the company currently owns.
    fn get_bonuses(&self, bonus_options: &Vec<bool>) -> Vec<Bonus>;

    /// Returns all game tiles, including special tiles that players cannot
    /// place on the map.
    fn all_tiles(&self) -> &[Tile];

    /// Returns the tiles that players are allowed to place on the map.
    fn player_tiles(&self) -> &[Tile];

    /// Returns the name of each game phase, in the order that they occur.
    fn phase_names(&self) -> &[&str];

    /// Returns the number of game phases.
    fn phase_count(&self) -> usize {
        self.phase_names().len()
    }

    /// Returns the name of the current game phase.
    fn current_phase_name(&self) -> &str {
        self.phase_names()[self.get_phase_ix()]
    }

    /// Returns the index of the current game phase.
    fn get_phase_ix(&self) -> usize;

    /// Changes the current game phase, which may update the map.
    ///
    /// Note that this uses the phase index (`usize`) instead of the phase
    /// name, because using names can result in situations where we need both
    /// an immutable borrow (to get the phase names with [Game::phase_names])
    /// and a mutable borrow of the [Game] to change the game phase.
    fn set_phase_ix(&mut self, map: &mut Map, phase: usize) -> bool;

    /// Changes the current game phase, which may update the map.
    ///
    /// Note that that caller will almost certainly need to own the phase name
    /// string in order to call this method, rather than using the `&str`
    /// values returned by [Game::phase_names].
    fn set_phase_name(&mut self, map: &mut Map, phase: &str) -> bool {
        let ix_opt =
            self.phase_names()
                .iter()
                .enumerate()
                .find_map(
                    |(ix, name)| if *name == phase { Some(ix) } else { None },
                );
        ix_opt.map(|ix| self.set_phase_ix(map, ix)).unwrap_or(false)
    }

    /// Advance to the next game phase, if it exists.
    fn next_phase(&mut self, map: &mut Map) -> bool {
        let curr = self.get_phase_ix();
        if curr < usize::MAX {
            self.set_phase_ix(map, curr + 1)
        } else {
            false
        }
    }

    /// Revert to the previous game phase, if it exists.
    fn prev_phase(&mut self, map: &mut Map) -> bool {
        let curr = self.get_phase_ix();
        if curr > 0 {
            self.set_phase_ix(map, curr - 1)
        } else {
            false
        }
    }
}
