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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

// pub type RouteConflicts = rc_hash::RouteConflicts;
pub type RouteConflicts = rc_vec::RouteConflicts;

pub mod rc_hash {

    use super::Conflict;
    use std::collections::HashSet;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct RouteConflicts {
        rc: HashSet<Conflict>,
    }

    impl RouteConflicts {
        pub fn new() -> Self {
            RouteConflicts { rc: HashSet::new() }
        }

        pub fn is_disjoint(&self, other: &Self) -> bool {
            self.rc.is_disjoint(&other.rc)
        }

        pub fn merge(&self, other: &Self) -> Self {
            RouteConflicts {
                rc: self.rc.union(&other.rc).map(|c| *c).collect(),
            }
        }

        pub fn len(&self) -> usize {
            self.rc.len()
        }

        pub fn iter(&self) -> impl Iterator<Item = &Conflict> {
            self.rc.iter()
        }
    }

    impl From<HashSet<Conflict>> for RouteConflicts {
        fn from(set: HashSet<Conflict>) -> Self {
            RouteConflicts { rc: set }
        }
    }

    impl From<&HashSet<Conflict>> for RouteConflicts {
        fn from(set: &HashSet<Conflict>) -> Self {
            let rc = set.iter().map(|c| *c).collect();
            RouteConflicts { rc }
        }
    }
}

pub mod rc_vec {

    use super::Conflict;
    use std::collections::HashSet;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct RouteConflicts {
        rc: Vec<Conflict>,
    }

    impl RouteConflicts {
        pub fn new() -> Self {
            RouteConflicts { rc: Vec::new() }
        }

        pub fn is_disjoint(&self, other: &Self) -> bool {
            let a = &self.rc;
            let b = &other.rc;
            let la = a.len();
            let lb = b.len();
            let mut ixa = 0;
            let mut ixb = 0;
            loop {
                if ixa >= la || ixb >= lb {
                    return true;
                }
                if a[ixa] < b[ixb] {
                    ixa += 1;
                } else if a[ixa] > b[ixb] {
                    ixb += 1;
                } else {
                    return false;
                }
            }
        }

        // NOTE: alternate implementation, which requires itertools.
        // pub fn merge(&self, other: &Self) -> Self {
        //     let conflict_set: std::collections::HashSet<Conflict> =
        //         self.rc.iter().merge(other.rc.iter()).map(|c| *c).collect();
        //     let mut result: Vec<Conflict> =
        //         conflict_set.into_iter().collect();
        //     result.sort();
        //     return Self { rc: result };
        // }

        pub fn merge(&self, other: &Self) -> Self {
            // NOTE: both slices are already sorted.
            let a = &self.rc;
            let b = &other.rc;
            let la = a.len();
            let lb = b.len();
            let mut rc: Vec<Conflict> = Vec::with_capacity(la + lb);
            let mut ixa = 0;
            let mut ixb = 0;
            loop {
                if ixa >= la {
                    while ixb < lb {
                        rc.push(b[ixb]);
                        ixb += 1;
                    }
                    break;
                } else if ixb >= lb {
                    while ixa < la {
                        rc.push(a[ixa]);
                        ixa += 1;
                    }
                    break;
                }
                if a[ixa] < b[ixb] {
                    rc.push(a[ixa]);
                    ixa += 1;
                } else if a[ixa] > b[ixb] {
                    rc.push(b[ixb]);
                    ixb += 1;
                } else {
                    // We know that a[ixa] == b[ixb].
                    rc.push(a[ixa]);
                    ixa += 1;
                    ixb += 1;
                }
            }
            RouteConflicts { rc }
        }

        pub fn len(&self) -> usize {
            self.rc.len()
        }

        pub fn iter(&self) -> impl Iterator<Item = &Conflict> {
            self.rc.iter()
        }
    }

    impl From<HashSet<Conflict>> for RouteConflicts {
        fn from(set: HashSet<Conflict>) -> Self {
            let mut rc: Vec<_> = set.iter().map(|c| *c).collect();
            rc.sort();
            RouteConflicts { rc }
        }
    }

    impl From<&HashSet<Conflict>> for RouteConflicts {
        fn from(set: &HashSet<Conflict>) -> Self {
            let mut rc: Vec<_> = set.iter().map(|c| *c).collect();
            rc.sort();
            RouteConflicts { rc }
        }
    }
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
