//! Update placed tokens when upgrading tiles.
//!
//! When one tile (the "original") is upgraded to a new tile, any token placed
//! on the original tile should ideally be placed on the new tile in a token
//! space that preserves the token's connectivity on the original tile.
//!
//! This can be expressed as a
//! [maximum flow problem](https://en.wikipedia.org/wiki/Maximum_flow_problem)
//! and solved using a range of well-known algorithms.
//!
//! We use the Edmonds-Karp algorithm, which is provided by
//! [n18tile::ekmf](super::ekmf).
//!
//! # Details
//!
//! The problem of placing tokens on the new tile can be divided into several
//! steps:
//!
//! 1. For each token on the original tile, identify the tile faces to which
//!    it is connected.
//! 2. For each city on the new tile, identify the tile faces to which it is
//!    connected.
//! 3. For each token on the original tile, identify the cities (if any) on
//!    the new tile that have the required tile face connections.
//! 4. Construct a flow network that contains:
//!    - A node for each token on the original tile;
//!    - A node for each city on the new tile;
//!    - A connection from the source node to each token node, with capacity
//!      `1`.
//!    - Connections from each token to the cities identified in step 3, with
//!      capacity `1`; and
//!    - A connection from each city node to the sink node, with capacity
//!      equal to the number of token spaces in that city.
//! 5. Calculate the maximum flow through this network.
//!    - If this flow is less than the number of tokens on the original tile,
//!      the tokens cannot be placed on the new tile.
//!    - Otherwise, the tokens can be placed on the new tile.
//! 6. If the tokens can be placed on the new tile, examine the returned flow
//!    matrix; a non-zero flow from a token node to a city node indicates that
//!    the token should be placed in the corresponding city.

use std::collections::{BTreeMap, BTreeSet};

use super::ekmf::Matrix;
use super::{Connection, Tile, TokenSpace};
use n18hex::{HexFace, RotateCW};

/// The state of each token space on a tile.
pub type TokenIndex = BTreeMap<TokenSpace, usize>;

/// Returns the hexagon faces that are connected to each city on `tile`, when
/// accounting for the tile's rotation `rotn`.
fn city_face_connections(
    tile: &Tile,
    rotn: &RotateCW,
) -> BTreeMap<usize, BTreeSet<HexFace>> {
    let mut city_faces: BTreeMap<usize, BTreeSet<HexFace>> = BTreeMap::new();
    let city_count = tile.cities().len();
    for ix in 0..city_count {
        let start = Connection::City { ix };
        // NOTE: apply the tile rotation, so that faces correspond to the
        // tile's orientation.
        let faces: BTreeSet<HexFace> = tile
            .connected_faces(start)
            .iter()
            .map(|&face| face + rotn)
            .collect();
        city_faces.insert(ix, faces);
    }
    city_faces
}

/// Returns the hexagon faces that are connected to each token on `tile`, when
/// accounting for the tile's rotation `rotn`.
fn token_face_connections(
    tile: &Tile,
    rotn: &RotateCW,
    tokens: &TokenIndex,
) -> BTreeMap<usize, BTreeSet<HexFace>> {
    let mut token_faces: BTreeMap<usize, BTreeSet<HexFace>> = BTreeMap::new();

    for (ix, (token_space, _token)) in tokens.iter().enumerate() {
        let start = Connection::City {
            ix: token_space.city_ix(),
        };
        // NOTE: apply the tile rotation, so that faces correspond to the
        // tile's orientation.
        let want_faces: BTreeSet<HexFace> = tile
            .connected_faces(start)
            .iter()
            .map(|&face| face + rotn)
            .collect();
        token_faces.insert(ix, want_faces);
    }
    token_faces
}

/// Returns the cities in `city_faces` that are valid for each token in
/// `token_faces`.
fn valid_cities(
    token_faces: BTreeMap<usize, BTreeSet<HexFace>>,
    city_faces: BTreeMap<usize, BTreeSet<HexFace>>,
) -> BTreeMap<usize, Vec<usize>> {
    token_faces
        .into_iter()
        .map(|(token_ix, want_faces)| {
            let city_ixs: Vec<usize> = city_faces
                .iter()
                .filter_map(|(&city_ix, city_faces)| {
                    if want_faces.is_subset(city_faces) {
                        Some(city_ix)
                    } else {
                        None
                    }
                })
                .collect();
            (token_ix, city_ixs)
        })
        .collect()
}

/// Returns a flow matrix where each token is connected to the cities where
/// they can be placed.
///
/// The first node is the source, followed by a node for each token, then by a
/// node for each city, and the final node is the sink.
fn flow_matrix(
    tile: &Tile,
    token_cities: BTreeMap<usize, Vec<usize>>,
) -> Matrix {
    let token_count = token_cities.len();
    let city_count = tile.cities().len();
    let n = token_count + city_count + 2;
    let mut capacities = Matrix::square(n);

    for (token_ix, city_ixs) in token_cities.iter() {
        // Connect the source node to the token node.
        let source_ix = token_ix + 1;
        capacities[(0, source_ix)] = 1;
        for city_ix in city_ixs {
            // Connect the token node to each city node in turn.
            let dest_ix = city_ix + token_count + 1;
            capacities[(source_ix, dest_ix)] = 1;
        }
    }

    for ix in 0..city_count {
        // Connect each city node to the sink node.
        let node_ix = ix + token_count + 1;
        capacities[(node_ix, n - 1)] = tile.cities()[ix].tokens.count();
    }

    capacities
}

/// Returns the placement of each allocated token in the provided flow matrix.
fn flow_matrix_tokens(
    flow_mat: Matrix,
    tokens: &TokenIndex,
    tile: &Tile,
) -> Option<TokenIndex> {
    let token_count = tokens.len();
    let city_count = tile.cities().len();
    let mut tokens_table: TokenIndex = BTreeMap::new();
    let tokens_iter = tokens.values();

    for (token_n, token) in tokens_iter.enumerate() {
        let token_ix = token_n + 1;
        // Find the city, if any, to which this token is connected.
        let city_n = (0..city_count).find(|city_n| {
            let city_ix = city_n + token_count + 1;
            flow_mat[(token_ix, city_ix)] == 1
        });
        if let Some(n) = city_n {
            // Place token in the next free space in this city.
            let tok_spaces = tile.city_token_spaces(n);
            let free_space_opt =
                tok_spaces.iter().find(|ts| !tokens_table.contains_key(ts));
            if let Some(tok_space) = free_space_opt {
                tokens_table.insert(*tok_space, *token);
            } else {
                // NOTE: this should not be possible.
                panic!("Token #{} has no token space?", token_n);
            }
        } else {
            // NOTE: this should not be possible.
            panic!("Token #{} -> NOWHERE", token_n);
        }
    }

    Some(tokens_table)
}

/// Attempts to place each token from `old_tile` on `new_tile` in such a way
/// so as to preserve each token's connectivity with adjacent hexes.
pub fn try_placing_tokens(
    orig_tile: &Tile,
    orig_rotn: &RotateCW,
    tokens: &TokenIndex,
    new_tile: &Tile,
    new_rotn: &RotateCW,
) -> Option<TokenIndex> {
    if tokens.is_empty() {
        return None;
    }

    // Identify the faces connected to each token on the original tile.
    let token_faces = token_face_connections(orig_tile, orig_rotn, tokens);
    // Identify the faces connected to each city on the new tile.
    let city_faces = city_face_connections(new_tile, new_rotn);
    // For each token, identify the cities on the new tile where that token
    // can be placed.
    let token_cities = valid_cities(token_faces, city_faces);
    // Construct a flow matrix that connects tokens and cities.
    let capacities = flow_matrix(new_tile, token_cities);
    // Calculate the maximum flow through this network, and record the
    // allocation of tokens to cities.
    let (net_flow, flow_mat) = capacities.max_flow_mat();
    // If we could not place all of the tokens, do not place any tokens.
    if net_flow != tokens.len() {
        return None;
    }
    // Extract the placement of each token on the new tile.
    flow_matrix_tokens(flow_mat, tokens, new_tile)
}
