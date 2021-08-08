//! Provides tile catalogues and defines common tiles as per the [18xx Tile
//! Database](http://www.fwtwr.com/18xx/tiles/).
//!
//! # Overview
//!
//! This module defines a [Builder] type that collects tiles and assembles
//! them into a [Catalogue] type.
//!
//! Catalogues define the range of tiles that can be placed on a map, and
//! their [availability](Availability).
//!
//! ```rust
//! # use n18catalogue::{Availability, Builder, Catalogue};
//! // Construct a small catalogue of standard tiles, identified by name.
//! let tiles = vec![
//!     ("3", Availability::Limited(4)),
//!     ("208", Availability::Unlimited),
//! ];
//! let catalogue = Builder::subset(tiles).unwrap().build();
//! ```
//!
use std::collections::BTreeMap;
use std::iter::FromIterator;

use n18hex::*;
use n18tile::*;

mod tiles;

#[cfg(test)]
mod tests;

/// Tiles are created by functions that accept a reference hexagon and a tile
/// name.
///
/// The tile name is provided as an argument in order to avoid duplication
/// (the tile name is also the key under which it is stored in the catalogue).
type TileFn = dyn Fn(&Hex, &str) -> Tile;

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
/// library.
/// Custom tiles can also be added (e.g., game-specific tiles used to define
/// the starting map state and off-board locations).
///
/// Construct the final catalogue with [Builder::build()].
///
/// # Example usage
///
/// ```rust
/// # use n18catalogue::{Availability, Builder, Catalogue};
/// let tiles = vec![
///     ("3", Availability::Limited(4)),
///     ("208", Availability::Unlimited),
/// ];
/// let builder = Builder::subset(tiles).unwrap();
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
        for (name, tile_fn) in &tiles::all_tile_fns() {
            cat.add_unlimited_tile(tile_fn(cat.hex(), name));
        }
        cat
    }

    /// Returns a [Builder] that contains all of the specified built-in tiles
    /// (identified by name), where each tile has a fixed or unlimited number
    /// of copies.
    ///
    /// If any unknown or duplicate tile names are encountered, this returns
    /// the unknown and duplicate names, respectively.
    ///
    /// # Example usage
    ///
    /// If there is a limited number of each tile, the availability can be
    /// provided as `usize` values:
    ///
    /// ```rust
    /// # use n18catalogue::Builder;
    /// // 2 copies of tiles "3" and "5", 4 copies of tile "4".
    /// let tiles = vec![("3", 2), ("4", 4), ("5", 2)];
    /// let cat = Builder::subset_available(tiles).unwrap().build();
    /// ```
    ///
    /// If there is an unlimited number of any tile, the availability must be
    /// provided as `Option<usize>` values:
    ///
    /// ```rust
    /// # use n18catalogue::Builder;
    /// // 2 copies of tiles "3" and "5", unlimited copies of tile "4".
    /// let tiles = vec![("3", Some(2)), ("4", None), ("5", Some(2))];
    /// let cat = Builder::subset_available(tiles).unwrap().build();
    /// ```
    pub fn subset_available<S, T, A>(
        tiles: T,
    ) -> Result<Self, (Vec<String>, Vec<String>)>
    where
        S: AsRef<str>,
        T: IntoIterator<Item = (S, A)>,
        A: Into<Option<usize>>,
    {
        let iter = tiles
            .into_iter()
            .map(|(name, avail)| (name, avail.into().into()));
        Self::subset(iter)
    }

    /// Returns a [Builder] that contains all of the specified built-in tiles
    /// (identified by name).
    ///
    /// If any unknown or duplicate tile names are encountered, this returns
    /// the unknown and duplicate names, respectively.
    pub fn subset<S, T>(tiles: T) -> Result<Self, (Vec<String>, Vec<String>)>
    where
        S: AsRef<str>,
        T: IntoIterator<Item = (S, Availability)>,
    {
        let mut cat = Builder::empty();
        let mut known_tiles: BTreeMap<_, _> = tiles::all_tile_fns()
            .into_iter()
            .map(|(name, tile_fn)| {
                (String::from(name), tile_fn(cat.hex(), name))
            })
            .collect();

        let mut unknown = vec![];
        let mut duplicates = vec![];
        let mut seen = std::collections::BTreeSet::new();

        for (name, count) in tiles.into_iter() {
            let name = name.as_ref();
            if seen.contains(name) {
                duplicates.push(name.to_string());
                continue;
            }
            if let Some(tile) = known_tiles.remove(name) {
                cat.add_tile(tile, count);
                seen.insert(name.to_string());
            } else {
                unknown.push(name.to_string())
            }
        }

        if unknown.is_empty() && duplicates.is_empty() {
            Ok(cat)
        } else {
            Err((unknown, duplicates))
        }
    }

    /// Returns a [Builder] that contains all of the specified built-in tiles
    /// (identified by name and having unlimited availability).
    ///
    /// If any unknown or duplicate tile names are encountered, this returns
    /// the unknown and duplicate names, respectively.
    pub fn subset_unlimited<S, I>(
        names: I,
    ) -> Result<Self, (Vec<String>, Vec<String>)>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Builder::subset(
            names.into_iter().map(|s| (s, Availability::Unlimited)),
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
            .map(|tile| self.add_unlimited_tile(tile))
            .flatten()
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
            .map(|(tile, count)| self.add_tile(tile, count))
            .flatten()
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
            .map(|tile| self.add_tile(tile, Availability::Unavailable))
            .flatten()
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

/// Tiles as per the [18xx Tile Database](http://www.fwtwr.com/18xx/tiles/).
pub fn tile_catalogue() -> Vec<Tile> {
    use crate::track::DitShape::*;
    use HexColour::*;
    use HexCorner::*;
    use HexFace::*;
    use HexPosition::*;
    use TrackEnd::*;

    let h: Hex = Hex::default();
    let hex = &h;

    vec![
        Tile::new(
            Yellow,
            "3",
            vec![
                Track::hard_l(Bottom)
                    .with_span(0.0, 0.5)
                    .with_dit(End, 10, Bar),
                Track::hard_l(Bottom).with_span(0.5, 1.0),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), Centre(None)),
        Tile::new(
            Yellow,
            "4",
            vec![
                Track::straight(Bottom)
                    .with_span(0.0, 0.25)
                    .with_dit(End, 10, Bar),
                Track::straight(Bottom).with_span(0.25, 1.0),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), LowerLeft.to_centre(0.3)),
        Tile::new(
            Yellow,
            "5",
            vec![Track::mid(Bottom), Track::mid(LowerRight)],
            vec![City::single(20)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.3)),
        Tile::new(
            Yellow,
            "6",
            vec![Track::mid(Bottom), Track::mid(UpperRight)],
            vec![City::single(20)],
            hex,
        )
        .label(Label::Revenue(0), Top.to_centre(0.2)),
        Tile::new(Yellow, "7", vec![Track::hard_r(Bottom)], vec![], hex),
        Tile::new(Yellow, "8", vec![Track::gentle_r(Bottom)], vec![], hex),
        Tile::new(Yellow, "9", vec![Track::straight(Bottom)], vec![], hex),
        Tile::new(
            Green,
            "14",
            vec![
                Track::mid(Bottom),
                Track::mid(Top),
                Track::mid(LowerLeft),
                Track::mid(UpperRight),
            ],
            vec![City::double(30)],
            hex,
        )
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Green,
            "15",
            vec![
                Track::mid(Bottom),
                Track::mid(Top),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
            ],
            vec![City::double(30)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Green,
            "16",
            vec![Track::gentle_r(Bottom), Track::gentle_r(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "17",
            vec![Track::gentle_r(Bottom), Track::gentle_l(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "18",
            vec![Track::straight(Bottom), Track::hard_l(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "19",
            vec![Track::gentle_r(LowerLeft), Track::straight(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "20",
            vec![Track::straight(LowerLeft), Track::straight(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "21",
            vec![Track::hard_l(Top), Track::gentle_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "22",
            vec![Track::hard_r(Top), Track::gentle_r(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "23",
            vec![Track::straight(Bottom), Track::gentle_r(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "24",
            vec![Track::straight(Bottom), Track::gentle_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "25",
            vec![Track::gentle_r(Bottom), Track::gentle_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "26",
            vec![Track::straight(Bottom), Track::hard_r(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "27",
            vec![Track::straight(Bottom), Track::hard_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "28",
            vec![Track::gentle_r(Bottom), Track::hard_r(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "29",
            vec![Track::gentle_l(Bottom), Track::hard_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "30",
            vec![Track::hard_l(Bottom), Track::gentle_r(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "31",
            vec![Track::hard_r(Bottom), Track::gentle_l(Bottom)],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "39",
            vec![
                Track::gentle_l(Bottom),
                Track::hard_l(Bottom),
                Track::hard_l(LowerLeft),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "40",
            vec![
                Track::gentle_l(Bottom),
                Track::gentle_l(UpperLeft),
                Track::gentle_l(UpperRight),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "41",
            vec![
                Track::straight(Bottom),
                Track::gentle_r(Bottom),
                Track::hard_l(Top),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "42",
            vec![
                Track::straight(Bottom),
                Track::gentle_l(Bottom),
                Track::hard_r(Top),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "43",
            vec![
                Track::straight(Bottom),
                Track::gentle_l(Bottom),
                Track::hard_l(LowerLeft),
                Track::gentle_l(LowerLeft),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "44",
            vec![
                Track::straight(Bottom),
                Track::hard_l(Bottom),
                Track::hard_l(Top),
                Track::straight(LowerLeft),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "45",
            vec![
                Track::gentle_l(UpperLeft),
                Track::hard_r(Top),
                Track::gentle_r(Bottom),
                Track::straight(Bottom),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "46",
            vec![
                Track::gentle_l(UpperLeft),
                Track::hard_l(Top),
                Track::gentle_l(Bottom),
                Track::straight(Bottom),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Brown,
            "47",
            vec![
                Track::straight(Bottom),
                Track::gentle_r(Bottom),
                Track::gentle_l(LowerLeft),
                Track::straight(LowerLeft),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Yellow,
            "57",
            vec![Track::mid(Bottom), Track::mid(Top)],
            vec![City::single(20)],
            hex,
        )
        .label(Label::Revenue(0), UpperLeft.to_centre(0.2)),
        Tile::new(
            Yellow,
            "58",
            vec![
                Track::gentle_r(Bottom)
                    .with_span(0.0, 0.5)
                    .with_dit(End, 10, Bar),
                Track::gentle_r(Bottom).with_span(0.5, 1.0),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), UpperLeft.to_centre(0.5)),
        Tile::new(
            Brown,
            "63",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
                Track::mid(LowerRight),
            ],
            vec![City::double(40)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        Tile::new(
            Brown,
            "70",
            vec![
                Track::gentle_l(Top),
                Track::hard_l(Top),
                Track::gentle_r(Bottom),
                Track::hard_r(Bottom),
            ],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "87",
            vec![
                Track::mid(Bottom).with_dit(End, 10, Circle),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), Right.to_centre(0.4)),
        Tile::new(
            Green,
            "88",
            vec![
                Track::mid(Bottom).with_dit(End, 10, Circle),
                Track::mid(LowerRight),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), UpperRight.to_centre(0.2)),
        Tile::new(
            Green,
            "120",
            vec![
                Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                Track::hard_l(Top).with_span(0.0, 0.5),
                Track::hard_l(Top).with_span(0.5, 1.0),
            ],
            vec![
                City::single_at_corner(60, &Left),
                City::single_at_corner(60, &TopRight),
            ],
            hex,
        )
        .label(
            Label::City("T".to_string()),
            LowerRight.in_dir(Direction::W, 0.15),
        )
        .label(Label::Revenue(0), Centre(None)),
        Tile::new(
            Brown,
            "122",
            vec![
                Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                Track::hard_l(Top).with_span(0.0, 0.5),
                Track::hard_l(Top).with_span(0.5, 1.0),
            ],
            vec![
                City::double_at_corner(80, &Left),
                City::double_at_corner(80, &TopRight),
            ],
            hex,
        )
        .label(
            Label::City("T".to_string()),
            BottomRight.in_dir(Direction::N, 0.2),
        )
        .label(Label::Revenue(0), Centre(None)),
        Tile::new(
            Grey,
            "124",
            vec![
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            vec![City::quad(100)],
            hex,
        )
        .label(Label::City("T".to_string()), TopRight.to_centre(0.05))
        .label(Label::Revenue(0), Right.to_centre(0.08)),
        Tile::new(
            Yellow,
            "201",
            vec![Track::mid(Bottom), Track::mid(LowerRight)],
            vec![City::single(30)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.25))
        .label(Label::Y, LowerLeft.to_centre(0.2)),
        Tile::new(
            Yellow,
            "202",
            vec![Track::mid(Bottom), Track::mid(UpperRight)],
            vec![City::single(30)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.25))
        .label(Label::Y, LowerLeft.to_centre(0.2)),
        Tile::new(
            Green,
            "204",
            vec![
                Track::mid(Bottom).with_dit(End, 10, Circle),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), LowerLeft.to_centre(0.25)),
        Tile::new(
            Green,
            "207",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            vec![City::double(40)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.15))
        .label(Label::Y, TopRight.to_centre(0.15)),
        Tile::new(
            Green,
            "208",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperRight),
                Track::mid(Top),
            ],
            vec![City::double(40)],
            hex,
        )
        .label(Label::Revenue(0), BottomLeft.to_centre(0.15))
        .label(Label::Y, TopLeft.to_centre(0.15)),
        Tile::new(
            Brown,
            "611",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            vec![City::double(40)],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.125)),
        Tile::new(
            Green,
            "619",
            vec![
                Track::mid(Bottom),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            vec![City::double(30)],
            hex,
        )
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Yellow,
            "621",
            vec![
                Track::straight(Bottom).with_span(0.0, 0.5),
                Track::straight(Bottom).with_span(0.5, 1.0),
            ],
            vec![City::single(30)],
            hex,
        )
        .label(Label::Revenue(0), UpperLeft.to_centre(0.1))
        .label(Label::Y, LowerLeft.to_centre(0.2)),
        Tile::new(
            Green,
            "622",
            vec![
                Track::mid(Bottom),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
            ],
            vec![City::double(40)],
            hex,
        )
        .label(Label::Revenue(0), TopRight.to_centre(0.15))
        .label(Label::Y, BottomLeft.to_centre(0.15)),
        Tile::new(
            Brown,
            "623",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
                Track::mid(LowerRight),
            ],
            vec![City::double(50)],
            hex,
        )
        .label(Label::Y, TopRight.to_centre(0.15))
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Green,
            "624",
            vec![Track::hard_l(Bottom), Track::hard_l(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "625",
            vec![Track::hard_r(Bottom), Track::hard_l(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "626",
            vec![Track::hard_r(LowerRight), Track::hard_l(LowerLeft)],
            vec![],
            hex,
        ),
        Tile::new(
            Green,
            "637",
            vec![
                Track::hard_l(Bottom).with_span(0.0, 0.5),
                Track::hard_l(Bottom).with_span(0.5, 1.0),
                Track::hard_l(UpperLeft).with_span(0.0, 0.5),
                Track::hard_l(UpperLeft).with_span(0.5, 1.0),
                Track::hard_l(UpperRight).with_span(0.0, 0.5),
                Track::hard_l(UpperRight).with_span(0.5, 1.0),
            ],
            vec![
                City::single_at_corner(50, &BottomLeft),
                City::single_at_corner(50, &TopLeft),
                City::single_at_corner(50, &Right),
            ],
            hex,
        )
        .label(Label::City("M".to_string()), Left.to_centre(0.25))
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Grey,
            "639",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
                Track::mid(LowerRight),
            ],
            vec![City::quad(100)],
            hex,
        )
        .label(Label::City("M".to_string()), TopRight.to_centre(0.05))
        .label(Label::Revenue(0), Right.to_centre(0.08)),
        Tile::new(
            Brown,
            "801",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
            ],
            vec![City::double(50)],
            hex,
        )
        .label(Label::Y, Right.to_centre(0.2))
        .label(Label::Revenue(0), TopRight.to_centre(0.15)),
        Tile::new(
            Brown,
            "911",
            vec![
                Track::mid(Bottom).with_dit(End, 10, Circle),
                Track::mid(LowerLeft),
                Track::mid(Top),
                Track::mid(UpperRight),
                Track::mid(LowerRight),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), UpperLeft.to_centre(0.25)),
        Tile::new(
            Green,
            "X1",
            vec![
                Track::straight(Bottom).with_span(0.0, 0.9),
                Track::straight(Bottom).with_span(0.9, 1.0),
                Track::straight(LowerLeft).with_span(0.0, 0.1),
                Track::straight(LowerLeft).with_span(0.1, 1.0),
                Track::straight(LowerRight).with_span(0.0, 0.1),
                Track::straight(LowerRight).with_span(0.1, 1.0),
            ],
            vec![
                City::single_at_face(50, &Top),
                City::single_at_face(50, &LowerLeft),
                City::single_at_face(50, &LowerRight),
            ],
            hex,
        )
        .label(
            Label::City("M".to_string()),
            BottomLeft.in_dir(Direction::E, 0.05),
        )
        .label(Label::Revenue(0), TopLeft.in_dir(Direction::S30W, 0.16)),
        Tile::new(
            Green,
            "X2",
            vec![
                Track::gentle_r(LowerLeft).with_span(0.0, 0.9),
                Track::gentle_r(LowerLeft).with_span(0.9, 1.0),
                Track::gentle_l(UpperLeft).with_span(0.0, 0.1),
                Track::gentle_l(UpperLeft).with_span(0.1, 1.0),
                Track::straight(Bottom).with_span(0.0, 0.9),
                Track::straight(Bottom).with_span(0.9, 1.0),
            ],
            vec![
                City::single_at_face(50, &Top),
                City::single_at_face(50, &UpperLeft),
                City::single_at_face(50, &LowerRight),
            ],
            hex,
        )
        .label(
            Label::City("M".to_string()),
            BottomLeft.in_dir(Direction::E, 0.05),
        )
        .label(Label::Revenue(0), Right.in_dir(Direction::N60W, 0.15)),
        Tile::new(
            Green,
            "X3",
            vec![
                Track::gentle_l(Top).with_span(0.0, 0.1),
                Track::gentle_l(Top).with_span(0.1, 1.0),
                Track::gentle_r(Bottom).with_span(0.0, 0.1),
                Track::gentle_r(Bottom).with_span(0.1, 1.0),
                Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                Track::hard_l(LowerLeft).with_span(0.5, 1.0),
            ],
            vec![
                City::single_at_face(50, &Top),
                City::single_at_face(50, &Bottom),
                City::single_at_corner(50, &Left),
            ],
            hex,
        )
        .label(
            Label::City("M".to_string()),
            BottomLeft.in_dir(Direction::N30W, 0.1),
        )
        .label(Label::Revenue(0), TopLeft.in_dir(Direction::S30W, 0.16)),
        Tile::new(
            Green,
            "X4",
            vec![
                Track::straight(Top).with_span(0.0, 0.1),
                Track::straight(Top).with_span(0.1, 1.0),
                Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                Track::hard_r(LowerRight).with_span(0.0, 0.5),
                Track::hard_r(LowerRight).with_span(0.5, 1.0),
            ],
            vec![
                City::single_at_face(50, &Top),
                City::single_at_corner(50, &Left),
                City::single_at_corner(50, &Right),
            ],
            hex,
        )
        .label(
            Label::City("M".to_string()),
            BottomRight.in_dir(Direction::N, 0.2),
        )
        .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
        Tile::new(
            Brown,
            "X5",
            vec![
                Track::straight(Top).with_span(0.0, 0.1),
                Track::straight(Top)
                    .with_span(0.1, 1.0)
                    .with_clip(0.3625, 0.75),
                Track::mid(UpperLeft),
                Track::mid(LowerLeft),
                Track::mid(LowerRight),
                Track::mid(UpperRight),
            ],
            vec![
                City::single_at_face(70, &Top),
                City::double(70).in_dir(Direction::S, 0.1),
            ],
            hex,
        )
        .label(
            Label::City("M".to_string()),
            BottomLeft.in_dir(Direction::E, 0.05),
        )
        .label(Label::Revenue(0), Left.to_centre(0.1)),
        Tile::new(
            Brown,
            "X6",
            vec![
                Track::hard_l(LowerLeft).with_span(0.0, 0.5),
                Track::hard_l(LowerLeft).with_span(0.5, 1.0),
                Track::mid(Top),
                Track::mid(Bottom),
                Track::mid(LowerRight),
                Track::mid(UpperRight),
            ],
            vec![
                City::single_at_corner(70, &Left),
                City::double(70)
                    .rotate(Rotation::Cw90)
                    .in_dir(Direction::E, 0.1),
            ],
            hex,
        )
        .label(
            Label::City("M".to_string()),
            BottomLeft.in_dir(Direction::E, 0.05),
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Brown,
            "X7",
            vec![
                Track::gentle_l(UpperLeft).with_span(0.0, 0.9),
                Track::gentle_l(UpperLeft).with_span(0.9, 1.0),
                Track::gentle_r(LowerLeft).with_span(0.0, 0.5),
                Track::gentle_l(LowerRight).with_span(0.0, 0.5),
                Track::straight(Top).with_span(0.0, 0.65),
                Track::straight(Bottom).with_span(0.0, 0.35),
            ],
            vec![
                City::single_at_face(70, &UpperRight),
                City::double(70).in_dir(Direction::S, 0.3),
            ],
            hex,
        )
        .label(Label::City("M".to_string()), Left.to_centre(0.15))
        .label(Label::Revenue(0), TopLeft.to_centre(0.15)),
        Tile::new(
            Grey,
            "X8",
            vec![
                Track::mid(Bottom),
                Track::mid(LowerLeft),
                Track::mid(UpperLeft),
                Track::mid(Top),
                Track::mid(LowerRight),
                Track::mid(UpperRight),
            ],
            vec![City::triple(60).rotate(Rotation::HalfTurn)],
            hex,
        )
        .label(Label::City("O".to_string()), Left.to_centre(0.15))
        .label(Label::Revenue(0), BottomLeft.to_centre(0.1)),
        Tile::new(
            Yellow,
            "IN10",
            vec![
                Track::gentle_l(Bottom)
                    .with_span(0.0, 0.85)
                    .with_dit(End, 30, Bar),
                Track::gentle_l(Bottom).with_span(0.85, 1.0),
                Track::gentle_r(Bottom)
                    .with_span(0.0, 0.85)
                    .with_dit(End, 30, Bar),
                Track::gentle_r(Bottom).with_span(0.85, 1.0),
                Track::straight(UpperLeft).with_span(0.125, 1.0),
                Track::gentle_l(Top),
            ],
            vec![],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
        Tile::new(
            Green,
            "IN11",
            vec![
                Track::straight(LowerRight),
                Track::gentle_r(LowerRight).with_span(0.0, 0.5),
                Track::gentle_r(LowerRight).with_span(0.5, 1.0),
                Track::gentle_l(Bottom).with_span(0.0, 0.5),
                Track::gentle_l(Bottom).with_span(0.5, 1.0),
                Track::straight(Bottom),
            ],
            vec![
                City::single_at_face(30, &LowerLeft)
                    .in_dir(Direction::N60E, 0.2),
                City::single_at_face(30, &UpperRight)
                    .in_dir(Direction::S60W, 0.2),
            ],
            hex,
        )
        .label(Label::Revenue(0), TopLeft.to_centre(0.1)),
    ]
}
