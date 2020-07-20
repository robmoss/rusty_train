//! Paths and routes may not share certain features.

use n18hex::HexFace;
use n18map::HexAddress;
use n18tile::Connection;

/// A rule defines which elements of a path or route may not be shared.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConflictRule {
    /// No track segment (including hex faces) in common.
    TrackOnly,
    /// No track segment or city (including dits) in common.
    TrackOrCity,
    /// No track segment, or any city (including dits) on the same hex.
    TrackOrCityHex,
    /// No hexes in common.
    Hex,
}

/// A specific element of a path or route that cannot be shared.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Conflict {
    /// A face on a specific hex.
    Face { addr: HexAddress, face: HexFace },
    /// A specific track segment on a specific hex.
    Track { addr: HexAddress, ix: usize },
    /// A specific city on a specific hex.
    City { addr: HexAddress, ix: usize },
    /// A specific dit on a specific hex.
    Dit { addr: HexAddress, ix: usize },
    /// Any city on a specific hex.
    CityHex { addr: HexAddress },
    /// A specific hex.
    Hex { addr: HexAddress },
}

impl ConflictRule {
    /// Returns the conflict that this connection adds to a path or route.
    pub fn maybe_conflict(
        &self,
        addr: &HexAddress,
        conn: &Connection,
    ) -> Option<Conflict> {
        use ConflictRule::*;
        use Connection::*;

        // NOTE: not trivial, need to return the most general conflict.
        match conn {
            Track { ix, end: _ } => match self {
                Hex => Some(Conflict::Hex { addr: *addr }),
                _ => Some(Conflict::Track {
                    addr: *addr,
                    ix: *ix,
                }),
            },
            Face { face } => match self {
                Hex => Some(Conflict::Hex { addr: *addr }),
                _ => Some(Conflict::Face {
                    addr: *addr,
                    face: *face,
                }),
            },
            Dit { ix } => match self {
                Hex => Some(Conflict::Hex { addr: *addr }),
                TrackOrCityHex => Some(Conflict::CityHex { addr: *addr }),
                TrackOrCity => Some(Conflict::Dit {
                    addr: *addr,
                    ix: *ix,
                }),
                TrackOnly => None,
            },
            City { ix } => match self {
                Hex => Some(Conflict::Hex { addr: *addr }),
                TrackOrCityHex => Some(Conflict::CityHex { addr: *addr }),
                TrackOrCity => Some(Conflict::City {
                    addr: *addr,
                    ix: *ix,
                }),
                TrackOnly => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConflictRule;

    #[test]
    /// Check that the conflict rules have the desired ordering.
    /// This is necessary for ensuring that route-combining constraints are
    /// **at least as flexible** as the route-building constraints.
    /// That is, `route_combining_rule <= route_building_rule`.
    fn rule_ordering_1() {
        use ConflictRule::*;

        assert!(TrackOnly < TrackOrCity);
        assert!(TrackOnly < TrackOrCityHex);
        assert!(TrackOnly < Hex);
        assert!(TrackOrCity < TrackOrCityHex);
        assert!(TrackOrCity < Hex);
        assert!(TrackOrCityHex < Hex);
    }
}
