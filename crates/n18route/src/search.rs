//! Search a map for paths from a starting location.

use log::info;

use std::collections::BTreeSet;

use super::conflict::{Conflict, ConflictRule};
use super::{Path, Step, StopLocation, Visit};
use n18hex::HexColour;
use n18map::{HexAddress, Map};
use n18tile::{Connection, Tile, TokenSpace};
use n18token::Token;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathLimit {
    Cities { count: usize },
    CitiesAndTowns { count: usize },
    Hexes { count: usize },
}

/// The search criteria for identifying valid paths that start from a specific
/// location.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Query {
    pub addr: HexAddress,
    pub from: Connection,
    pub criteria: Criteria,
}

/// The search criteria for identifying valid paths.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Criteria {
    pub token: Token,
    pub path_limit: Option<PathLimit>,
    /// The rule that governs what elements can be shared in a single route.
    pub conflict_rule: ConflictRule,
    /// The rule that governs what elements can be shared between routes.
    pub route_conflict_rule: ConflictRule,
}

/// The current state of the path-exploration algorithm.
struct Context {
    /// The previous steps in this path.
    path: Vec<Step>,
    /// The existing path elements that may not be re-used.
    conflicts: BTreeSet<Conflict>,
    /// The existing path elements that may not be re-used by other routes.
    route_conflicts: BTreeSet<Conflict>,
    /// The cities and dits that have been visited, possibly for revenue.
    visits: Vec<Visit>,
    /// The number cities and dits that have been visited.
    num_visits: usize,
    /// The number of cities that have been visited.
    num_cities: usize,
    /// The number of dits that have been visited.
    num_dits: usize,
    /// The number of hexes that have been visited.
    num_hexes: usize,
}

impl Context {
    fn new(map: &Map, query: &Query) -> Self {
        let path: Vec<Step> = vec![Step {
            addr: query.addr,
            conn: query.from,
        }];
        let mut conflicts = BTreeSet::new();
        if let Some(conflict) = query
            .criteria
            .conflict_rule
            .maybe_conflict(&query.addr, &query.from)
        {
            conflicts.insert(conflict);
        }
        let mut route_conflicts = BTreeSet::new();
        if let Some(conflict) = query
            .criteria
            .route_conflict_rule
            .maybe_conflict(&query.addr, &query.from)
        {
            route_conflicts.insert(conflict);
        }

        if query.criteria.route_conflict_rule >= query.criteria.conflict_rule
        {
            panic!("Route conflict rule must be more general than path conflict rule")
        }

        // NOTE: record the starting city/dit and its revenue.
        let tile = map.tile_at(query.addr).unwrap();
        let (first_stop, num_cities, num_dits) = match query.from {
            Connection::City { ix: city_ix } => {
                let city = tile.cities()[city_ix];
                (
                    Visit {
                        addr: query.addr,
                        revenue: city.revenue,
                        visits: StopLocation::City { ix: city_ix },
                    },
                    1,
                    0,
                )
            }
            Connection::Dit { ix: dit_ix } => {
                let dit = tile.dits()[dit_ix];
                (
                    Visit {
                        addr: query.addr,
                        revenue: dit.revenue,
                        visits: StopLocation::Dit { ix: dit_ix },
                    },
                    0,
                    1,
                )
            }
            _ => panic!("Invalid starting connection"),
        };

        Context {
            path,
            conflicts,
            route_conflicts,
            visits: vec![first_stop],
            num_visits: 1,
            num_cities,
            num_dits,
            num_hexes: 1,
        }
    }

    fn current_path(&self) -> Path {
        Path {
            steps: self.path.clone(),
            conflicts: self.conflicts.clone(),
            route_conflicts: (&self.route_conflicts).into(),
            visits: self.visits.clone(),
            num_visits: self.visits.len(),
            num_cities: self.num_cities,
            num_dits: self.num_dits,
            num_hexes: self.num_hexes,
            revenue: self.visits.iter().map(|visit| visit.revenue).sum(),
        }
    }

    fn can_continue(&self, path_limit: &Option<PathLimit>) -> bool {
        if let Some(limit) = path_limit {
            match limit {
                PathLimit::Cities { count } => self.num_cities < *count,
                PathLimit::CitiesAndTowns { count } => {
                    self.num_visits < *count
                }
                PathLimit::Hexes { count } => self.num_hexes < *count,
            }
        } else {
            true
        }
    }
}

/// Returns all valid paths that match the provided criteria and which pass
/// through any matching token on the map.
pub fn paths_for_token(map: &Map, criteria: &Criteria) -> Vec<Path> {
    let locations: Vec<(HexAddress, TokenSpace)> = map
        .find_placed_tokens(&criteria.token)
        .iter()
        .map(|(addr, token_space)| (**addr, **token_space))
        .collect();
    // Allow the search from each token to proceed in parallel.
    use rayon::prelude::*;
    info!("Searching for paths from {} locations", locations.len());
    let paths = locations
        .par_iter()
        .flat_map(|(addr, token_space)| {
            let query = Query {
                addr: *addr,
                from: Connection::City {
                    ix: token_space.city_ix(),
                },
                criteria: *criteria,
            };
            let paths = paths_through(map, &query);
            info!("Found {} paths that pass through {}", paths.len(), addr);
            paths
        })
        .collect::<Vec<Path>>();
    info!("Found {} paths in total", paths.len());
    paths
}

/// Returns all valid paths that match the provided criteria, passing through
/// the specified token.
pub fn paths_through(map: &Map, query: &Query) -> Vec<Path> {
    let mut paths = paths_from(map, query);
    let mut extra_paths = path_combinations(query, &paths);
    paths.append(&mut extra_paths);
    paths
}

/// Returns all valid paths that match the provided criteria, starting from
/// the specified token.
pub fn paths_from(map: &Map, query: &Query) -> Vec<Path> {
    let mut context = Context::new(map, query);
    let mut paths: Vec<Path> = vec![];
    let start_tile = map.tile_at(query.addr).unwrap();
    // NOTE: it is conceivable (although perhaps not sensible) that a token
    // could be placed in a token space that is not connected to any track
    // segments, in which case `start_tile.connections()` will return `None`,
    // and this is best handled by returning an empty vector.
    let conns_opt = start_tile.connections(&query.from);
    let connections = if let Some(conns) = conns_opt {
        conns
    } else {
        return vec![];
    };
    for conn in connections.iter() {
        depth_first_search(
            map,
            query,
            &mut context,
            &mut paths,
            query.addr,
            *conn,
            start_tile,
        )
    }
    paths
}

/// Returns all valid combination of path pairs, which must all start from the
/// same location.
fn path_combinations(query: &Query, paths: &[Path]) -> Vec<Path> {
    // NOTE: all of the paths start from the same token space.
    // If more than 2 stops are allowed, and/or if cities can be skipped
    // (including the token space itself), then we also need to consider
    // joining pairs of paths together.
    let mut new_paths: Vec<Path> = vec![];

    // Loop over each pair of paths.
    for (i, path_i) in paths.iter().enumerate() {
        for path_j in paths.iter().skip(i + 1) {
            // First, check that these paths don't conflict with each other.
            let conflicts: BTreeSet<_> =
                path_i.conflicts.intersection(&path_j.conflicts).collect();
            if conflicts.len() != 1 {
                continue;
            }

            // Ensure that the combination of these two paths doesn't exceed
            // the path limit (if any).
            let can_append = if let Some(limit) = query.criteria.path_limit {
                match limit {
                    PathLimit::Cities { count } => {
                        let n = path_i.num_cities + path_j.num_cities - 1;
                        n <= count
                    }
                    PathLimit::CitiesAndTowns { count } => {
                        let n = path_i.num_visits + path_j.num_visits - 1;
                        n <= count
                    }
                    PathLimit::Hexes { count } => {
                        let n = path_i.num_hexes + path_j.num_hexes - 1;
                        n <= count
                    }
                }
            } else {
                true
            };

            if can_append {
                let new_path = path_i.append(path_j);
                new_paths.push(new_path);
            }
        }
    }

    new_paths
}

fn dfs_over(
    map: &Map,
    query: &Query,
    ctx: &mut Context,
    paths: &mut Vec<Path>,
    addr: HexAddress,
    conns: Option<&[Connection]>,
    tile: &Tile,
) {
    if let Some(connections) = conns {
        for next_conn in connections.iter() {
            match next_conn {
                Connection::Face { face } => {
                    // If the connection is a hex face, we need to instead
                    // examine the connections attached to the adjacent face.
                    // NOTE: record this face and the adjacent face, so that
                    // routes that don't share any track segments but do
                    // share a hex face will be detected!!!
                    let adj = map.adjacent_face(addr, *face);
                    if let Some((new_addr, new_face, new_tile)) = adj {
                        let first_face = Step {
                            addr,
                            conn: *next_conn,
                        };
                        let second_face = Step {
                            addr: new_addr,
                            conn: Connection::Face { face: new_face },
                        };

                        // NOTE: record the hex face conflicts according to
                        // the map orientation, so that we always have an
                        // upper face and a lower face, and only need to
                        // record one of these.
                        let map_face_1 = map
                            .map_face_from_tile_face(addr, *face)
                            .expect("No map face for current tile");
                        let map_face_2 = map
                            .map_face_from_tile_face(new_addr, new_face)
                            .expect("No map face for adjacent tile");
                        let map_conn_1 =
                            Connection::Face { face: map_face_1 };
                        let map_conn_2 =
                            Connection::Face { face: map_face_2 };

                        // Record the traversed hex faces if a single route
                        // cannot reuse them.
                        let conflict_1 = query
                            .criteria
                            .conflict_rule
                            .maybe_conflict(&addr, &map_conn_1);
                        if let Some(conflict) = conflict_1 {
                            if ctx.conflicts.contains(&conflict) {
                                // Stop searching here.
                                return;
                            }
                            ctx.conflicts.insert(conflict);
                        }
                        let conflict_2 = query
                            .criteria
                            .conflict_rule
                            .maybe_conflict(&new_addr, &map_conn_2);
                        if let Some(conflict) = conflict_2 {
                            if ctx.conflicts.contains(&conflict) {
                                return;
                            }
                            ctx.conflicts.insert(conflict);
                        }

                        // Record the traversed hex faces if multiple routes
                        // cannot share them.
                        let route_conflict_1 = query
                            .criteria
                            .route_conflict_rule
                            .maybe_conflict(&addr, &map_conn_1);
                        if let Some(conflict) = route_conflict_1 {
                            ctx.route_conflicts.insert(conflict);
                        }
                        let route_conflict_2 = query
                            .criteria
                            .route_conflict_rule
                            .maybe_conflict(&new_addr, &map_conn_2);
                        if let Some(conflict) = route_conflict_2 {
                            ctx.route_conflicts.insert(conflict);
                        }

                        ctx.path.push(first_face);
                        ctx.path.push(second_face);
                        ctx.num_hexes += 1;

                        let new_conn = Connection::Face { face: new_face };
                        let new_conns_opt = new_tile.connections(&new_conn);
                        if let Some(new_conns) = new_conns_opt {
                            for new_conn in new_conns.iter() {
                                // NOTE: we can skip any Face connection here!
                                depth_first_search(
                                    map, query, ctx, paths, new_addr,
                                    *new_conn, new_tile,
                                );
                            }
                        }
                        // Pop the two face connections.
                        ctx.num_hexes -= 1;
                        ctx.path.pop();
                        ctx.path.pop();

                        // Remove traversed hex face conflicts, if any.
                        if let Some(conflict) = conflict_1 {
                            ctx.conflicts.remove(&conflict);
                        }
                        if let Some(conflict) = conflict_2 {
                            ctx.conflicts.remove(&conflict);
                        }
                        if let Some(conflict) = route_conflict_1 {
                            ctx.route_conflicts.remove(&conflict);
                        }
                        if let Some(conflict) = route_conflict_2 {
                            ctx.route_conflicts.remove(&conflict);
                        }
                    }
                }
                _ => {
                    depth_first_search(
                        map, query, ctx, paths, addr, *next_conn, tile,
                    );
                }
            }
        }
    }
}

fn depth_first_search(
    map: &Map,
    query: &Query,
    ctx: &mut Context,
    paths: &mut Vec<Path>,
    addr: HexAddress,
    conn: Connection,
    tile: &Tile,
) {
    if ctx
        .path
        .iter()
        .any(|step| step.addr == addr && step.conn.equivalent_to(&conn))
    {
        // NOTE: already visited this connection.
        return;
    }

    // Check if this connection conflicts with an earlier connection.
    let conflict = query.criteria.conflict_rule.maybe_conflict(&addr, &conn);
    if let Some(conflict) = conflict {
        if ctx.conflicts.contains(&conflict) {
            return;
        }
        ctx.conflicts.insert(conflict);
    }

    let route_conflict = query
        .criteria
        .route_conflict_rule
        .maybe_conflict(&addr, &conn);
    if let Some(conflict) = route_conflict {
        ctx.route_conflicts.insert(conflict);
    }

    // If we're at a city that contains a matching token, this means that the
    // starting location and this location can be reached in either direction.
    // To avoid exploring this connection multiple times, we can use the Ord
    // implementation for (HexAddress, usize) to ensure that we only explore
    // it in a single (and arbitrary, but consistent) direction.
    if let Connection::City { ix: city_ix } = conn {
        let token_tbl = map.hex_state(addr).unwrap().tokens();
        let has_token = token_tbl.iter().any(|(&space, &tok)| {
            space.city_ix() == city_ix && tok == query.criteria.token
        });
        if has_token {
            let start_ix = if let Connection::City { ix } = query.from {
                ix
            } else {
                panic!("Path starts at a dit?")
            };
            let start = (query.addr, start_ix);
            let here = (addr, city_ix);
            if start > here {
                return;
            }
        }
    }

    // Record this step and any conflict that it adds.
    let step = Step { addr, conn };
    ctx.path.push(step);

    // If this is a track connection, switch to the other end.
    let conn = if let Some(new_conn) = conn.other_end() {
        new_conn
    } else {
        conn
    };

    let conns = tile.connections(&conn);

    match conn {
        Connection::City { ix: city_ix } => {
            // Visit this city and save the current path.
            let city = tile.cities()[city_ix];
            let visit = Visit {
                addr,
                revenue: city.revenue,
                visits: StopLocation::City { ix: city_ix },
            };
            ctx.num_visits += 1;
            ctx.num_cities += 1;
            ctx.visits.push(visit);
            paths.push(ctx.current_path());
            // NOTE: if we can continue travelling past this city, then do so.
            // NOTE: trains cannot continue past an off-board city/town.
            let off_board = tile.colour == HexColour::Red
                || tile.colour == HexColour::Blue;
            if !off_board {
                let token_spaces = tile.city_token_spaces(city_ix);
                // NOTE: we must only check tokens associated with this city.
                let city_tokens: Vec<_> = map
                    .hex_state(addr)
                    .unwrap()
                    .tokens()
                    .iter()
                    .filter(|(space, _tok)| space.city_ix() == city_ix)
                    .collect();
                let can_continue = token_spaces.is_empty()
                    || (city_tokens.len() < token_spaces.len())
                    || city_tokens
                        .iter()
                        .any(|(_space, tok)| **tok == query.criteria.token);
                let more_visits_allowed =
                    ctx.can_continue(&query.criteria.path_limit);
                if can_continue && more_visits_allowed {
                    dfs_over(map, query, ctx, paths, addr, conns, tile);
                }
            }
            ctx.visits.pop();
            ctx.num_cities -= 1;
            ctx.num_visits -= 1;
        }
        Connection::Dit { ix: dit_ix } => {
            // Visit this dit and save the current path.
            let dit = tile.dits()[dit_ix];
            let visit = Visit {
                addr,
                revenue: dit.revenue,
                visits: StopLocation::Dit { ix: dit_ix },
            };
            ctx.num_visits += 1;
            ctx.num_dits += 1;
            ctx.visits.push(visit);
            paths.push(ctx.current_path());
            // NOTE: if we can continue travelling past this dit, then do so.
            let off_board = tile.colour == HexColour::Red
                || tile.colour == HexColour::Blue;
            let more_visits_allowed =
                ctx.can_continue(&query.criteria.path_limit);
            if !off_board && more_visits_allowed {
                dfs_over(map, query, ctx, paths, addr, conns, tile);
            }
            ctx.visits.pop();
            ctx.num_dits -= 1;
            ctx.num_visits -= 1;
        }
        _ => {
            // NOTE: no path to save, just visit subsequent connections.
            dfs_over(map, query, ctx, paths, addr, conns, tile);
        }
    }

    // Remove this step and any conflict that it adds.
    ctx.path.pop();
    if let Some(conflict) = conflict {
        ctx.conflicts.remove(&conflict);
    }
    if let Some(conflict) = route_conflict {
        ctx.route_conflicts.remove(&conflict);
    }
}

#[cfg(test)]
mod tests {
    use super::{Criteria, PathLimit, Query};
    use crate::conflict::ConflictRule;
    use n18hex::{Orientation, RotateCW};
    use n18map::{Descr, HexAddress, Map, TileDescr};
    use n18tile::Connection;
    use n18token::{Token, Tokens};

    /// Return a 2x2 map that contains the following tiles:
    ///
    /// - Tile 5 at (0, 0);
    /// - Tile 6 at (0, 1) (rotated clockwise twice);
    /// - Tile 58 at (1, 0) (rotated anti-clockwise once);
    /// - Tile 63 at (1, 1);
    ///
    /// "LP" tokens are placed on tiles 5 and 63; and "PO" tokens are placed
    /// on tiles 6 and 63.
    ///
    /// Note that this map may be used by test cases in other modules.
    pub fn map_2x2_tiles_5_6_58_63(tokens: Tokens) -> Map {
        let tiles = n18catalogue::tile_catalogue();
        let descr = descr_2x2_tiles_5_6_58_63();
        descr.build_map(tiles, tokens)
    }

    /// Define the tokens used in the following test cases.
    fn define_tokens() -> Tokens {
        use n18token::TokenStyle;

        vec![
            (
                "LP".to_string(),
                Token::new(TokenStyle::SideArcs {
                    fg: (63, 153, 153).into(),
                    bg: (255, 127, 127).into(),
                    text: (0, 0, 0).into(),
                }),
            ),
            (
                "PO".to_string(),
                Token::new(TokenStyle::SideArcs {
                    fg: (63, 153, 153).into(),
                    bg: (127, 255, 127).into(),
                    text: (0, 0, 0).into(),
                }),
            ),
        ]
        .into()
    }

    /// Defines the map that should be created by `map_2x2_tiles_5_6_58_63`.
    fn descr_2x2_tiles_5_6_58_63() -> Descr {
        (
            Orientation::FlatTop,
            vec![
                TileDescr {
                    row: 0,
                    col: 0,
                    tile: "5".to_string(),
                    rotation: RotateCW::Zero,
                    tokens: vec![(0, "LP".to_string())],
                },
                TileDescr {
                    row: 0,
                    col: 1,
                    tile: "6".to_string(),
                    rotation: RotateCW::Two,
                    tokens: vec![(0, "PO".to_string())],
                },
                TileDescr {
                    row: 1,
                    col: 0,
                    tile: "58".to_string(),
                    rotation: RotateCW::Five,
                    tokens: vec![],
                },
                TileDescr {
                    row: 1,
                    col: 1,
                    tile: "63".to_string(),
                    rotation: RotateCW::Zero,
                    tokens: vec![
                        (0, "PO".to_string()),
                        (1, "LP".to_string()),
                    ],
                },
            ],
        )
            .into()
    }

    /// Test that the maximum revenue obtained by paths of different lengths
    /// and either starting from, or passing through, a specific city are as
    /// expected.
    ///
    /// This uses a 2x2 map that contains the following tiles:
    ///
    /// - Tile 5 at (0, 0);
    /// - Tile 6 at (0, 1) (rotated clockwise twice);
    /// - Tile 58 at (1, 0) (rotated anti-clockwise once);
    /// - Tile 63 at (1, 1);
    ///
    /// "LP" tokens are placed on tiles 5 and 63; and "PO" tokens are placed
    /// on tiles 6 and 63.
    #[test]
    fn test_2x2_paths() {
        let tokens = define_tokens();
        let token_lp = *tokens.token("LP").unwrap();
        let map = map_2x2_tiles_5_6_58_63(tokens);

        let query = Query {
            addr: HexAddress::new(0, 0),
            from: Connection::City { ix: 0 },
            criteria: Criteria {
                token: token_lp,
                path_limit: Some(PathLimit::CitiesAndTowns { count: 2 }),
                conflict_rule: ConflictRule::TrackOrCityHex,
                route_conflict_rule: ConflictRule::TrackOnly,
            },
        };
        let from_len2 = super::paths_from(&map, &query);
        let via_len2 = super::paths_through(&map, &query);
        let rev_from_len2 = from_len2.iter().map(|path| path.revenue).max();
        let rev_via_len2 = via_len2.iter().map(|path| path.revenue).max();
        assert_eq!(rev_from_len2, Some(40));
        assert_eq!(rev_via_len2, Some(40));

        let query = Query {
            addr: HexAddress::new(0, 0),
            from: Connection::City { ix: 0 },
            criteria: Criteria {
                token: token_lp,
                path_limit: Some(PathLimit::CitiesAndTowns { count: 3 }),
                conflict_rule: ConflictRule::TrackOrCityHex,
                route_conflict_rule: ConflictRule::TrackOnly,
            },
        };
        let from_len3 = super::paths_from(&map, &query);
        let via_len3 = super::paths_through(&map, &query);
        let rev_from_len3 = from_len3.iter().map(|path| path.revenue).max();
        let rev_via_len3 = via_len3.iter().map(|path| path.revenue).max();
        assert_eq!(rev_from_len3, Some(70));
        assert_eq!(rev_via_len3, Some(70));

        let query = Query {
            addr: HexAddress::new(0, 0),
            from: Connection::City { ix: 0 },
            criteria: Criteria {
                token: token_lp,
                path_limit: Some(PathLimit::CitiesAndTowns { count: 4 }),
                conflict_rule: ConflictRule::TrackOrCityHex,
                route_conflict_rule: ConflictRule::TrackOnly,
            },
        };
        let from_len4 = super::paths_from(&map, &query);
        let via_len4 = super::paths_through(&map, &query);
        let rev_from_len4 = from_len4.iter().map(|path| path.revenue).max();
        let rev_via_len4 = via_len4.iter().map(|path| path.revenue).max();
        assert_eq!(rev_from_len4, Some(90));
        assert_eq!(rev_via_len4, Some(90));

        let query = Query {
            addr: HexAddress::new(0, 0),
            from: Connection::City { ix: 0 },
            criteria: Criteria {
                token: token_lp,
                path_limit: None,
                conflict_rule: ConflictRule::TrackOrCityHex,
                route_conflict_rule: ConflictRule::TrackOnly,
            },
        };
        let from_any = super::paths_from(&map, &query);
        let via_any = super::paths_through(&map, &query);
        let rev_from_any = from_any.iter().map(|path| path.revenue).max();
        let rev_via_any = via_any.iter().map(|path| path.revenue).max();
        assert_eq!(rev_from_any, Some(90));
        assert_eq!(rev_via_any, Some(90));
    }
}
