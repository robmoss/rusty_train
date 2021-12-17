//! Defines the address of each town and city on the map.

use n18map::HexAddress;

/// Defines the address of each town and city on the map.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Anan,
    Muki,
    Muroto,
    Nahari,
    Nangoku,
    Kouchi,
    Kubokawa,
    Nakamura,
    Sukumo,
    Uwajima,
    Yawatahama,
    Ohzu,
    Matsuyama,
    Imabari,
    Saijou,
    Niihama,
    Kotohira,
    Marugame,
    SakaideAndOkoyama,
    Takamatsu,
    RitsurinKouen,
    NarutoAndAwaji,
    Tokushima,
    Komatsujima,
    Ikeda,
}

impl Location {
    /// Returns the hex address for the provided location.
    pub fn address(&self) -> HexAddress {
        use Location::*;
        let addr = match self {
            Anan => "J11",
            Muki => "I12",
            Muroto => "G14",
            Nahari => "G12",
            Nangoku => "G10",
            Kouchi => "F9",
            Kubokawa => "C10",
            Nakamura => "B11",
            Sukumo => "A10",
            Uwajima => "B7",
            Yawatahama => "B3",
            Ohzu => "C4",
            Matsuyama => "E2",
            Imabari => "F1",
            Saijou => "F3",
            Niihama => "G4",
            Kotohira => "I4",
            Marugame => "I2",
            SakaideAndOkoyama => "J1",
            Takamatsu => "K4",
            RitsurinKouen => "J5",
            NarutoAndAwaji => "L7",
            Tokushima => "K8",
            Komatsujima => "J9",
            Ikeda => "H7",
        };
        super::COORDS.parse(addr).unwrap()
    }

    /// Returns the name of the provided location.
    pub fn as_str(&self) -> &'static str {
        use Location::*;
        match self {
            Anan => "Anan",
            Muki => "Muki",
            Muroto => "Muroto",
            Nahari => "Nahari",
            Nangoku => "Nangoku",
            Kouchi => "Kouchi",
            Kubokawa => "Kubokawa",
            Nakamura => "Nakamura",
            Sukumo => "Sukumo",
            Uwajima => "Uwajima",
            Yawatahama => "Yawatahama",
            Ohzu => "Ohzu",
            Matsuyama => "Matsuyama",
            Imabari => "Imabari",
            Saijou => "Saijou",
            Niihama => "Niihama",
            Kotohira => "Kotohira",
            Marugame => "Marugame",
            SakaideAndOkoyama => "Sakaide & Okoyama",
            Takamatsu => "Takamatsu",
            RitsurinKouen => "Ritsurin Kouen",
            NarutoAndAwaji => "Naruto & Awaji",
            Tokushima => "Tokushima",
            Komatsujima => "Komatsujima",
            Ikeda => "Ikeda",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Location;
    use super::Location::*;

    const ALL_LOCNS: [Location; 25] = [
        Anan,
        Muki,
        Muroto,
        Nahari,
        Nangoku,
        Kouchi,
        Kubokawa,
        Nakamura,
        Sukumo,
        Uwajima,
        Yawatahama,
        Ohzu,
        Matsuyama,
        Imabari,
        Saijou,
        Niihama,
        Kotohira,
        Marugame,
        SakaideAndOkoyama,
        Takamatsu,
        RitsurinKouen,
        NarutoAndAwaji,
        Tokushima,
        Komatsujima,
        Ikeda,
    ];

    /// Ensures that each location has a valid hex address.
    #[test]
    fn test_location_addresses() {
        for locn in &ALL_LOCNS {
            locn.address();
        }
    }
}
