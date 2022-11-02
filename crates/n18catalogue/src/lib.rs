//! Provides tile catalogues and defines common tiles as per the [18xx Tile
//! Database](http://www.fwtwr.com/18xx/tiles/).
//!
//! # Overview
//!
//! This module defines a [Builder] type that assembles [tiles](Tile) into
//! [Catalogues](Catalogue), and provides [many predefined tiles](Kind).
//!
//! Catalogues define the range of tiles that can be placed on a map, and
//! their [availability](Availability).
//!
//! ```rust
//! # use n18catalogue::{Availability, Builder, Catalogue, Kind};
//! // Construct a small catalogue of standard tiles, identified by name.
//! let tiles = vec![
//!     (Kind::_3, Availability::Limited(4)),
//!     (Kind::_208, Availability::Unlimited),
//! ];
//! let catalogue = Builder::with_tiles(tiles).unwrap().build();
//! ```
//!
use std::collections::BTreeMap;
use std::iter::FromIterator;

use n18hex::*;
use n18tile::*;

mod tiles;

pub use tiles::Kind;

#[cfg(test)]
mod tests;

/// Defines how many copies of a specific tile are available.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Availability {
    /// There is a fixed number of copies.
    Limited(usize),
    /// There are an unlimited number of copies.
    Unlimited,
    /// The tile is not available to players.
    ///
    /// This is typically used for game-specific tiles that define the
    /// starting map state and off-board locations.
    Unavailable,
}

impl From<Option<usize>> for Availability {
    fn from(src: Option<usize>) -> Self {
        if let Some(count) = src {
            Availability::Limited(count)
        } else {
            Availability::Unlimited
        }
    }
}

/// A builder for constructing tile catalogues.
///
/// This uses the default [Hex] to construct tiles from the built-in tile
/// library, as identified by [Kind].
/// Custom tiles can also be added (e.g., game-specific tiles used to define
/// the starting map state and off-board locations).
///
/// Construct the final catalogue with [Builder::build()].
///
/// # Example usage
///
/// ```rust
/// # use n18catalogue::{Availability, Builder, Catalogue, Kind};
/// let tiles = vec![
///     (Kind::_3, Availability::Limited(4)),
///     (Kind::_208, Availability::Unlimited),
/// ];
/// let builder = Builder::with_tiles(tiles).unwrap();
/// let catalogue = builder.build();
/// assert_eq!(catalogue.len(), 2);
///
/// // Catalogue can be indexed to retrieve (Tile, Availability) tuples.
/// assert_eq!(catalogue[0].0.name, "3");
/// assert_eq!(catalogue[0].1, Availability::Limited(4));
/// assert_eq!(catalogue[1].0.name, "208");
/// assert_eq!(catalogue[1].1, Availability::Unlimited);
/// ```
///
/// # Collecting pre-existing tiles
///
/// [Builder] implements the following traits:
///
/// - `FromIterator<(Tile, Availability)>`;
/// - `From<Vec<(Tile, Availability)>>`;
/// - `FromIterator<Tile>` (all tiles are assumed to have
///   [unlimited availability](Availability::Unlimited)); and
/// - `From<Vec<Tile>>` (all tiles are assumed to have
///   [unlimited availability](Availability::Unlimited)).
///
/// # Further details
///
/// This builder is provided, in part, because it is convenient to use a
/// single [Hex] value to create tiles, but the [Catalogue] type **cannot**
/// own a [Hex].
///
/// This is because we need [Catalogue] to be Send and Sync, so that maps can
/// be shared between threads when searching for optimal routes.
/// [Hex] isn't Sync, because it contains a Cairo surface and context, neither
/// of which are Sync.
#[derive(Debug)]
pub struct Builder {
    hex: Hex,
    tiles: Vec<(Tile, Availability)>,
    tile_tree: BTreeMap<String, usize>,
}

impl From<Vec<Tile>> for Builder {
    fn from(src: Vec<Tile>) -> Self {
        src.into_iter().collect()
    }
}

impl From<Vec<(Tile, Availability)>> for Builder {
    fn from(src: Vec<(Tile, Availability)>) -> Self {
        src.into_iter().collect()
    }
}

impl FromIterator<Tile> for Builder {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Tile>,
    {
        let mut cat = Builder::empty();
        cat.add_unlimited_tiles(iter);
        cat
    }
}

impl FromIterator<(Tile, Availability)> for Builder {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Tile, Availability)>,
    {
        let mut cat = Builder::empty();
        cat.add_tiles(iter);
        cat
    }
}

impl Builder {
    /// Converts the collected tiles into a [Catalogue].
    pub fn build(self) -> Catalogue {
        Catalogue {
            tiles: self.tiles,
            tile_tree: self.tile_tree,
        }
    }

    /// Returns a [Builder] that contains no tiles.
    pub fn empty() -> Self {
        Builder {
            hex: Hex::default(),
            tiles: vec![],
            tile_tree: BTreeMap::new(),
        }
    }

    /// Returns a [Builder] that contains all of the built-in tiles.
    pub fn all_tiles() -> Self {
        let mut cat = Builder::empty();
        for kind in Kind::iter() {
            cat.add_unlimited_tile(kind.build(cat.hex()));
        }
        cat
    }

    /// Returns a [Builder] that contains all of the specified built-in tiles
    /// (identified by [Kind]), where each tile has a fixed or unlimited
    /// number of copies.
    ///
    /// If any duplicate tiles are encountered, this returns the duplicates.
    ///
    /// # Example usage
    ///
    /// If there is a limited number of each tile, the availability can be
    /// provided as `usize` values:
    ///
    /// ```rust
    /// # use n18catalogue::{Builder, Kind};
    /// // 2 copies of tiles "3" and "5", 4 copies of tile "4".
    /// let tiles = vec![(Kind::_3, 2), (Kind::_4, 4), (Kind::_5, 2)];
    /// let cat = Builder::with_available_tiles(tiles).unwrap().build();
    /// ```
    ///
    /// If there is an unlimited number of any tile, the availability must be
    /// provided as `Option<usize>` values:
    ///
    /// ```rust
    /// # use n18catalogue::{Builder, Kind};
    /// // 2 copies of tile "3", unlimited copies of tile "4".
    /// let tiles = vec![(Kind::_3, Some(2)), (Kind::_4, None)];
    /// let cat = Builder::with_available_tiles(tiles).unwrap().build();
    /// ```
    pub fn with_available_tiles<T, A>(tiles: T) -> Result<Self, Vec<Kind>>
    where
        T: IntoIterator<Item = (Kind, A)>,
        A: Into<Option<usize>>,
    {
        let iter = tiles
            .into_iter()
            .map(|(kind, avail)| (kind, avail.into().into()));
        Self::with_tiles(iter)
    }

    /// Returns a [Builder] that contains all of the specified built-in tiles
    /// (identified by [Kind]).
    ///
    /// If any duplicate tiles are encountered, this returns the duplicates.
    pub fn with_tiles<T>(tiles: T) -> Result<Self, Vec<Kind>>
    where
        T: IntoIterator<Item = (Kind, Availability)>,
    {
        let mut cat = Builder::empty();
        let mut duplicates = vec![];
        let mut seen = std::collections::BTreeSet::new();

        for (kind, count) in tiles.into_iter() {
            if seen.contains(&kind) {
                duplicates.push(kind);
                continue;
            }
            cat.add_tile(kind.build(cat.hex()), count);
            seen.insert(kind);
        }

        if duplicates.is_empty() {
            Ok(cat)
        } else {
            Err(duplicates)
        }
    }

    /// Returns a [Builder] that contains all of the specified built-in tiles
    /// (identified by name and having unlimited availability).
    ///
    /// If any duplicate tiles are encountered, this returns the duplicates.
    pub fn with_unlimited_tiles<T>(names: T) -> Result<Self, Vec<Kind>>
    where
        T: IntoIterator<Item = Kind>,
    {
        Builder::with_tiles(
            names.into_iter().map(|k| (k, Availability::Unlimited)),
        )
    }

    /// Returns a reference to the [Hex] used to construct tiles.
    pub fn hex(&self) -> &Hex {
        &self.hex
    }

    /// Adds a tile to the collection, and returns the index of the previous
    /// tile with this name (if any).
    pub fn add_tile(
        &mut self,
        tile: Tile,
        limit: Availability,
    ) -> Option<usize> {
        if let Some(&ix) = self.tile_tree.get(&tile.name) {
            self.tiles[ix] = (tile, limit);
            Some(ix)
        } else {
            let ix = self.tiles.len();
            let name = tile.name.clone();
            self.tiles.push((tile, limit));
            self.tile_tree.insert(name, ix)
        }
    }

    /// Adds a tile with unlimited availability to the collection, and returns
    /// the index of the previous tile with this name (if any).
    pub fn add_unlimited_tile(&mut self, tile: Tile) -> Option<usize> {
        self.add_tile(tile, Availability::Unlimited)
    }

    /// Adds a tile with limited availability to the collection, and returns
    /// the index of the previous tile with this name (if any).
    pub fn add_limited_tile(
        &mut self,
        tile: Tile,
        count: usize,
    ) -> Option<usize> {
        self.add_tile(tile, Availability::Limited(count))
    }

    /// Adds an unavailable tile to the collection, and returns the index of
    /// the previous tile with this name (if any).
    pub fn add_unavailable_tile(&mut self, tile: Tile) -> Option<usize> {
        self.add_tile(tile, Availability::Unavailable)
    }

    /// Adds tiles with unlimited availability to the collection, and returns
    /// the indices of any previous tiles with the same name.
    pub fn add_unlimited_tiles<T>(&mut self, tiles: T) -> Option<Vec<usize>>
    where
        T: IntoIterator<Item = Tile>,
    {
        let existing: Vec<usize> = tiles
            .into_iter()
            .filter_map(|tile| self.add_unlimited_tile(tile))
            .collect();
        if existing.is_empty() {
            None
        } else {
            Some(existing)
        }
    }

    /// Adds tiles with limited availability to the collection, and returns
    /// the indices of any previous tiles with the same name.
    pub fn add_tiles<T>(&mut self, tiles: T) -> Option<Vec<usize>>
    where
        T: IntoIterator<Item = (Tile, Availability)>,
    {
        let existing: Vec<usize> = tiles
            .into_iter()
            .filter_map(|(tile, count)| self.add_tile(tile, count))
            .collect();
        if existing.is_empty() {
            None
        } else {
            Some(existing)
        }
    }

    /// Adds unavailable tiles to the collection, and returns the indices of
    /// any previous tiles with the same name.
    pub fn add_unavailable_tiles<T>(&mut self, tiles: T) -> Option<Vec<usize>>
    where
        T: IntoIterator<Item = Tile>,
    {
        let existing: Vec<usize> = tiles
            .into_iter()
            .filter_map(|tile| self.add_tile(tile, Availability::Unavailable))
            .collect();
        if existing.is_empty() {
            None
        } else {
            Some(existing)
        }
    }
}

/// A tile catalogue is a collection of tiles.
#[derive(Clone, Debug, PartialEq)]
pub struct Catalogue {
    tiles: Vec<(Tile, Availability)>,
    tile_tree: BTreeMap<String, usize>,
}

impl From<Vec<Tile>> for Catalogue {
    fn from(src: Vec<Tile>) -> Self {
        src.into_iter().collect()
    }
}

impl From<Vec<(Tile, Availability)>> for Catalogue {
    fn from(src: Vec<(Tile, Availability)>) -> Self {
        src.into_iter().collect()
    }
}

/// Convert a catalogue into a vector of tiles and their availability.
impl From<Catalogue> for Vec<(Tile, Availability)> {
    fn from(src: Catalogue) -> Self {
        src.tiles
    }
}

impl FromIterator<Tile> for Catalogue {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Tile>,
    {
        iter.into_iter().collect::<Builder>().build()
    }
}

impl FromIterator<(Tile, Availability)> for Catalogue {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Tile, Availability)>,
    {
        iter.into_iter().collect::<Builder>().build()
    }
}

impl std::ops::Index<usize> for Catalogue {
    type Output = (Tile, Availability);

    fn index(&self, index: usize) -> &Self::Output {
        &self.tiles[index]
    }
}

impl Catalogue {
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn nth(&self, ix: usize) -> Option<&(Tile, Availability)> {
        self.tiles.get(ix)
    }

    /// Returns an iterator over tiles and their availability, in indexed
    /// order.
    pub fn iter(&self) -> impl Iterator<Item = &(Tile, Availability)> {
        self.tiles.iter()
    }

    /// Returns an iterator over tiles, in indexed order.
    pub fn tile_iter(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter().map(|(tile, _avail)| tile)
    }

    /// Returns an iterator over tile names, in indexed order.
    pub fn tile_names(&self) -> impl Iterator<Item = &String> {
        self.tiles.iter().map(|(tile, _avail)| &tile.name)
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.tile_tree.get(name).copied()
    }

    pub fn tile(&self, name: &str) -> Option<&Tile> {
        self.tile_tree.get(name).map(|ix| &(self.tiles[*ix].0))
    }

    pub fn availability(&self, name: &str) -> Option<&Availability> {
        self.tile_tree.get(name).map(|ix| &(self.tiles[*ix].1))
    }

    pub fn tile_and_availability(
        &self,
        name: &str,
    ) -> Option<&(Tile, Availability)> {
        self.tile_tree.get(name).map(|ix| &self.tiles[*ix])
    }

    pub fn get_subset<S>(&self, names: &[S]) -> Result<Vec<&Tile>, String>
    where
        S: AsRef<str>,
    {
        names
            .iter()
            .map(|name| {
                self.tile(name.as_ref())
                    .ok_or_else(|| name.as_ref().to_string())
            })
            .collect()
    }
}

/// Returns all of the built-in tiles provided by [Kind], named as per the
/// [18xx Tile Database](http://www.fwtwr.com/18xx/tiles/).
pub fn tile_catalogue() -> Vec<Tile> {
    let hex = Hex::default();
    Kind::iter().map(|kind| kind.build(&hex)).collect()
}
