//! Search a map for paths from a starting location.

use std::collections::HashSet;

use super::conflict::{Conflict, ConflictRule};
use super::{Path, Step, Stop, StopLocation};
use crate::connection::Connection;
use crate::map::{HexAddress, Map, Token};
use crate::tile::Tile;

/// The search criteria for identifying valid paths.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Query {
    pub addr: HexAddress,
    pub from: Connection,
    pub token: Token,
    pub max_visits: Option<usize>,
    pub skip_cities: bool,
    pub skip_dits: bool,
    pub conflict_rule: ConflictRule,
}

/// The current state of the path-exploration algorithm.
struct Context {
    /// The previous steps in this path.
    path: Vec<Step>,
    /// The existing path elements that may not be re-used.
    conflicts: HashSet<Conflict>,
    /// The number of cities that have already been skipped.
    skipped_cities: usize,
    /// The number of dits that have already been skipped.
    skipped_dits: usize,
    /// The cities and dits that have been visited, possibly for revenue.
    stops: Vec<Stop>,
    /// The number cities and dits that have been visited for revenue.
    num_visits: usize,
}

impl Context {
    fn new(map: &Map, query: &Query) -> Self {
        let path: Vec<Step> = vec![Step {
            addr: query.addr,
            conn: query.from,
        }];
        let mut conflicts = HashSet::new();
        if let Some(conflict) =
            query.conflict_rule.maybe_conflict(&query.addr, &query.from)
        {
            conflicts.insert(conflict);
        }

        // NOTE: record the starting city/dit and its revenue.
        let tile = map.tile_at(query.addr).unwrap();
        let first_stop = match query.from {
            Connection::City { ix: city_ix } => {
                let city = tile.cities()[city_ix];
                Stop {
                    addr: query.addr,
                    revenue: Some(city.revenue),
                    stop_at: StopLocation::City { ix: city_ix },
                }
            }
            Connection::Dit { ix: dit_ix } => {
                let dit = tile.dits()[dit_ix];
                Stop {
                    addr: query.addr,
                    revenue: Some(dit.revenue),
                    stop_at: StopLocation::Dit { ix: dit_ix },
                }
            }
            _ => panic!("Invalid starting connection"),
        };

        Context {
            path,
            conflicts,
            stops: vec![first_stop],
            skipped_cities: 0,
            skipped_dits: 0,
            num_visits: 1,
        }
    }

    fn get_current_path(&self) -> Path {
        Path {
            steps: self.path.clone(),
            conflicts: self.conflicts.clone(),
            stops: self.stops.clone(),
            revenue: self.stops.iter().filter_map(|stop| stop.revenue).sum(),
        }
    }
}

/// Returns all valid paths that match the provided criteria.
pub fn paths_from(map: &Map, query: &Query) -> Vec<Path> {
    let mut context = Context::new(map, query);
    let mut paths: Vec<Path> = vec![];
    let start_tile = map.tile_at(query.addr).unwrap();
    let connections = start_tile.connections(&query.from).unwrap();
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
                            addr: addr,
                            conn: *next_conn,
                        };
                        let second_face = Step {
                            addr: new_addr,
                            conn: Connection::Face { face: new_face },
                        };
                        ctx.path.push(first_face);
                        ctx.path.push(second_face);

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
                        ctx.path.pop();
                        ctx.path.pop();
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
    let conflict = query.conflict_rule.maybe_conflict(&addr, &conn);
    if let Some(conflict) = conflict {
        if ctx.conflicts.contains(&conflict) {
            return;
        }
        ctx.conflicts.insert(conflict);
    }

    // Record this step and any conflict that it adds.
    let step = Step {
        addr: addr,
        conn: conn,
    };
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
            let stop = Stop {
                addr,
                revenue: Some(city.revenue),
                stop_at: StopLocation::City { ix: city_ix },
            };
            ctx.num_visits += 1;
            ctx.stops.push(stop);
            paths.push(ctx.get_current_path());
            // NOTE: if we can continue travelling past this city, then do so.
            let token_spaces = tile.city_token_spaces(city_ix);
            let token_tbl = map.get_hex(addr).unwrap().get_tokens();
            let can_continue = token_spaces.len() == 0
                || (token_tbl.len() < token_spaces.len())
                || token_tbl.values().any(|&tok| tok == query.token);
            let more_visits_allowed = query
                .max_visits
                .map(|max| max > ctx.num_visits)
                .unwrap_or(true);
            if can_continue && more_visits_allowed {
                dfs_over(map, query, ctx, paths, addr, conns, tile);
            }
            ctx.stops.pop();
            ctx.num_visits -= 1;

            // NOTE: also record paths that skip over this city, if allowed.
            // NOTE: a city may be a central dit.
            let can_skip = if token_spaces.len() == 0 {
                query.skip_dits
            } else {
                query.skip_cities
            };
            if can_skip && can_continue {
                let stop = Stop {
                    addr,
                    revenue: None,
                    stop_at: StopLocation::City { ix: city_ix },
                };
                ctx.stops.push(stop);
                if token_spaces.len() == 0 {
                    ctx.skipped_dits += 1;
                } else {
                    ctx.skipped_cities += 1;
                }
                dfs_over(map, query, ctx, paths, addr, conns, tile);
                if token_spaces.len() == 0 {
                    ctx.skipped_dits -= 1;
                } else {
                    ctx.skipped_cities -= 1;
                }
                ctx.stops.pop();
            }
        }
        Connection::Dit { ix: dit_ix } => {
            // Visit this dit and save the current path.
            let dit = tile.dits()[dit_ix];
            let stop = Stop {
                addr,
                revenue: Some(dit.revenue),
                stop_at: StopLocation::Dit { ix: dit_ix },
            };
            ctx.num_visits += 1;
            ctx.stops.push(stop);
            paths.push(ctx.get_current_path());
            // NOTE: if we can continue travelling past this dit, then do so.
            let more_visits_allowed = query
                .max_visits
                .map(|max| max > ctx.num_visits)
                .unwrap_or(true);
            if more_visits_allowed {
                dfs_over(map, query, ctx, paths, addr, conns, tile);
            }
            ctx.stops.pop();
            ctx.num_visits -= 1;

            // NOTE: also record paths that skip over this dit, if allowed.
            if query.skip_dits {
                let stop = Stop {
                    addr,
                    revenue: None,
                    stop_at: StopLocation::Dit { ix: dit_ix },
                };
                ctx.stops.push(stop);
                ctx.skipped_dits += 1;
                dfs_over(map, query, ctx, paths, addr, conns, tile);
                ctx.skipped_dits -= 1;
                ctx.stops.pop();
            }
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
}
