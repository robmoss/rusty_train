//! Defines the address of each town and city on the map.

use n18map::HexAddress;

/// Defines the address of each town and city on the map.
pub enum Location {
    Akron,
    Albany,
    Allentown,
    Altoona,
    AtlanticCity,
    Baltimore,
    Barre,
    Boston,
    Buffalo,
    Burlington,
    CanadianWestE,
    CanadianWestW,
    Canton,
    Chicago,
    Cleveland,
    Columbus,
    DeepSouth,
    Detroit,
    Dunkirk,
    Erie,
    FallRiver,
    Flint,
    GulfOfMexicoN,
    GulfOfMexicoS,
    Hamilton,
    Hartfort,
    Kingston,
    Lancaster,
    Lansing,
    London,
    MaritimeProvinces,
    Montreal,
    NewHaven,
    NewYork,
    Ottawa,
    Philadelphia,
    Pittsburgh,
    Providence,
    Reading,
    Richmond,
    Rochester,
    Scranton,
    Toledo,
    Toronto,
    Trenton,
    Washington,
    Windsor,
}

impl Location {
    /// Returns the hex address for the provided location.
    pub fn address(&self) -> HexAddress {
        use Location::*;
        let addr = match self {
            Akron => "G7",
            Albany => "S5",
            Allentown => "Q7",
            Altoona => "L8",
            AtlanticCity => "S9",
            Baltimore => "O9",
            Barre => "J2",
            Boston => "W5",
            Buffalo => "K5",
            Burlington => "T2",
            CanadianWestE => "K1",
            CanadianWestW => "I1",
            Canton => "G7",
            Chicago => "B6",
            Cleveland => "F6",
            Columbus => "D8",
            DeepSouth => "M11",
            Detroit => "E5",
            Dunkirk => "K5",
            Erie => "J6",
            FallRiver => "X6",
            Flint => "D4",
            GulfOfMexicoN => "A9",
            GulfOfMexicoS => "B10",
            Hamilton => "J4",
            Hartfort => "T6",
            Kingston => "O3",
            Lancaster => "P8",
            Lansing => "B4",
            London => "G5",
            MaritimeProvinces => "X2",
            Montreal => "S1",
            NewHaven => "T6",
            NewYork => "S7",
            Ottawa => "P2",
            Philadelphia => "R8",
            Pittsburgh => "J8",
            Providence => "V6",
            Reading => "Q7",
            Richmond => "O11",
            Rochester => "N4",
            Scranton => "P6",
            Toledo => "D6",
            Toronto => "J4",
            Trenton => "R8",
            Washington => "N10",
            Windsor => "E5",
        };
        super::COORDS.parse(addr).unwrap()
    }

    /// Returns the name of the provided location.
    pub fn as_str(&self) -> &'static str {
        use Location::*;
        match self {
            Akron => "Akron",
            Albany => "Albany",
            Allentown => "Allentown",
            Altoona => "Altoona",
            AtlanticCity => "Atlantic City",
            Baltimore => "Baltimore",
            Barre => "Barre",
            Boston => "Boston",
            Buffalo => "Buffalo",
            Burlington => "Burlington",
            CanadianWestE => "Canadian West",
            CanadianWestW => "Canadian West",
            Canton => "Canton",
            Chicago => "Chicago",
            Cleveland => "Cleveland",
            Columbus => "Columbus",
            DeepSouth => "Deep South",
            Detroit => "Detroit",
            Dunkirk => "Dunkirk",
            Erie => "Erie",
            FallRiver => "Fall River",
            Flint => "Flint",
            GulfOfMexicoN => "Gulf of Mexico",
            GulfOfMexicoS => "Gulf of Mexico",
            Hamilton => "Hamilton",
            Hartfort => "Hartfort",
            Kingston => "Kingston",
            Lancaster => "Lancaster",
            Lansing => "Lansing",
            London => "London",
            MaritimeProvinces => "Maritime Provinces",
            Montreal => "Montreal",
            NewHaven => "New Haven",
            NewYork => "New York",
            Ottawa => "Ottawa",
            Philadelphia => "Philadelphia",
            Pittsburgh => "Pittsburgh",
            Providence => "Providence",
            Reading => "Reading",
            Richmond => "Richmond",
            Rochester => "Rochester",
            Scranton => "Scranton",
            Toledo => "Toledo",
            Toronto => "Toronto",
            Trenton => "Trenton",
            Washington => "Washington",
            Windsor => "Windsor",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Location;
    use super::Location::*;

    const ALL_LOCNS: [Location; 47] = [
        Akron,
        Albany,
        Allentown,
        Altoona,
        AtlanticCity,
        Baltimore,
        Barre,
        Boston,
        Buffalo,
        Burlington,
        CanadianWestE,
        CanadianWestW,
        Canton,
        Chicago,
        Cleveland,
        Columbus,
        DeepSouth,
        Detroit,
        Dunkirk,
        Erie,
        FallRiver,
        Flint,
        GulfOfMexicoN,
        GulfOfMexicoS,
        Hamilton,
        Hartfort,
        Kingston,
        Lancaster,
        Lansing,
        London,
        MaritimeProvinces,
        Montreal,
        NewHaven,
        NewYork,
        Ottawa,
        Philadelphia,
        Pittsburgh,
        Providence,
        Reading,
        Richmond,
        Rochester,
        Scranton,
        Toledo,
        Toronto,
        Trenton,
        Washington,
        Windsor,
    ];

    /// Ensures that each location's name appears in the name of the tile that
    /// is initially placed at the location.
    #[test]
    fn test_location_addresses() {
        let initial_state = super::super::initial_map();
        for locn in &ALL_LOCNS {
            let locn_addr = locn.address();
            let locn_name = locn.as_str();
            assert!(initial_state.iter().any(|(addr, tile_opt)| {
                if let Some((tile_name, _rotn)) = tile_opt {
                    *addr == locn_addr && tile_name.contains(locn_name)
                } else {
                    false
                }
            }));
        }
    }
}
