//! Serialise and deserialise train routes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq)]
struct HexAddress {
    addr: n18map::HexAddress,
}

impl std::convert::From<n18map::HexAddress> for HexAddress {
    fn from(src: n18map::HexAddress) -> Self {
        Self { addr: src }
    }
}

impl std::convert::From<HexAddress> for n18map::HexAddress {
    fn from(src: HexAddress) -> Self {
        src.addr
    }
}

// See https://github.com/serde-rs/serde/issues/1316
impl<'de> Deserialize<'de> for HexAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let addr: n18map::HexAddress = String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)?;
        Ok(Self { addr })
    }
}

// See https://github.com/serde-rs/serde/issues/1316
impl Serialize for HexAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.addr)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
enum StopLocation {
    City { ix: usize },
    Dit { ix: usize },
}

impl std::convert::From<n18route::StopLocation> for StopLocation {
    fn from(src: n18route::StopLocation) -> Self {
        use n18route::StopLocation::*;

        match src {
            City { ix } => StopLocation::City { ix },
            Dit { ix } => StopLocation::Dit { ix },
        }
    }
}

impl std::convert::From<StopLocation> for n18route::StopLocation {
    fn from(src: StopLocation) -> Self {
        use n18route::StopLocation::*;

        match src {
            StopLocation::City { ix } => City { ix },
            StopLocation::Dit { ix } => Dit { ix },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
struct Visit {
    addr: HexAddress,
    revenue: usize,
    visits: StopLocation,
}

impl std::convert::From<n18route::Visit> for Visit {
    fn from(src: n18route::Visit) -> Self {
        Self {
            addr: src.addr.into(),
            revenue: src.revenue,
            visits: src.visits.into(),
        }
    }
}

impl std::convert::From<Visit> for n18route::Visit {
    fn from(src: Visit) -> Self {
        Self {
            addr: src.addr.into(),
            revenue: src.revenue,
            visits: src.visits.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
enum Connection {
    Track { ix: usize, end: super::TrackEnd },
    Dit { ix: usize },
    City { ix: usize },
    Face { face: super::HexFace },
}

impl std::convert::From<n18tile::Connection> for Connection {
    fn from(src: n18tile::Connection) -> Self {
        use n18tile::Connection::*;

        match src {
            Track { ix, end } => Connection::Track {
                ix,
                end: end.into(),
            },
            Dit { ix } => Connection::Dit { ix },
            City { ix } => Connection::City { ix },
            Face { face } => Connection::Face { face: face.into() },
        }
    }
}

impl std::convert::From<Connection> for n18tile::Connection {
    fn from(src: Connection) -> Self {
        use n18tile::Connection::*;

        match src {
            Connection::Track { ix, end } => Track {
                ix,
                end: end.into(),
            },
            Connection::Dit { ix } => Dit { ix },
            Connection::City { ix } => City { ix },
            Connection::Face { face } => Face {
                face: (&face).into(),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
struct Step {
    addr: HexAddress,
    conn: Connection,
}

impl std::convert::From<n18route::Step> for Step {
    fn from(src: n18route::Step) -> Self {
        Self {
            addr: src.addr.into(),
            conn: src.conn.into(),
        }
    }
}

impl std::convert::From<Step> for n18route::Step {
    fn from(src: Step) -> Self {
        Self {
            addr: src.addr.into(),
            conn: src.conn.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Route {
    steps: Vec<Step>,
    visits: Vec<Visit>,
}

impl std::convert::From<n18route::Route> for Route {
    fn from(src: n18route::Route) -> Self {
        Self {
            steps: src.steps.into_iter().map(|x| x.into()).collect(),
            visits: src.visits.into_iter().map(|x| x.into()).collect(),
        }
    }
}

impl std::convert::From<Route> for n18route::Route {
    fn from(src: Route) -> Self {
        Self {
            steps: src.steps.into_iter().map(|x| x.into()).collect(),
            visits: src.visits.into_iter().map(|x| x.into()).collect(),
        }
    }
}

impl std::convert::From<&n18route::Route> for Route {
    fn from(src: &n18route::Route) -> Self {
        Self {
            steps: src.steps.iter().map(|&x| x.into()).collect(),
            visits: src.visits.iter().map(|&x| x.into()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
enum TrainType {
    MustStop,
    SkipTowns,
    SkipAny,
}

impl std::convert::From<n18route::TrainType> for TrainType {
    fn from(src: n18route::TrainType) -> Self {
        use n18route::TrainType::*;

        match src {
            MustStop => TrainType::MustStop,
            SkipTowns => TrainType::SkipTowns,
            SkipAny => TrainType::SkipAny,
        }
    }
}

impl std::convert::From<TrainType> for n18route::TrainType {
    fn from(src: TrainType) -> Self {
        use n18route::TrainType::*;

        match src {
            TrainType::MustStop => MustStop,
            TrainType::SkipTowns => SkipTowns,
            TrainType::SkipAny => SkipAny,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
struct Train {
    train_type: TrainType,
    max_stops: Option<usize>,
    revenue_multiplier: usize,
}

impl std::convert::From<n18route::Train> for Train {
    fn from(src: n18route::Train) -> Self {
        Self {
            train_type: src.train_type.into(),
            max_stops: src.max_stops,
            revenue_multiplier: src.revenue_multiplier,
        }
    }
}

impl std::convert::From<Train> for n18route::Train {
    fn from(src: Train) -> Self {
        Self {
            train_type: src.train_type.into(),
            max_stops: src.max_stops,
            revenue_multiplier: src.revenue_multiplier,
        }
    }
}

impl std::convert::From<&n18route::Train> for Train {
    fn from(src: &n18route::Train) -> Self {
        Self {
            train_type: src.train_type.into(),
            max_stops: src.max_stops,
            revenue_multiplier: src.revenue_multiplier,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct TrainRoute {
    train: Train,
    revenue: usize,
    route: Route,
}

impl std::convert::From<n18route::TrainRoute> for TrainRoute {
    fn from(src: n18route::TrainRoute) -> Self {
        Self {
            train: src.train.into(),
            revenue: src.revenue,
            route: src.route.into(),
        }
    }
}

impl std::convert::From<TrainRoute> for n18route::TrainRoute {
    fn from(src: TrainRoute) -> Self {
        Self {
            train: src.train.into(),
            revenue: src.revenue,
            route: src.route.into(),
        }
    }
}

impl std::convert::From<&n18route::TrainRoute> for TrainRoute {
    fn from(src: &n18route::TrainRoute) -> Self {
        Self {
            train: src.train.into(),
            revenue: src.revenue,
            route: (&src.route).into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(super) struct Routes {
    net_revenue: usize,
    train_routes: Vec<TrainRoute>,
}

impl std::convert::From<n18route::Routes> for Routes {
    fn from(src: n18route::Routes) -> Self {
        Self {
            net_revenue: src.net_revenue,
            train_routes: src
                .train_routes
                .into_iter()
                .map(|x| x.into())
                .collect(),
        }
    }
}

impl std::convert::From<Routes> for n18route::Routes {
    fn from(src: Routes) -> Self {
        Self {
            net_revenue: src.net_revenue,
            train_routes: src
                .train_routes
                .into_iter()
                .map(|x| x.into())
                .collect(),
        }
    }
}

impl std::convert::From<&n18route::Routes> for Routes {
    fn from(src: &n18route::Routes) -> Self {
        Self {
            net_revenue: src.net_revenue,
            train_routes: src.train_routes.iter().map(|x| x.into()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;

    use super::*;

    static OUT_DIR: &'static str = "../../tests/output";

    fn output_path(file: &'static str) -> std::path::PathBuf {
        std::path::Path::new(OUT_DIR).join(file)
    }

    #[test]
    fn json_hex_address_round_trip() {
        let filename = output_path("test-hex_address_round_trip.json");
        let addr: n18map::HexAddress = "A5".parse().unwrap();
        let de_in = HexAddress { addr };
        let file = File::create(&filename).unwrap();
        serde_json::to_writer(file, &de_in).unwrap();
        let file = File::open(&filename).unwrap();
        let reader = BufReader::new(file);
        let de_out: HexAddress = serde_json::from_reader(reader).unwrap();
        assert_eq!(de_in.addr, de_out.addr);
    }

    #[test]
    fn json_visit_round_trip() {
        let filename = output_path("test-visit_round_trip.json");
        let addr: n18map::HexAddress = "A5".parse().unwrap();
        let visit_in = n18route::Visit {
            addr,
            revenue: 50,
            visits: n18route::StopLocation::City { ix: 2 },
        };
        let de_in: Visit = visit_in.into();
        let file = File::create(&filename).unwrap();
        serde_json::to_writer(file, &de_in).unwrap();
        let file = File::open(&filename).unwrap();
        let reader = BufReader::new(file);
        let de_out: Visit = serde_json::from_reader(reader).unwrap();
        assert_eq!(de_in, de_out);
        let visit_out: n18route::Visit = de_out.into();
        assert_eq!(visit_in, visit_out);
    }
}
