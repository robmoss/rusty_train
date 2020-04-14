use crate::city::City;
use crate::hex::{Hex, HexFace};
use crate::track::{Track, TrackEnd};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Connection {
    Track { ix: usize, end: TrackEnd },
    Dit { ix: usize },
    City { ix: usize },
    Face { face: HexFace },
}

impl Connection {
    /// Returns whether this connection is equivalent to another connection.
    ///
    /// This is less restrictive than equality as defined by `std::cmp::Eq`,
    /// because connections to either end of the same track segment are
    /// considered equivalent **but are not equal to each other**.
    pub fn equivalent_to(&self, other: &Self) -> bool {
        use Connection::*;

        match (self, other) {
            // NOTE: track connections are equivalent regardless of direction.
            (Track { ix: a, .. }, Track { ix: b, .. }) => a == b,
            (Dit { ix: a }, Dit { ix: b }) => a == b,
            (City { ix: a }, City { ix: b }) => a == b,
            (Face { face: a }, Face { face: b }) => a == b,
            _ => false,
        }
    }

    /// Returns the connection at the other end of a track segment, or `None`
    /// if the provided connection is not a track segment.
    pub fn other_end(&self) -> Option<Self> {
        use Connection::*;

        match self {
            Track { ix, end } => Some(Track {
                ix: *ix,
                end: end.other_end(),
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Dit {
    pub track_ix: usize,
    end: TrackEnd,
    pub revenue: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connections {
    // NOTE: dits are drawn by the track segment that "owns" them, but for
    // connectivity purposes they are separate entities and so they should
    // also be stored separately; here they are stored by index (0..N).
    dits: Vec<Dit>,
    track: HashMap<(usize, TrackEnd), Vec<Connection>>,
    face: HashMap<HexFace, Vec<Connection>>,
    city: HashMap<usize, Vec<Connection>>,
    dit: HashMap<usize, Vec<Connection>>,
    none: Vec<Connection>,
}

impl Connections {
    // - Track ends may connect to a hex face, a dit, or a city.
    // - If they cross another track, the tracks are not connected.
    // - If they cross a dit or a city, this is an ERROR.
    //
    // Tracks with dits - the dit must be at one end or the other, and we
    // can calculate the dit angle so that we need never look it up again.
    // Then it really becomes a separate entity from the track.
    //
    // The (de)serialisation layer could possibly break apart a single
    // track as defined in the existing setup (which may span a city or
    // dit) into two track segments, and maybe piece these two back
    // together when serialising? Could be really messy and inconsistent
    // though. But the round-tripping would be nice!

    pub fn new(tracks: &Vec<Track>, cities: &Vec<City>, hex: &Hex) -> Self {
        let mut dits = vec![];
        let mut track_conns = HashMap::new();
        let mut face_conns = HashMap::new();
        let mut dit_conns = HashMap::new();
        let mut city_conns = HashMap::new();

        let ctx = hex.context();

        for i in 0..tracks.len() {
            let track = tracks[i];

            // Record connections between this track and hex faces.
            for (end, face) in track.connected_to_faces() {
                face_conns
                    .entry(face)
                    .or_insert(vec![])
                    .push(Connection::Track { ix: i, end });
                track_conns
                    .entry((i, end))
                    .or_insert(vec![])
                    .push(Connection::Face { face });
            }

            if let Some((dit_end, revenue, _shape)) = track.dit {
                // Record the connection between this track and the dit at one
                // of its end.
                let dit_ix = dits.len();
                dits.push(Dit {
                    track_ix: i,
                    end: dit_end,
                    revenue,
                });
                dit_conns.entry(dit_ix).or_insert(vec![]).push(
                    Connection::Track {
                        ix: i,
                        end: dit_end,
                    },
                );
                track_conns
                    .entry((i, dit_end))
                    .or_insert(vec![])
                    .push(Connection::Dit { ix: dit_ix });

                // NOTE: Also connect this dit to any track segments that are
                // connected to this end of the track.
                for j in 0..tracks.len() {
                    if j == i {
                        continue;
                    }
                    let other = tracks[j];
                    let conn_opt = track.connected_at(&other, hex, ctx);
                    if let Some((conn_end, other_end)) = conn_opt {
                        if conn_end == dit_end {
                            dit_conns.entry(dit_ix).or_insert(vec![]).push(
                                Connection::Track {
                                    ix: j,
                                    end: other_end,
                                },
                            );
                            track_conns
                                .entry((j, other_end))
                                .or_insert(vec![])
                                .push(Connection::Dit { ix: dit_ix });
                        }
                    }
                }
            }
        }

        for (cx, city) in cities.iter().enumerate() {
            for (i, track) in tracks.iter().enumerate() {
                let end_opt = track.connected_to_fill_at(city, hex, ctx);
                if let Some(end) = end_opt {
                    city_conns
                        .entry(cx)
                        .or_insert(vec![])
                        .push(Connection::Track { ix: i, end });
                    track_conns
                        .entry((i, end))
                        .or_insert(vec![])
                        .push(Connection::City { ix: cx });
                }
            }
        }

        Connections {
            dits,
            track: track_conns,
            face: face_conns,
            city: city_conns,
            dit: dit_conns,
            none: vec![],
        }
    }

    pub fn dits(&self) -> &[Dit] {
        &self.dits
    }

    pub fn from(&self, from: &Connection) -> Option<&[Connection]> {
        use Connection::*;

        let conns_opt = match from {
            Track { ix, end } => {
                let key = (*ix, *end);
                self.track.get(&key)
            }
            Dit { ix } => self.dit.get(ix),
            City { ix } => self.city.get(ix),
            Face { face } => self.face.get(face),
        };

        conns_opt.map(|cs| cs.as_slice())
    }
}

#[cfg(test)]
/// Tests that check whether connections are defined appropriately.
mod tests {
    use crate::connection::Connection;
    use crate::hex::{Hex, HexFace};

    static HEX_DIAMETER: f64 = 150.0;

    #[test]
    /// Tile 5 contains a city with a single token space, and track segments
    /// that run from this city to the bottom and lower-right hex faces.
    fn test_single_tile_5() {
        let tile_name = "5";
        let hex = Hex::new(HEX_DIAMETER);
        let tiles = crate::catalogue::tile_catalogue(&hex);
        let tile = tiles.iter().find(|t| t.name == tile_name);
        assert!(tile.is_some());
        let tile = tile.unwrap();
        let city_ix = 0;

        // Find connections from this tile's only city.
        let from = Connection::City { ix: city_ix };
        let conns = tile.connections(&from);
        assert!(conns.is_some());
        let conns = conns.unwrap();
        assert_eq!(conns.len(), 2);

        // Ensure that each connection is a Track segment.
        for conn in conns {
            match conn {
                Connection::Track { ix, end: _ } => {
                    assert!(*ix == 0 || *ix == 1);

                    // Ensure the other end of the track connects to a face.
                    let other_end = conn.other_end();
                    assert!(other_end.is_some());
                    let conn = other_end.unwrap();
                    let track_conns = tile.connections(&conn);
                    assert!(track_conns.is_some());
                    let track_conns = track_conns.unwrap();
                    assert_eq!(track_conns.len(), 1);
                    let track_conn = track_conns[0];
                    if let Connection::Face { face } = track_conn {
                        if face != HexFace::Bottom
                            && face != HexFace::LowerRight
                        {
                            panic!(
                                "Unexpected 2nd connection {:?}",
                                track_conn
                            );
                        }
                    } else {
                        panic!("Unexpected 2nd connection {:?}", track_conn);
                    }
                }
                _ => {
                    panic!("Unexpected connection: {:?} is not a track", conn)
                }
            }
        }
    }
}
