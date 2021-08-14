use crate::Kind;
use n18hex::{Hex, HexFace, HexFace::*};

static FACES: [HexFace; 6] =
    [Bottom, LowerLeft, UpperLeft, Top, UpperRight, LowerRight];

#[test]
/// Check face-to-face connections for tile 43.
fn tile_43() {
    let hex = Hex::default();

    let tile = Kind::_43.build(&hex);
    assert!(tile.connected_faces_are(Bottom, &[UpperLeft, Top]));
    assert!(tile.connected_faces_are(LowerLeft, &[UpperLeft, Top]));
    assert!(tile.connected_faces_are(UpperLeft, &[LowerLeft, Bottom]));
    assert!(tile.connected_faces_are(Top, &[LowerLeft, Bottom]));
    assert!(tile.no_connected_faces(UpperRight));
    assert!(tile.no_connected_faces(LowerRight));
}

#[test]
/// Check face-to-dit connections for tile 4.
fn tile_4() {
    let hex = Hex::default();

    let tile = Kind::_4.build(&hex);
    assert_eq!(tile.connected_dits_are(Bottom, &[10]), Some(vec![0]));
    assert_eq!(tile.connected_dits_are(Bottom, &[20]), None);
    assert_eq!(tile.connected_dits_are(Top, &[10]), Some(vec![0]));
    assert_eq!(tile.connected_dits_are(Top, &[20]), None);
    for face in FACES {
        if face == Top || face == Bottom {
            continue;
        }
        assert_eq!(tile.connected_dits_are(face, &[10]), None);
        assert_eq!(tile.connected_dits_are(face, &[20]), None);
    }
}

#[test]
/// Check face-to-dit connections for tile IN10.
fn tile_in10() {
    let hex = Hex::default();

    let tile = Kind::IN10.build(&hex);
    for face in FACES {
        if face == Bottom {
            let ixs = Some(vec![0, 1]);
            assert_eq!(tile.connected_dits_are(face, &[30, 30]), ixs);
            assert_eq!(tile.connected_dits_are(face, &[30]), None);
        } else if face == UpperLeft || face == LowerRight {
            let ixs = Some(vec![0]);
            assert_eq!(tile.connected_dits_are(face, &[30]), ixs);
            assert_eq!(tile.connected_dits_are(face, &[30, 30]), None);
        } else if face == UpperRight {
            let ixs = Some(vec![1]);
            assert_eq!(tile.connected_dits_are(face, &[30]), ixs);
            assert_eq!(tile.connected_dits_are(face, &[30, 30]), None);
        } else {
            assert_eq!(tile.connected_dits_are(face, &[30]), None);
            assert_eq!(tile.connected_dits_are(face, &[30, 30]), None);
        }
    }
}

#[test]
/// Check face-to-city connections for tile 5.
fn tile_5() {
    let hex = Hex::default();

    let tile = Kind::_5.build(&hex);
    for face in FACES {
        if face == Bottom || face == LowerRight {
            // Check that these faces are connected to a single-token city
            // that is worth $20.
            assert_eq!(tile.connected_cities_are(face, &[]), None);
            assert_eq!(
                tile.connected_cities_are(face, &[(20, 1)]),
                Some(vec![0])
            );
        } else {
            // Check that all other faces are not connected to the city.
            assert_eq!(tile.connected_cities_are(face, &[]), Some(vec![]));
            assert_eq!(tile.connected_cities_are(face, &[(20, 1)]), None);
        }
    }
}

#[test]
/// Check face-to-city connections for tile 122.
fn tile_122() {
    let hex = Hex::default();

    let tile = Kind::_122.build(&hex);
    // Identify the city that is connected to the lower-left face.
    let city_ll_ix = {
        let res = tile.connected_cities_are(LowerLeft, &[(80, 2)]);
        assert!(res.is_some());
        let ixs = res.unwrap();
        assert!(ixs.len() == 1);
        ixs[0]
    };
    assert_eq!(
        tile.connected_cities_are(UpperLeft, &[(80, 2)]),
        Some(vec![city_ll_ix])
    );
    // Identify the city that is connected to the upper-right face.
    let city_ur_ix = {
        let res = tile.connected_cities_are(UpperRight, &[(80, 2)]);
        assert!(res.is_some());
        let ixs = res.unwrap();
        assert!(ixs.len() == 1);
        ixs[0]
    };
    assert_eq!(
        tile.connected_cities_are(Top, &[(80, 2)]),
        Some(vec![city_ur_ix])
    );
    // Ensure the two cities are different.
    assert_ne!(city_ll_ix, city_ur_ix);
    // Check that the remaining two faces have no city connections.
    assert_eq!(tile.connected_cities_are(Bottom, &[(80, 2)]), None);
    assert_eq!(tile.connected_cities_are(LowerRight, &[(80, 2)]), None);
    // Check that no face is connected to a city with different revenue or
    // number of token spaces.
    for face in FACES {
        assert_eq!(tile.connected_cities_are(face, &[(70, 2)]), None);
        assert_eq!(tile.connected_cities_are(face, &[(80, 1)]), None);
    }
}

#[test]
/// Check face-to-city connections for tile 639.
fn tile_639() {
    let hex = Hex::default();

    let tile = Kind::_639.build(&hex);
    for face in FACES {
        assert_eq!(
            tile.connected_cities_are(face, &[(100, 4)]),
            Some(vec![0])
        );
        assert_eq!(tile.connected_cities_are(face, &[(100, 3)]), None);
        assert_eq!(tile.connected_cities_are(face, &[(110, 4)]), None);
    }
}

#[test]
/// Check face-to-city connections for tile X2.
fn tile_x2() {
    let hex = Hex::default();

    let tile = Kind::X2.build(&hex);
    let ix_vert = {
        let res = tile.connected_cities_are(Top, &[(50, 1)]);
        assert_eq!(tile.connected_cities_are(Bottom, &[(50, 1)]), res);
        assert!(res.is_some());
        let ixs = res.unwrap();
        assert_eq!(ixs.len(), 1);
        ixs[0]
    };
    let ix_upper = {
        let res = tile.connected_cities_are(UpperLeft, &[(50, 1)]);
        assert_eq!(tile.connected_cities_are(UpperRight, &[(50, 1)]), res);
        assert!(res.is_some());
        let ixs = res.unwrap();
        assert_eq!(ixs.len(), 1);
        ixs[0]
    };
    let ix_lower = {
        let res = tile.connected_cities_are(LowerLeft, &[(50, 1)]);
        assert_eq!(tile.connected_cities_are(LowerRight, &[(50, 1)]), res);
        assert!(res.is_some());
        let ixs = res.unwrap();
        assert_eq!(ixs.len(), 1);
        ixs[0]
    };
    assert_ne!(ix_vert, ix_lower);
    assert_ne!(ix_vert, ix_upper);
    assert_ne!(ix_lower, ix_upper);
}

#[test]
/// Check face-to-city connections for tile X5.
fn tile_x5() {
    let hex = Hex::default();

    let tile = Kind::X5.build(&hex);
    let ix_vert = {
        let res = tile.connected_cities_are(Top, &[(70, 1)]);
        assert_eq!(tile.connected_cities_are(Bottom, &[(70, 1)]), res);
        assert!(res.is_some());
        let ixs = res.unwrap();
        assert_eq!(ixs.len(), 1);
        ixs[0]
    };
    assert!(ix_vert < 2);
    for face in FACES {
        if face == Top || face == Bottom {
            continue;
        }
        assert_eq!(tile.connected_cities_are(face, &[(70, 1)]), None);
        let res = tile.connected_cities_are(face, &[(70, 2)]);
        assert!(res.is_some());
        let ixs = res.unwrap();
        assert_eq!(ixs.len(), 1);
        let ix = ixs[0];
        assert!(ix < 2);
        assert_ne!(ix, ix_vert);
    }
}
