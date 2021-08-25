//! Provide a convenient way to construct arbitrary paths.
//!
//! # Example
//!
//! ```rust,no_run
//! # extern crate cairo;
//! # use cairo::{Context, ImageSurface};
//! # use n18hex::{Colour, Hex};
//! # use n18map::Map;
//! # use n18token::Tokens;
//! # use n18route::builder::Result;
//! use n18route::builder::RouteBuilder;
//! use n18hex::HexFace;
//! use n18brush::highlight_route;
//!
//! # fn main() -> Result<()> {
//! # let hex = Hex::new(125.0);
//! # let tiles: Vec<n18tile::Tile> = vec![];
//! # let map = Map::new(tiles.into(), Tokens::new(vec![]), vec![]);
//! # let surf = cairo::ImageSurface::create(cairo::Format::ARgb32, 10, 10)
//! #     .unwrap();
//! # let ctx = cairo::Context::new(&surf).unwrap();
//! // let hex: n18hex::Hex = ...
//! // let map: n18map::Map = ...
//! // let ctx: cairo::Context = ...
//! let route = RouteBuilder::from_edge(&map, "A1", HexFace::LowerRight)?
//!     .to_city(0, true)?
//!     .to_edge(HexFace::Bottom)?
//!     .to_edge(HexFace::Bottom)?
//!     .to_city(0, true)?
//!     .to_edge(HexFace::LowerRight)?
//!     .to_edge(HexFace::UpperRight)?
//!     .to_edge(HexFace::LowerRight)?
//!     .to_city(0, false)?
//!     .to_edge(HexFace::Bottom)?
//!     .into_route();
//!
//! let highlight = Colour::from((179, 25, 25));
//! highlight.apply_colour(&ctx);
//! highlight_route(&hex, &ctx, &map, &route);
//! # Ok(())
//! # }
//! ```
//!

use super::{Path, Route, Step, StopLocation, Visit};
use n18hex::HexFace;
use n18map::{HexAddress, Map};
use n18tile::{Connection, Tile};
use std::collections::BTreeSet;

/// The different ways in which building a path may fail.
pub enum Error {
    /// A string could not be parsed as a valid hex address.
    InvalidHexAddress(String),
    /// There is no tile placed on the specified hex.
    NoTileAtHex(HexAddress),
    /// Unable to connect to a city space that does not exist.
    InvalidCity(HexAddress, usize),
    /// Unable to connect to a dit that does not exist.
    InvalidDit(HexAddress, usize),
    /// Unable to find a path to connect two locations on a tile.
    NotConnected(HexAddress, Connection, Connection),
}

/// Shorthand result type for `RouteBuilder` operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Required in order to implement `std::error::Error`.
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Error::*;
        match self {
            InvalidHexAddress(s) => write!(f, "Invalid hex address {}", s),
            NoTileAtHex(addr) => write!(f, "No tile at hex {}", addr),
            InvalidCity(addr, ix) => {
                write!(f, "No city #{} at hex {}", ix, addr)
            }
            InvalidDit(addr, ix) => {
                write!(f, "No dit #{} at hex {}", ix, addr)
            }
            NotConnected(addr, _src, dest) => match dest {
                Connection::City { ix } => {
                    write!(f, "No connection to city #{} on hex {}", ix, addr)
                }
                Connection::Dit { ix } => {
                    write!(f, "No connection to dit #{} on hex {}", ix, addr)
                }
                Connection::Face { face } => {
                    write!(f, "No connection to {:?} on hex {}", face, addr)
                }
                Connection::Track { ix, .. } => write!(
                    f,
                    "No connection to track #{} on hex {}",
                    ix, addr
                ),
            },
        }
    }
}

/// Required in order to implement `std::error::Error`.
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

/// Provides a builder interface for constructing routes.
pub struct RouteBuilder<'a> {
    map: &'a Map,
    steps: Vec<Step>,
    visits: Vec<Visit>,
    num_visits: usize,
    num_cities: usize,
    num_dits: usize,
    num_hexes: usize,
}

fn edge_to_tile_face(
    map: &Map,
    addr: HexAddress,
    face: HexFace,
) -> Result<HexFace> {
    let rotn = map
        .hex_state(addr)
        .ok_or(Error::NoTileAtHex(addr))?
        .rotation();
    let num_cw_turns = rotn.count_turns();
    let mut tile_face = face;
    for _ in 0..num_cw_turns {
        // NOTE: "undo" each clockwise rotation.
        // Switch this to .clockwise() to convert tile faces to map edges.
        tile_face = tile_face.anti_clockwise()
    }
    Ok(tile_face)
}

impl<'a> RouteBuilder<'a> {
    fn new(map: &'a Map, start: Step) -> Self {
        // NOTE: allowing paths to start at a hex face could be useful for,
        // e.g., illustrating conflicting segments of multiple paths.
        let (stop, cities, dits) = match start.conn {
            Connection::City { ix } => {
                (Some(StopLocation::City { ix }), 1, 0)
            }
            Connection::Dit { ix } => (Some(StopLocation::Dit { ix }), 0, 1),
            Connection::Track { .. } => (None, 0, 0),
            Connection::Face { .. } => (None, 0, 0),
        };
        let initial_visit = if let Some(stop) = stop {
            let visit = Visit {
                addr: start.addr,
                // NOTE: visit will only be highlighted by n18brush if they
                // have positive revenue.
                revenue: 1,
                visits: stop,
            };
            vec![visit]
        } else {
            vec![]
        };
        RouteBuilder {
            map,
            steps: vec![start],
            visits: initial_visit,
            num_visits: 1,
            num_cities: cities,
            num_dits: dits,
            num_hexes: 1,
        }
    }

    /// Start building a path from the edge of a tile, where `face` is
    /// specified with respect to the tile's innate orientation, rather than
    /// the tile's rotation on the map.
    pub fn from_tile_face(
        map: &'a Map,
        addr: &str,
        face: HexFace,
    ) -> Result<Self> {
        let addr = addr
            .parse()
            .map_err(|_| Error::InvalidHexAddress(addr.to_string()))?;
        map.tile_at(addr).ok_or(Error::NoTileAtHex(addr))?;
        let start = Step {
            addr,
            conn: Connection::Face { face },
        };
        Ok(Self::new(map, start))
    }

    /// Start building a path from the edge of a tile, where `face` is
    /// specified with respect to the tile's rotation on the map, rather than
    /// the tile's innate orientation.
    pub fn from_edge(
        map: &'a Map,
        addr: &str,
        face: HexFace,
    ) -> Result<Self> {
        let tile_addr = addr
            .parse()
            .map_err(|_| Error::InvalidHexAddress(addr.to_string()))?;
        let tile_face = edge_to_tile_face(map, tile_addr, face)?;
        Self::from_tile_face(map, addr, tile_face)
    }

    /// Start building a path from the city space `ix` of a tile.
    pub fn from_city(map: &'a Map, addr: &str, ix: usize) -> Result<Self> {
        let addr = addr
            .parse()
            .map_err(|_| Error::InvalidHexAddress(addr.to_string()))?;
        // NOTE: ensure that this initial step is valid.
        let tile = map.tile_at(addr).ok_or(Error::NoTileAtHex(addr))?;
        if tile.cities().len() <= ix {
            return Err(Error::InvalidCity(addr, ix));
        }
        let start = Step {
            addr,
            conn: Connection::City { ix },
        };
        Ok(Self::new(map, start))
    }

    /// Start building a path from the dit `ix` of a tile.
    pub fn from_dit(map: &'a Map, addr: &str, ix: usize) -> Result<Self> {
        let addr = addr
            .parse()
            .map_err(|_| Error::InvalidHexAddress(addr.to_string()))?;
        // NOTE: ensure that this initial step is valid.
        let tile = map.tile_at(addr).ok_or(Error::NoTileAtHex(addr))?;
        if tile.dits().len() <= ix {
            return Err(Error::InvalidDit(addr, ix));
        }
        let start = Step {
            addr,
            conn: Connection::Dit { ix },
        };
        Ok(Self::new(map, start))
    }

    /// Extend the path to the edge of the current tile, where `face` is
    /// specified with respect to the tile's innate orientation, and on to the
    /// edge of the adjacent tile (if any).
    // Note: allow this function to consume self, because we're using the "to"
    // prefix to refer to route connectivity, not type conversion.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_tile_face(mut self, face: HexFace) -> Result<Self> {
        // NOTE: this does not account for the tile rotation, it refers to the
        // tile's orientation as defined.
        // NOTE: this also adds a step to the adjacent hex face.
        let curr = self.steps.last().unwrap();
        let mut new_steps =
            self.find_steps_to(curr, Connection::Face { face })?;
        self.steps.append(&mut new_steps);
        self.num_hexes += 1;
        Ok(self)
    }

    /// Extend the path to the edge of the current tile, where `face` is
    /// specified with respect to the tile's rotation on the map, and on to
    /// the edge of the adjacent tile (if any).
    // Note: allow this function to consume self, because we're using the "to"
    // prefix to refer to route connectivity, not type conversion.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_edge(self, face: HexFace) -> Result<Self> {
        // Account for the tile rotation!
        let curr = self.steps.last().unwrap();
        let addr = curr.addr;
        let tile_face = edge_to_tile_face(self.map, addr, face)?;
        self.to_tile_face(tile_face)
    }

    /// Extend the path to the city space `ix` and, optionally, record this as
    /// a "stop" (i.e., earning revenue) so that it will be drawn as such by,
    /// e.g., `n18brush` functions.
    // Note: allow this function to consume self, because we're using the "to"
    // prefix to refer to route connectivity, not type conversion.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_city(mut self, ix: usize, stop: bool) -> Result<Self> {
        let curr = self.steps.last().unwrap();
        // NOTE: need to copy the address for the visit in order to add new
        // steps, or the borrow checker will complain that we have mutable and
        // non-mutable borrows of self.steps.
        let addr = curr.addr;
        let mut new_steps =
            self.find_steps_to(curr, Connection::City { ix })?;
        self.steps.append(&mut new_steps);
        let revenue = if stop { 1 } else { 0 };
        self.num_cities += 1;
        self.num_visits += 1;
        self.visits.push(Visit {
            addr,
            // NOTE: visit will only be highlighted by n18brush if they
            // have positive revenue.
            revenue,
            visits: StopLocation::City { ix },
        });
        Ok(self)
    }

    /// Extend the path to the dit `ix` and, optionally, record this as a
    /// "stop" (i.e., earning revenue) so that it will be drawn as such by,
    /// e.g., `n18brush` functions.
    // Note: allow this function to consume self, because we're using the "to"
    // prefix to refer to route connectivity, not type conversion.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_dit(mut self, ix: usize, stop: bool) -> Result<Self> {
        let curr = self.steps.last().unwrap();
        // NOTE: need to copy the address for the visit in order to add new
        // steps, or the borrow checker will complain that we have mutable and
        // non-mutable borrows of self.steps.
        let addr = curr.addr;
        let mut new_steps =
            self.find_steps_to(curr, Connection::Dit { ix })?;
        self.steps.append(&mut new_steps);
        let revenue = if stop { 1 } else { 0 };
        self.num_dits += 1;
        self.num_visits += 1;
        self.visits.push(Visit {
            addr,
            // NOTE: visit will only be highlighted by n18brush if they
            // have positive revenue.
            revenue,
            visits: StopLocation::Dit { ix },
        });
        Ok(self)
    }

    /// Construct the described path.
    ///
    /// The returned path can be drawn using `n18brush::highlight_path`.
    pub fn into_path(self) -> Path {
        Path {
            steps: self.steps,
            conflicts: BTreeSet::new(),
            route_conflicts: crate::conflict::RouteConflicts::new(),
            visits: self.visits,
            num_visits: self.num_visits,
            num_cities: self.num_cities,
            num_dits: self.num_dits,
            num_hexes: self.num_hexes,
            revenue: 0,
        }
    }

    /// Construct the described route.
    ///
    /// The returned route can be drawn using `n18brush::highlight_route`.
    pub fn into_route(self) -> Route {
        Route {
            steps: self.steps,
            visits: self.visits,
        }
    }

    fn find_steps_to(
        &self,
        src: &Step,
        dest: Connection,
    ) -> Result<Vec<Step>> {
        let mut seen: BTreeSet<Connection> = BTreeSet::new();
        let tile = self
            .map
            .tile_at(src.addr)
            .ok_or(Error::NoTileAtHex(src.addr))?;
        let mut steps = self
            .depth_first_search(
                &mut seen, tile, &src.addr, &src.conn, &dest, 0,
            )
            .ok_or(Error::NotConnected(src.addr, src.conn, dest))?;
        // NOTE: if dest is HexFace, also add the adjacent face on the next hex
        if let Connection::Face { face } = dest {
            let adj = self.map.adjacent_face(src.addr, face);
            if let Some((adj_addr, adj_face, _adj_tile)) = adj {
                let conn = Connection::Face { face: adj_face };
                let step = Step {
                    addr: adj_addr,
                    conn,
                };
                steps.push(step)
            } else {
                // TODO: no adjacent face, should we error here?
                // Or is this something we should percolate up to the
                // user-visible functions to_edge() and to_tile_face()?
            }
        }
        Ok(steps)
    }

    fn depth_first_search(
        &self,
        seen: &mut BTreeSet<Connection>,
        tile: &Tile,
        addr: &HexAddress,
        src: &Connection,
        dest: &Connection,
        level: usize,
    ) -> Option<Vec<Step>> {
        let mut found: Option<Vec<Step>> = None;
        for conn in tile.connections(src).unwrap() {
            if seen.contains(conn) {
                continue;
            } else {
                seen.insert(*conn)
            };

            // NOTE: if this is a track connection, switch to the other end.
            let other_end = conn.other_end().unwrap_or(*conn);
            let conn = if conn != &other_end {
                if seen.contains(&other_end) {
                    continue;
                } else {
                    seen.insert(other_end);
                    &other_end
                }
            } else {
                conn
            };

            if conn == dest {
                found = Some(vec![Step {
                    addr: *addr,
                    conn: *conn,
                }]);
                break;
            } else {
                let steps_opt = self.depth_first_search(
                    seen,
                    tile,
                    addr,
                    conn,
                    dest,
                    level + 1,
                );
                if let Some(mut steps) = steps_opt {
                    // NOTE: need to insert the source connection.
                    steps.insert(
                        0,
                        Step {
                            addr: *addr,
                            conn: *conn,
                        },
                    );
                    found = Some(steps);
                    break;
                }
            }
        }
        found
    }
}
