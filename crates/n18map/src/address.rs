//! Defines coordinate systems for identifying map hexes.
//!
//! [Map](crate::Map) and [HexAddress] use **offset** coordinates, which
//! identify each map hex with a `(row, column)` tuple.
//! Note that these coordinates depend on the hex [Orientation] (see the
//! example diagrams, below).
//!
//! It is typically more convenient to use alpha-numeric coordinates, which
//! identify each map hex with a string of the form `[A-Z]+[0-9]+`.
//! The [Coordinates] type defines how the relationship between these strings
//! and the map hexes.
//!
//! # Flat-top orientation: alternating rows
//!
//! With alternating-row coordinates, odd-numbered columns are "shoved down":
//!
//! ```text
//!   ___       ___       ___       ___       ___       ___
//!  /   \     /   \     /   \     /   \     /   \     /   \
//! / 0,0 \___/ 0,2 \___/ 0,4 \___/ 0,6 \___/ 0,8 \___/ 0,10\
//! \     /   \     /   \     /   \     /   \     /   \     /
//!  \___/ 0,1 \___/ 0,3 \___/ 0,5 \___/ 0,7 \___/ 0,9 \___/
//!  /   \     /   \     /   \     /   \     /   \     /   \
//! / 1,0 \___/ 1,2 \___/ 1,4 \___/ 1,6 \___/ 1,8 \___/ 1,10\
//! \     /   \     /   \     /   \     /   \     /   \     /
//!  \___/ 1,1 \___/ 1,3 \___/ 1,5 \___/ 1,7 \___/ 1,9 \___/
//!  /   \     /   \     /   \     /   \     /   \     /   \
//! / 2,0 \___/ 2,2 \___/ 2,4 \___/ 2,6 \___/ 2,8 \___/ 2,10\
//! \     /   \     /   \     /   \     /   \     /   \     /
//!  \___/     \___/     \___/     \___/     \___/     \___/
//! ```
//!
//! ## Pointed-top orientation: alternating columns
//!
//! With alternating-column coordinates, odd-numbered rows are "shoved right":
//!
//! ```text
//!   / \   / \   / \   / \   / \   / \   / \   / \   / \
//!  /   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \
//! |     |     |     |     |     |     |     |     |     |
//! | 0,0 | 0,1 | 0,2 | 0,3 | 0,4 | 0,5 | 0,6 | 0,7 | 0,8 |
//! |     |     |     |     |     |     |     |     |     |
//!  \   / \   / \   / \   / \   / \   / \   / \   / \   / \
//!   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \
//!    |     |     |     |     |     |     |     |     |     |
//!    | 1,0 | 1,1 | 1,2 | 1,3 | 1,4 | 1,5 | 1,6 | 1,7 | 1,8 |
//!    |     |     |     |     |     |     |     |     |     |
//!   / \   / \   / \   / \   / \   / \   / \   / \   / \   /
//!  /   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \ /
//! |     |     |     |     |     |     |     |     |     |
//! | 2,0 | 2,1 | 2,2 | 2,3 | 2,4 | 2,5 | 2,6 | 2,7 | 2,8 |
//! |     |     |     |     |     |     |     |     |     |
//!  \   / \   / \   / \   / \   / \   / \   / \   / \   /
//!   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \ /
//! ```
//!
//! # Alpha-numeric coordinates
//!
//! To identify map hexes with strings such as "A1" and "B4", the following
//! settings must be defined:
//!
//! - The hex [Orientation] (as shown in the diagrams above);
//!
//! - Whether letters represent [columns](Letters::AsColumns) or
//!   [rows](Letters::AsRows); and
//!
//! - Whether the first (top) row contains columns with [even
//!   numbers](FirstRow::EvenColumns) or [odd numbers](FirstRow::OddColumns).
//!
//! These settings are collected in the [Coordinates] type, which translates
//! between alpha-numeric coordinates and [HexAddress] values:
//!
//! ```rust
//! # use n18hex::Orientation;
//! # use n18map::{Coordinates, FirstRow, HexAddress, Letters};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let coords = Coordinates {
//!     orientation: Orientation::FlatTop,
//!     letters: Letters::AsColumns,
//!     first_row: FirstRow::OddColumns,
//! };
//!
//! // Parse alpha-numeric strings into HexAddress values.
//! assert_eq!(coords.parse("A1")?, (0, 0).into());
//! assert_eq!(coords.parse("A3")?, (1, 0).into());
//! assert!(coords.parse("A2").is_err());
//! assert_eq!(coords.parse("B2")?, (0, 1).into());
//! assert_eq!(coords.parse("B4")?, (1, 1).into());
//! assert!(coords.parse("B3").is_err());
//!
//! // Convert HexAddress values back to alpha-numeric strings.
//! assert_eq!(coords.format(&(1, 0).into()), Some("A3".to_string()));
//! assert_eq!(coords.format(&(0, 1).into()), Some("B2".to_string()));
//! # Ok(())
//! # }
//! ```
//!
//! ## Special locations
//!
//! It is not always possible to identify each map hex using alpha-numeric
//! coordinates.
//!
//! For example, any hex that is above and/or to the left of the first hex
//! (either "A1" or "A2") cannot be identified, since alpha-numeric
//! coordinates (as implemented) do not support negative row or column
//! numbers.
//!
//! But such hexes can be identified by their **relative position**:
//!
//! ```rust
//! # use n18hex::{HexFace, Orientation};
//! # use n18map::HexAddress;
//! let a1: HexAddress = (0, 0).into();
//! let above_a1 = a1.adjacent(HexFace::Top, Orientation::FlatTop);
//! assert_eq!(above_a1, (-1, 0).into());
//! ```
//!
//! # Logical coordinates
//!
//! Map hexes can also be identified using logical coordinates.
//! Logical coordinates are defined by a row number and a column number, where
//! the column number may be any `isize` value and the valid row numbers are
//! determined by column number:
//!
//! - For even-numbered columns, the row number **must be even**; and
//! - For odd-numbered columns, the row number **must be odd**.
//!
//! Note that logical coordinates are **independent** of the map hex
//! orientation, and whether the first row contains even or odd columns.
//!
//! Valid `(row, column)` pairs include `(0, 0)`, `(1, 7)`, and `(-2, -4)`.
//!
//! Invalid `(row, column)` pairs include `(0, 1)`, `(1, 8)`, and `(-2, -3)`.
//!
//! ```rust
//! # use n18map::HexAddress;
//! let valid_addr = HexAddress::logical(1, 3);
//! assert!(valid_addr.is_some());
//! let invalid_addr = HexAddress::logical(1, 4);
//! assert!(invalid_addr.is_none());
//! ```
//!

use n18hex::{HexFace, Orientation};

/// Letters may represent columns or rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Letters {
    AsColumns,
    AsRows,
}

/// Defines where the first row begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FirstRow {
    OddColumns,
    EvenColumns,
}

/// Identifies how alpha-numeric coordinates translate to map locations.
///
/// Diagrams of each supported coordinate system are shown below.
///
/// # Flat-top orientation
///
/// ## First row contains odd columns
///
/// With letters defining the column:
///
/// ```text
///   ___       ___       ___       ___       ___       ___
///  /   \     /   \     /   \     /   \     /   \     /   \
/// / A1  \___/ C1  \___/ E1  \___/ G1  \___/ I1  \___/ K1  \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/ B2  \___/ D2  \___/ F2  \___/ H2  \___/ L2  \___/
///  /   \     /   \     /   \     /   \     /   \     /   \
/// / A3  \___/ C3  \___/ E3  \___/ G3  \___/ I3  \___/ K3  \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/ B4  \___/ D4  \___/ F4  \___/ H4  \___/ L4  \___/
///  /   \     /   \     /   \     /   \     /   \     /   \
/// / A5  \___/ C5  \___/ E5  \___/ G5  \___/ I5  \___/ K5  \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/     \___/     \___/     \___/     \___/     \___/
/// ```
///
/// With letters defining the row:
///
/// ```text
///   ___       ___       ___       ___       ___       ___
///  /   \     /   \     /   \     /   \     /   \     /   \
/// / A1  \___/ A3  \___/ A5  \___/ A7  \___/ A9  \___/ A11 \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/ B2  \___/ B4  \___/ B6  \___/ B8  \___/ B10 \___/
///  /   \     /   \     /   \     /   \     /   \     /   \
/// / C1  \___/ C3  \___/ C5  \___/ C7  \___/ C9  \___/ C11 \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/ D2  \___/ D4  \___/ D6  \___/ D8  \___/ D10 \___/
///  /   \     /   \     /   \     /   \     /   \     /   \
/// / E1  \___/ E3  \___/ E5  \___/ E7  \___/ E9  \___/ E11 \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/     \___/     \___/     \___/     \___/     \___/
/// ```
///
/// ## First row contains even columns
///
/// With letters defining the column:
///
/// ```text
///   ___       ___       ___       ___       ___       ___
///  /   \     /   \     /   \     /   \     /   \     /   \
/// /     \___/     \___/     \___/     \___/     \___/     \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/ B1  \___/ D1  \___/ F1  \___/ H1  \___/ L1  \___/
///  /   \     /   \     /   \     /   \     /   \     /   \
/// / A2  \___/ C2  \___/ E2  \___/ G2  \___/ I2  \___/ K2  \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/ B3  \___/ D3  \___/ F3  \___/ H3  \___/ L3  \___/
///  /   \     /   \     /   \     /   \     /   \     /   \
/// / A4  \___/ C4  \___/ E4  \___/ G4  \___/ I4  \___/ K4  \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/     \___/     \___/     \___/     \___/     \___/
/// ```
///
/// With letters defining the row:
///
/// ```text
///   ___       ___       ___       ___       ___       ___
///  /   \     /   \     /   \     /   \     /   \     /   \
/// /     \___/     \___/     \___/     \___/     \___/     \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/ A2  \___/ A4  \___/ A6  \___/ A8  \___/ A10 \___/
///  /   \     /   \     /   \     /   \     /   \     /   \
/// / B1  \___/ B3  \___/ B5  \___/ B7  \___/ B9  \___/ B11 \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/ C2  \___/ C4  \___/ C6  \___/ C8  \___/ C10 \___/
///  /   \     /   \     /   \     /   \     /   \     /   \
/// / D1  \___/ D3  \___/ D5  \___/ D7  \___/ D9  \___/ D11 \
/// \     /   \     /   \     /   \     /   \     /   \     /
///  \___/     \___/     \___/     \___/     \___/     \___/
/// ```
///
/// # Pointed-top orientation
///
/// ## First row contains odd columns
///
/// With letters defining the column:
///
/// ```text
///   / \   / \   / \   / \   / \   / \
///  /   \ /   \ /   \ /   \ /   \ /   \
/// |     |     |     |     |     |     |
/// | A1  | C1  | E1  | G1  | I1  | K1  |
/// |     |     |     |     |     |     |
///  \   / \   / \   / \   / \   / \   / \
///   \ /   \ /   \ /   \ /   \ /   \ /   \
///    |     |     |     |     |     |     |
///    | B2  | D2  | F2  | H2  | J2  | L2  |
///    |     |     |     |     |     |     |
///   / \   / \   / \   / \   / \   / \   /
///  /   \ /   \ /   \ /   \ /   \ /   \ /
/// |     |     |     |     |     |     |
/// | A3  | C3  | E3  | G3  | I3  | K3  |
/// |     |     |     |     |     |     |
///  \   / \   / \   / \   / \   / \   / \
///   \ /   \ /   \ /   \ /   \ /   \ /   \
///    |     |     |     |     |     |     |
///    | B4  | D4  | F4  | H4  | J4  | L4  |
///    |     |     |     |     |     |     |
///   / \   / \   / \   / \   / \   / \   /
///  /   \ /   \ /   \ /   \ /   \ /   \ /
/// ```
///
/// With letters defining the row:
///
/// ```text
///   / \   / \   / \   / \   / \   / \
///  /   \ /   \ /   \ /   \ /   \ /   \
/// |     |     |     |     |     |     |
/// | A1  | A3  | A5  | A7  | A9  | A11 |
/// |     |     |     |     |     |     |
///  \   / \   / \   / \   / \   / \   / \
///   \ /   \ /   \ /   \ /   \ /   \ /   \
///    |     |     |     |     |     |     |
///    | B2  | B4  | B6  | B8  | B10 | B12 |
///    |     |     |     |     |     |     |
///   / \   / \   / \   / \   / \   / \   /
///  /   \ /   \ /   \ /   \ /   \ /   \ /
/// |     |     |     |     |     |     |
/// | C1  | C3  | C5  | C7  | C9  | C11 |
/// |     |     |     |     |     |     |
///  \   / \   / \   / \   / \   / \   / \
///   \ /   \ /   \ /   \ /   \ /   \ /   \
///    |     |     |     |     |     |     |
///    | D2  | D4  | D6  | D8  | D10 | D12 |
///    |     |     |     |     |     |     |
///   / \   / \   / \   / \   / \   / \   /
///  /   \ /   \ /   \ /   \ /   \ /   \ /
/// ```
///
/// ## First row contains even columns
///
/// With letters defining the column:
///
/// ```text
///   / \   / \   / \   / \   / \   / \
///  /   \ /   \ /   \ /   \ /   \ /   \
/// |     |     |     |     |     |     |
/// |     |     |     |     |     |     |
/// |     |     |     |     |     |     |
///  \   / \   / \   / \   / \   / \   / \
///   \ /   \ /   \ /   \ /   \ /   \ /   \
///    |     |     |     |     |     |     |
///    | B1  | D1  | F1  | H1  | J1  | L1  |
///    |     |     |     |     |     |     |
///   / \   / \   / \   / \   / \   / \   /
///  /   \ /   \ /   \ /   \ /   \ /   \ /
/// |     |     |     |     |     |     |
/// | A2  | C2  | E2  | G2  | I2  | K2 |
/// |     |     |     |     |     |     |
///  \   / \   / \   / \   / \   / \   / \
///   \ /   \ /   \ /   \ /   \ /   \ /   \
///    |     |     |     |     |     |     |
///    | B3  | D3  | F3  | H3  | J3  | L3  |
///    |     |     |     |     |     |     |
///   / \   / \   / \   / \   / \   / \   /
///  /   \ /   \ /   \ /   \ /   \ /   \ /
/// ```
///
/// With letters defining the row:
///
/// ```text
///   / \   / \   / \   / \   / \   / \
///  /   \ /   \ /   \ /   \ /   \ /   \
/// |     |     |     |     |     |     |
/// |     |     |     |     |     |     |
/// |     |     |     |     |     |     |
///  \   / \   / \   / \   / \   / \   / \
///   \ /   \ /   \ /   \ /   \ /   \ /   \
///    |     |     |     |     |     |     |
///    | A2  | A4  | A6  | A8  | A10 | A12 |
///    |     |     |     |     |     |     |
///   / \   / \   / \   / \   / \   / \   /
///  /   \ /   \ /   \ /   \ /   \ /   \ /
/// |     |     |     |     |     |     |
/// | B1  | B3  | B5  | B7  | B9  | B11 |
/// |     |     |     |     |     |     |
///  \   / \   / \   / \   / \   / \   / \
///   \ /   \ /   \ /   \ /   \ /   \ /   \
///    |     |     |     |     |     |     |
///    | C2  | C4  | C6  | C8  | C10 | C12 |
///    |     |     |     |     |     |     |
///   / \   / \   / \   / \   / \   / \   /
///  /   \ /   \ /   \ /   \ /   \ /   \ /
/// ```
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coordinates {
    /// The hexagon orientation.
    pub orientation: Orientation,
    /// The choice of letters to represent columns or rows.
    pub letters: Letters,
    /// The columns contained in the first row.
    pub first_row: FirstRow,
}

impl From<(Orientation, Letters, FirstRow)> for Coordinates {
    fn from(src: (Orientation, Letters, FirstRow)) -> Self {
        let (orientation, letters, first_row) = src;
        Coordinates {
            orientation,
            letters,
            first_row,
        }
    }
}

impl Coordinates {
    /// Parses an alpha-numeric string and returns the corresponding hex
    /// address.
    ///
    /// # Errors
    ///
    /// Returns a [ParseHexAddressError] if the string is not a valid address.
    pub fn parse(
        &self,
        text: &str,
    ) -> Result<HexAddress, ParseHexAddressError> {
        // Parse the input string.
        let (az_num, digit_num) =
            parse_az_and_digit(text).ok_or(ParseHexAddressError {
                text: text.to_string(),
            })?;

        // Account for row/column ordering.
        let (row_in, col_in) = match self.letters {
            Letters::AsColumns => (digit_num, az_num),
            Letters::AsRows => (az_num, digit_num),
        };

        // Account for the choice of first row.
        let (row_in, col_in) = match self.first_row {
            FirstRow::OddColumns => {
                // NOTE: no translation needed.
                (row_in, col_in)
            }
            FirstRow::EvenColumns => {
                // NOTE: shift everything down one row.
                (row_in + 1, col_in)
            }
        };

        // Check that we have a valid combination of row and column.
        // They should *both* be even, or *both* be odd.
        let odd_row = row_in % 2 == 1;
        let odd_col = col_in % 2 == 1;
        if odd_row != odd_col {
            return Err(ParseHexAddressError {
                text: text.to_string(),
            });
        }

        // Convert to offset coordinates, noting that row_in and col_in start
        // at one, not zero.
        let (row, col) = match self.orientation {
            Orientation::FlatTop => {
                if odd_col {
                    (row_in / 2, col_in - 1)
                } else {
                    ((row_in - 1) / 2, col_in - 1)
                }
            }
            Orientation::PointedTop => {
                if odd_col {
                    (row_in - 1, col_in / 2)
                } else {
                    (row_in - 1, (col_in - 1) / 2)
                }
            }
        };

        Ok((row, col).into())
    }

    /// Returns the alpha-numeric string that identifies the provided hex
    /// address, or `None` if there is no alpha-numeric string for the
    /// provided hex address.
    pub fn format(&self, addr: &HexAddress) -> Option<String> {
        let (row, col) = addr.into();

        let (row, col) = match self.orientation {
            Orientation::FlatTop => {
                if col % 2 == 0 {
                    (row * 2 + 1, col + 1)
                } else {
                    (row * 2 + 2, col + 1)
                }
            }
            Orientation::PointedTop => {
                if row % 2 == 0 {
                    (row + 1, col * 2 + 1)
                } else {
                    (row + 1, col * 2 + 2)
                }
            }
        };

        assert_eq!(row % 2, col % 2);

        let (row, col) = match self.first_row {
            FirstRow::OddColumns => {
                // NOTE: no translation needed.
                (row, col)
            }
            FirstRow::EvenColumns => {
                // NOTE: undo the shift down one row.
                (row - 1, col)
            }
        };

        // Account for row/column ordering.
        let (letter, digits) = match self.letters {
            Letters::AsColumns => (col, row),
            Letters::AsRows => (row, col),
        };

        if letter > 0 {
            let mut output = num_to_az(letter)?;
            output.push_str(&format!("{}", digits));
            Some(output)
        } else {
            None
        }
    }
}

/// Converts a number into a string of uppercase ASCII letters.
///
/// Note that this **is not** a standard base-26 format, because there is no
/// zero digit.
/// We cannot define `A` as zero, because this would mean that "A", "AA",
/// "AAA", etc, would all equal zero, and we want "A" == 1, "AA" = 27, etc.
///
/// # Examples
///
/// ```rust
/// # use n18map::address::num_to_az;
/// assert_eq!(num_to_az(1), Some("A".to_string()));
/// assert_eq!(num_to_az(2), Some("B".to_string()));
/// assert_eq!(num_to_az(3), Some("C".to_string()));
/// assert_eq!(num_to_az(26), Some("Z".to_string()));
/// assert_eq!(num_to_az(27), Some("AA".to_string()));
/// assert_eq!(num_to_az(28), Some("AB".to_string()));
/// assert_eq!(num_to_az(29), Some("AC".to_string()));
/// assert_eq!(num_to_az(52), Some("AZ".to_string()));
/// assert_eq!(num_to_az(53), Some("BA".to_string()));
/// assert_eq!(num_to_az(54), Some("BB".to_string()));
/// assert_eq!(num_to_az(55), Some("BC".to_string()));
/// assert_eq!(num_to_az(78), Some("BZ".to_string()));
/// assert_eq!(num_to_az(79), Some("CA".to_string()));
/// assert_eq!(num_to_az(0), None);
/// assert_eq!(num_to_az(-1), None);
/// ```
pub fn num_to_az(mut value: isize) -> Option<String> {
    // NOTE: convert from 1-based values to 0-based values.
    value -= 1;

    // This loop will terminate after the most-significant digit has been
    // added to `digits`, because this results in `value` becoming negative.
    let mut digits = vec![];
    while value >= 0 {
        // Insert the least-significant base-26 digit.
        let digit = value % 26;
        digits.insert(0, digit);
        // Remove this digit, and subtract 1 so that we continue to use
        // 0-based values (i.e., 0 = A, 1 = B, etc).
        value /= 26;
        value -= 1;
    }

    if digits.is_empty() {
        return None;
    }

    // Convert the sequence of digits into characters.
    let chars: Vec<char> = ('A'..='Z').collect();
    let mut output = String::from("");
    for digit in &digits {
        output.push(chars[*digit as usize]);
    }

    Some(output)
}

/// Parses a string of uppercase ASCII letters and returns their numeric
/// value.
///
/// Note that this **is not** a standard base-26 format, because there is no
/// zero digit.
/// We cannot define `A` as zero, because this would mean that "A", "AA",
/// "AAA", etc, would all equal zero, and we want "A" == 1, "AA" = 27, etc.
///
/// # Examples
///
/// ```rust
/// # use n18map::address::az_to_num;
/// assert_eq!(az_to_num("A"), Some(1));
/// assert_eq!(az_to_num("B"), Some(2));
/// assert_eq!(az_to_num("Z"), Some(26));
/// assert_eq!(az_to_num("AA"), Some(27));
/// assert_eq!(az_to_num("AB"), Some(28));
/// assert_eq!(az_to_num("AZ"), Some(52));
/// assert_eq!(az_to_num("BA"), Some(53));
/// assert_eq!(az_to_num("BB"), Some(54));
/// assert_eq!(az_to_num("BZ"), Some(78));
/// assert_eq!(az_to_num("a"), None);
/// assert_eq!(az_to_num("123"), None);
/// ```
pub fn az_to_num(text: &str) -> Option<isize> {
    let a = 'A';
    let z = 'Z';
    let digits_opt: Option<Vec<isize>> = text
        .chars()
        .map(|ch| {
            if ch >= a && ch <= z {
                Some((ch as u32 - a as u32 + 1) as isize)
            } else {
                None
            }
        })
        .collect();
    if let Some(digits) = digits_opt {
        let num_digits = digits.len();
        let total = digits
            .iter()
            .enumerate()
            .map(|(ix, digit)| {
                let exponent = (num_digits - ix - 1) as u32;
                let scale = 26_isize.pow(exponent);
                digit * scale
            })
            .sum();
        Some(total)
    } else {
        None
    }
}

/// Parses an alpha-numeric string (`[A-Z]+[0-9]+`) and returns the numbers
/// corresponding to the alphabetic and numeric segments, respectively.
///
/// Note that the alphabetic segment **is not** a standard base-26 format.
/// See [az_to_num] and [num_to_az] for details.
///
/// # Examples
///
/// ```rust
/// # use n18map::address::parse_az_and_digit;
/// assert_eq!(parse_az_and_digit("A1"), Some((1, 1)));
/// assert_eq!(parse_az_and_digit("A2"), Some((1, 2)));
/// assert_eq!(parse_az_and_digit("B3"), Some((2, 3)));
/// assert_eq!(parse_az_and_digit("AB12"), Some((28, 12)));
/// assert_eq!(parse_az_and_digit("C-1"), Some((3, -1)));
/// assert_eq!(parse_az_and_digit("ABC"), None);
/// assert_eq!(parse_az_and_digit("123"), None);
/// ```
pub fn parse_az_and_digit(text: &str) -> Option<(isize, isize)> {
    let (ix, _) = text
        .char_indices()
        .find(|(_ix, ch)| !('A'..='Z').contains(ch))?;
    if ix < 1 {
        // NOTE: text must start with one or more uppercase letters.
        return None;
    }
    let az_num = az_to_num(&text[..ix])?;
    let digits = text[ix..].parse::<isize>().ok()?;
    Some((az_num, digits))
}

/// The error returns when an alpha-numeric string cannot be parsed as a valid
/// map hex.
#[derive(Debug)]
pub struct ParseHexAddressError {
    pub text: String,
}

impl std::fmt::Display for ParseHexAddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Could not parse hex address '{}'", self.text)
    }
}

impl std::error::Error for ParseHexAddressError {}

pub trait Adjacency {
    fn adjacent(&self, addr: HexAddress, face: HexFace) -> HexAddress;
}

impl Adjacency for Orientation {
    /// Returns the address of the hex that is adjacent to the specified face
    /// (in terms of map orientation, not tile orientation) of the given hex.
    fn adjacent(&self, addr: HexAddress, face: HexFace) -> HexAddress {
        match self {
            Orientation::FlatTop => {
                let is_upper = addr.col % 2 == 0;
                match face {
                    HexFace::Top => (addr.row - 1, addr.col).into(),
                    HexFace::UpperRight => {
                        if is_upper {
                            (addr.row - 1, addr.col + 1).into()
                        } else {
                            (addr.row, addr.col + 1).into()
                        }
                    }
                    HexFace::LowerRight => {
                        if is_upper {
                            (addr.row, addr.col + 1).into()
                        } else {
                            (addr.row + 1, addr.col + 1).into()
                        }
                    }
                    HexFace::Bottom => (addr.row + 1, addr.col).into(),
                    HexFace::LowerLeft => {
                        if is_upper {
                            (addr.row, addr.col - 1).into()
                        } else {
                            (addr.row + 1, addr.col - 1).into()
                        }
                    }
                    HexFace::UpperLeft => {
                        if is_upper {
                            (addr.row - 1, addr.col - 1).into()
                        } else {
                            (addr.row, addr.col - 1).into()
                        }
                    }
                }
            }
            Orientation::PointedTop => {
                // NOTE: HexFace::Top is the upper-right face, and so on.
                let is_left = addr.row % 2 == 0;
                match face {
                    HexFace::Top => {
                        if is_left {
                            (addr.row - 1, addr.col).into()
                        } else {
                            (addr.row - 1, addr.col + 1).into()
                        }
                    }
                    HexFace::UpperRight => (addr.row, addr.col + 1).into(),
                    HexFace::LowerRight => {
                        if is_left {
                            (addr.row + 1, addr.col).into()
                        } else {
                            (addr.row + 1, addr.col + 1).into()
                        }
                    }
                    HexFace::Bottom => {
                        if is_left {
                            (addr.row + 1, addr.col - 1).into()
                        } else {
                            (addr.row + 1, addr.col).into()
                        }
                    }
                    HexFace::LowerLeft => (addr.row, addr.col - 1).into(),
                    HexFace::UpperLeft => {
                        if is_left {
                            (addr.row - 1, addr.col - 1).into()
                        } else {
                            (addr.row - 1, addr.col).into()
                        }
                    }
                }
            }
        }
    }
}

/// A hex location on a `Map`, identified by row and column.
///
/// Note that negative row and column numbers are permitted.
/// This allows maps to include tiles (such as off-board locations)
/// adjacent to any regular map hex.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HexAddress {
    /// The row number.
    pub(crate) row: isize,
    /// The column number.
    pub(crate) col: isize,
}

impl HexAddress {
    /// Returns a new `HexAddress` with the specified `row` and `column`
    /// (offset coordinates).
    pub fn new(row: isize, column: isize) -> Self {
        Self { row, col: column }
    }

    /// Returns a new `HexAddress` with the specified `row` and `column`
    /// (logical coordinates) if the coordinates are valid.
    pub fn logical(row: isize, column: isize) -> Option<Self> {
        if row % 2 != column % 2 {
            None
        } else {
            Some(Self { row, col: column })
        }
    }

    /// Returns the logical column number.
    pub fn logical_column(&self) -> isize {
        self.col
    }

    /// Returns the logical row number.
    pub fn logical_row(&self) -> isize {
        if self.col % 2 == 0 {
            2 * self.row
        } else {
            2 * self.row + 1
        }
    }

    /// Returns the address of the hex that is adjacent to the specified face
    /// (in terms of map orientation, not tile orientation) of this hex.
    pub fn adjacent(
        &self,
        face: HexFace,
        orientation: Orientation,
    ) -> HexAddress {
        orientation.adjacent(*self, face)
    }

    /// Calls a closure on this hex address, and returns the hex address.
    pub fn do_here<F>(&self, mut f: F) -> &HexAddress
    where
        F: FnMut(&Self),
    {
        f(self);
        self
    }

    /// Calls a closure on the address of the hex that is adjacent to the
    /// specified face (in terms of map orientation, not tile orientation) of
    /// this hex, without doing bounds checking, and returns the new hex
    /// address.
    ///
    /// This is short-hand for calling [adjacent](HexAddress::adjacent) and
    /// [do_here](HexAddress::do_here), then returning the new hex address.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use n18hex::{HexFace, Orientation, RotateCW};
    /// use n18map::{HexAddress, Map};
    ///
    /// fn place_connected_track(map: &mut Map, starting_city: HexAddress) {
    ///     let o = Orientation::FlatTop;
    ///     starting_city
    ///         // Move to the hex below and place tile 8.
    ///         .move_and_do(HexFace::Bottom, o, |&addr| {
    ///             let _ = map.place_tile(addr, "8", RotateCW::Five);
    ///         })
    ///         // Move to the hex on the lower right and place tile 9.
    ///         .move_and_do(HexFace::LowerRight, o, |&addr| {
    ///             let _ = map.place_tile(addr, "9", RotateCW::Two);
    ///         })
    ///         // Move to the hex on the lower right and place tile 9.
    ///         .move_and_do(HexFace::LowerRight, o, |&addr| {
    ///             let _ = map.place_tile(addr, "9", RotateCW::Two);
    ///         });
    /// }
    /// ```
    pub fn move_and_do<F>(
        &self,
        face: HexFace,
        orientation: Orientation,
        f: F,
    ) -> HexAddress
    where
        F: FnMut(&Self),
    {
        let addr = self.adjacent(face, orientation);
        addr.do_here(f);
        addr
    }
}

/// Formats [HexAddress] values using offset coordinates.
impl std::fmt::Display for HexAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

/// Converts `(row, column)` tuples into a [HexAddress] value.
///
/// The `row` and `column` values are defined in terms of offset coordinates,
/// as per [HexAddress::new].
impl From<(isize, isize)> for HexAddress {
    fn from(src: (isize, isize)) -> Self {
        let (row, col) = src;
        Self { row, col }
    }
}

/// Converts [HexAddress] references into offset coordinates.
impl From<&HexAddress> for (isize, isize) {
    fn from(src: &HexAddress) -> Self {
        (src.row, src.col)
    }
}

/// Converts [HexAddress] values into offset coordinates.
impl From<HexAddress> for (isize, isize) {
    fn from(src: HexAddress) -> Self {
        (src.row, src.col)
    }
}

/// Converts `(row, column)` tuples into a [HexAddress] value.
///
/// The `row` and `column` values are defined in terms of offset coordinates,
/// as per [HexAddress::new].
impl From<&(isize, isize)> for HexAddress {
    fn from(src: &(isize, isize)) -> Self {
        let (row, col) = src;
        Self {
            row: *row,
            col: *col,
        }
    }
}

#[cfg(test)]
mod tests {
    use n18hex::Orientation::*;

    use super::FirstRow::*;
    use super::Letters::*;
    use super::*;

    /// ```text
    ///   ___       ___       ___       ___       ___       ___
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// / A1  \___/ C1  \___/ E1  \___/ G1  \___/ I1  \___/ K1  \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/ B2  \___/ D2  \___/ F2  \___/ H2  \___/ L2  \___/
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// / A3  \___/ C3  \___/ E3  \___/ G3  \___/ I3  \___/ K3  \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/ B4  \___/ D4  \___/ F4  \___/ H4  \___/ L4  \___/
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// / A5  \___/ C5  \___/ E5  \___/ G5  \___/ I5  \___/ K5  \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/     \___/     \___/     \___/     \___/     \___/
    /// ```
    ///
    #[test]
    fn test_coords_flat_cols_odd() {
        let system = Coordinates::from((FlatTop, AsColumns, OddColumns));

        // Check valid row/column combinations.
        assert_eq!(system.parse("A1").unwrap(), (0, 0).into());
        assert_eq!(system.parse("A3").unwrap(), (1, 0).into());
        assert_eq!(system.parse("A5").unwrap(), (2, 0).into());
        assert_eq!(system.parse("B2").unwrap(), (0, 1).into());
        assert_eq!(system.parse("B4").unwrap(), (1, 1).into());
        assert_eq!(system.parse("B6").unwrap(), (2, 1).into());
        assert_eq!(system.parse("C1").unwrap(), (0, 2).into());
        assert_eq!(system.parse("C3").unwrap(), (1, 2).into());
        assert_eq!(system.parse("C5").unwrap(), (2, 2).into());
        assert_eq!(system.parse("D2").unwrap(), (0, 3).into());
        assert_eq!(system.parse("D4").unwrap(), (1, 3).into());
        assert_eq!(system.parse("D6").unwrap(), (2, 3).into());

        assert_eq!(system.parse("Y9").unwrap(), (4, 24).into());
        assert_eq!(system.parse("Z10").unwrap(), (4, 25).into());
        assert_eq!(system.parse("AA11").unwrap(), (5, 26).into());
        assert_eq!(system.parse("AZ12").unwrap(), (5, 51).into());

        // Check invalid row/column combinations.
        assert!(system.parse("A0").is_err());
        assert!(system.parse("A2").is_err());
        assert!(system.parse("A4").is_err());
        assert!(system.parse("B1").is_err());
        assert!(system.parse("B3").is_err());
        assert!(system.parse("B5").is_err());
        assert!(system.parse("C0").is_err());
        assert!(system.parse("C2").is_err());
        assert!(system.parse("C4").is_err());
        assert!(system.parse("D1").is_err());
        assert!(system.parse("D3").is_err());
        assert!(system.parse("D5").is_err());

        assert!(system.parse("Y10").is_err());
        assert!(system.parse("Z11").is_err());
        assert!(system.parse("AA12").is_err());
        assert!(system.parse("AZ13").is_err());

        // Check that valid inputs round-trip through parse() and format().
        let inputs = [
            "A1", "A3", "A5", "B2", "B4", "B6", "C1", "C3", "C5", "D2", "D4",
            "D6", "Y9", "Z10", "AA11", "AZ12",
        ];
        for text in inputs {
            let addr = system.parse(text).unwrap();
            let out = system.format(&addr).unwrap();
            assert_eq!(text, out)
        }
    }

    /// ```text
    ///   / \   / \   / \   / \   / \   / \
    ///  /   \ /   \ /   \ /   \ /   \ /   \
    /// |     |     |     |     |     |     |
    /// | A1  | C1  | E1  | G1  | I1  | K1  |
    /// |     |     |     |     |     |     |
    ///  \   / \   / \   / \   / \   / \   / \
    ///   \ /   \ /   \ /   \ /   \ /   \ /   \
    ///    |     |     |     |     |     |     |
    ///    | B2  | D2  | F2  | H2  | J2  | L2  |
    ///    |     |     |     |     |     |     |
    ///   / \   / \   / \   / \   / \   / \   /
    ///  /   \ /   \ /   \ /   \ /   \ /   \ /
    /// |     |     |     |     |     |     |
    /// | A3  | C3  | E3  | G3  | I3  | K3  |
    /// |     |     |     |     |     |     |
    ///  \   / \   / \   / \   / \   / \   / \
    ///   \ /   \ /   \ /   \ /   \ /   \ /   \
    ///    |     |     |     |     |     |     |
    ///    | B4  | D4  | F4  | H4  | J4  | L4  |
    ///    |     |     |     |     |     |     |
    ///   / \   / \   / \   / \   / \   / \   /
    ///  /   \ /   \ /   \ /   \ /   \ /   \ /
    /// ```
    ///
    #[test]
    fn test_coords_pointed_cols_odd() {
        let system = Coordinates::from((PointedTop, AsColumns, OddColumns));

        // Check valid row/column combinations.
        assert_eq!(system.parse("A1").unwrap(), (0, 0).into());
        assert_eq!(system.parse("A3").unwrap(), (2, 0).into());
        assert_eq!(system.parse("A5").unwrap(), (4, 0).into());
        assert_eq!(system.parse("B2").unwrap(), (1, 0).into());
        assert_eq!(system.parse("B4").unwrap(), (3, 0).into());
        assert_eq!(system.parse("B6").unwrap(), (5, 0).into());
        assert_eq!(system.parse("C1").unwrap(), (0, 1).into());
        assert_eq!(system.parse("C3").unwrap(), (2, 1).into());
        assert_eq!(system.parse("C5").unwrap(), (4, 1).into());
        assert_eq!(system.parse("D2").unwrap(), (1, 1).into());
        assert_eq!(system.parse("D4").unwrap(), (3, 1).into());
        assert_eq!(system.parse("D6").unwrap(), (5, 1).into());

        assert_eq!(system.parse("Y9").unwrap(), (8, 12).into());
        assert_eq!(system.parse("Z10").unwrap(), (9, 12).into());
        assert_eq!(system.parse("AA11").unwrap(), (10, 13).into());
        assert_eq!(system.parse("AZ12").unwrap(), (11, 25).into());

        // Check invalid row/column combinations.
        assert!(system.parse("A0").is_err());
        assert!(system.parse("A2").is_err());
        assert!(system.parse("A4").is_err());
        assert!(system.parse("B1").is_err());
        assert!(system.parse("B3").is_err());
        assert!(system.parse("B5").is_err());
        assert!(system.parse("C0").is_err());
        assert!(system.parse("C2").is_err());
        assert!(system.parse("C4").is_err());
        assert!(system.parse("D1").is_err());
        assert!(system.parse("D3").is_err());
        assert!(system.parse("D5").is_err());

        assert!(system.parse("Y10").is_err());
        assert!(system.parse("Z11").is_err());
        assert!(system.parse("AA12").is_err());
        assert!(system.parse("AZ13").is_err());

        // Check that valid inputs round-trip through parse() and format().
        let inputs = [
            "A1", "A3", "A5", "B2", "B4", "B6", "C1", "C3", "C5", "D2", "D4",
            "D6", "Y9", "Z10", "AA11", "AZ12",
        ];
        for text in inputs {
            let addr = system.parse(text).unwrap();
            let out = system.format(&addr).unwrap();
            assert_eq!(text, out)
        }
    }

    /// ```text
    ///   ___       ___       ___       ___       ___       ___
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// / A1  \___/ A3  \___/ A5  \___/ A7  \___/ A9  \___/ A11 \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/ B2  \___/ B4  \___/ B6  \___/ B8  \___/ B10 \___/
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// / C1  \___/ C3  \___/ C5  \___/ C7  \___/ C9  \___/ C11 \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/ D2  \___/ D4  \___/ D6  \___/ D8  \___/ D10 \___/
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// / E1  \___/ E3  \___/ E5  \___/ E7  \___/ E9  \___/ E11 \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/     \___/     \___/     \___/     \___/     \___/
    /// ```
    ///
    #[test]
    fn test_coords_flat_rows_odd() {
        let system = Coordinates::from((FlatTop, AsRows, OddColumns));

        // Check valid row/column combinations.
        assert_eq!(system.parse("A1").unwrap(), (0, 0).into());
        assert_eq!(system.parse("A3").unwrap(), (0, 2).into());
        assert_eq!(system.parse("A5").unwrap(), (0, 4).into());
        assert_eq!(system.parse("B2").unwrap(), (0, 1).into());
        assert_eq!(system.parse("B4").unwrap(), (0, 3).into());
        assert_eq!(system.parse("B6").unwrap(), (0, 5).into());
        assert_eq!(system.parse("C1").unwrap(), (1, 0).into());
        assert_eq!(system.parse("C3").unwrap(), (1, 2).into());
        assert_eq!(system.parse("C5").unwrap(), (1, 4).into());
        assert_eq!(system.parse("D2").unwrap(), (1, 1).into());
        assert_eq!(system.parse("D4").unwrap(), (1, 3).into());
        assert_eq!(system.parse("D6").unwrap(), (1, 5).into());

        assert_eq!(system.parse("Y9").unwrap(), (12, 8).into());
        assert_eq!(system.parse("Z10").unwrap(), (12, 9).into());
        assert_eq!(system.parse("AA11").unwrap(), (13, 10).into());
        assert_eq!(system.parse("AZ12").unwrap(), (25, 11).into());

        // Check invalid row/column combinations.
        assert!(system.parse("A0").is_err());
        assert!(system.parse("A2").is_err());
        assert!(system.parse("A4").is_err());
        assert!(system.parse("B1").is_err());
        assert!(system.parse("B3").is_err());
        assert!(system.parse("B5").is_err());
        assert!(system.parse("C0").is_err());
        assert!(system.parse("C2").is_err());
        assert!(system.parse("C4").is_err());
        assert!(system.parse("D1").is_err());
        assert!(system.parse("D3").is_err());
        assert!(system.parse("D5").is_err());

        assert!(system.parse("Y10").is_err());
        assert!(system.parse("Z11").is_err());
        assert!(system.parse("AA12").is_err());
        assert!(system.parse("AZ13").is_err());

        // Check that valid inputs round-trip through parse() and format().
        let inputs = [
            "A1", "A3", "A5", "B2", "B4", "B6", "C1", "C3", "C5", "D2", "D4",
            "D6", "Y9", "Z10", "AA11", "AZ12",
        ];
        for text in inputs {
            let addr = system.parse(text).unwrap();
            let out = system.format(&addr).unwrap();
            assert_eq!(text, out)
        }
    }

    /// ```text
    ///   / \   / \   / \   / \   / \   / \
    ///  /   \ /   \ /   \ /   \ /   \ /   \
    /// |     |     |     |     |     |     |
    /// | A1  | A3  | A5  | A7  | A9  | A11 |
    /// |     |     |     |     |     |     |
    ///  \   / \   / \   / \   / \   / \   / \
    ///   \ /   \ /   \ /   \ /   \ /   \ /   \
    ///    |     |     |     |     |     |     |
    ///    | B2  | B4  | B6  | B8  | B10 | B12 |
    ///    |     |     |     |     |     |     |
    ///   / \   / \   / \   / \   / \   / \   /
    ///  /   \ /   \ /   \ /   \ /   \ /   \ /
    /// |     |     |     |     |     |     |
    /// | C1  | C3  | C5  | C7  | C9  | C11 |
    /// |     |     |     |     |     |     |
    ///  \   / \   / \   / \   / \   / \   / \
    ///   \ /   \ /   \ /   \ /   \ /   \ /   \
    ///    |     |     |     |     |     |     |
    ///    | D2  | D4  | D6  | D8  | D10 | D12 |
    ///    |     |     |     |     |     |     |
    ///   / \   / \   / \   / \   / \   / \   /
    ///  /   \ /   \ /   \ /   \ /   \ /   \ /
    /// ```
    ///
    #[test]
    fn test_coords_pointed_rows_odd() {
        let system = Coordinates::from((PointedTop, AsRows, OddColumns));

        // Check valid row/column combinations.
        assert_eq!(system.parse("A1").unwrap(), (0, 0).into());
        assert_eq!(system.parse("A3").unwrap(), (0, 1).into());
        assert_eq!(system.parse("A5").unwrap(), (0, 2).into());
        assert_eq!(system.parse("B2").unwrap(), (1, 0).into());
        assert_eq!(system.parse("B4").unwrap(), (1, 1).into());
        assert_eq!(system.parse("B6").unwrap(), (1, 2).into());
        assert_eq!(system.parse("C1").unwrap(), (2, 0).into());
        assert_eq!(system.parse("C3").unwrap(), (2, 1).into());
        assert_eq!(system.parse("C5").unwrap(), (2, 2).into());
        assert_eq!(system.parse("D2").unwrap(), (3, 0).into());
        assert_eq!(system.parse("D4").unwrap(), (3, 1).into());
        assert_eq!(system.parse("D6").unwrap(), (3, 2).into());

        assert_eq!(system.parse("Y9").unwrap(), (24, 4).into());
        assert_eq!(system.parse("Z10").unwrap(), (25, 4).into());
        assert_eq!(system.parse("AA11").unwrap(), (26, 5).into());
        assert_eq!(system.parse("AZ12").unwrap(), (51, 5).into());

        // Check invalid row/column combinations.
        assert!(system.parse("A0").is_err());
        assert!(system.parse("A2").is_err());
        assert!(system.parse("A4").is_err());
        assert!(system.parse("B1").is_err());
        assert!(system.parse("B3").is_err());
        assert!(system.parse("B5").is_err());
        assert!(system.parse("C0").is_err());
        assert!(system.parse("C2").is_err());
        assert!(system.parse("C4").is_err());
        assert!(system.parse("D1").is_err());
        assert!(system.parse("D3").is_err());
        assert!(system.parse("D5").is_err());

        assert!(system.parse("Y10").is_err());
        assert!(system.parse("Z11").is_err());
        assert!(system.parse("AA12").is_err());
        assert!(system.parse("AZ13").is_err());

        // Check that valid inputs round-trip through parse() and format().
        let inputs = [
            "A1", "A3", "A5", "B2", "B4", "B6", "C1", "C3", "C5", "D2", "D4",
            "D6", "Y9", "Z10", "AA11", "AZ12",
        ];
        for text in inputs {
            let addr = system.parse(text).unwrap();
            let out = system.format(&addr).unwrap();
            assert_eq!(text, out)
        }
    }

    /// ```text
    ///   ___       ___       ___       ___       ___       ___
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// /     \___/     \___/     \___/     \___/     \___/     \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/ B1  \___/ D1  \___/ F1  \___/ H1  \___/ L1  \___/
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// / A2  \___/ C2  \___/ E2  \___/ G2  \___/ I2  \___/ K2  \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/ B3  \___/ D3  \___/ F3  \___/ H3  \___/ L3  \___/
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// / A4  \___/ C4  \___/ E4  \___/ G4  \___/ I4  \___/ K4  \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/     \___/     \___/     \___/     \___/     \___/
    /// ```
    ///
    #[test]
    fn test_coords_flat_cols_even() {
        let system = Coordinates::from((FlatTop, AsColumns, EvenColumns));

        // Check valid row/column combinations.
        assert_eq!(system.parse("A2").unwrap(), (1, 0).into());
        assert_eq!(system.parse("A4").unwrap(), (2, 0).into());
        assert_eq!(system.parse("A6").unwrap(), (3, 0).into());
        assert_eq!(system.parse("B1").unwrap(), (0, 1).into());
        assert_eq!(system.parse("B3").unwrap(), (1, 1).into());
        assert_eq!(system.parse("B5").unwrap(), (2, 1).into());
        assert_eq!(system.parse("C2").unwrap(), (1, 2).into());
        assert_eq!(system.parse("C4").unwrap(), (2, 2).into());
        assert_eq!(system.parse("C6").unwrap(), (3, 2).into());
        assert_eq!(system.parse("D1").unwrap(), (0, 3).into());
        assert_eq!(system.parse("D3").unwrap(), (1, 3).into());
        assert_eq!(system.parse("D5").unwrap(), (2, 3).into());

        assert_eq!(system.parse("Y8").unwrap(), (4, 24).into());
        assert_eq!(system.parse("Z9").unwrap(), (4, 25).into());
        assert_eq!(system.parse("AA10").unwrap(), (5, 26).into());
        assert_eq!(system.parse("AZ11").unwrap(), (5, 51).into());

        // Check invalid row/column combinations.
        assert!(system.parse("A1").is_err());
        assert!(system.parse("A3").is_err());
        assert!(system.parse("A5").is_err());
        assert!(system.parse("B2").is_err());
        assert!(system.parse("B4").is_err());
        assert!(system.parse("B6").is_err());
        assert!(system.parse("C1").is_err());
        assert!(system.parse("C3").is_err());
        assert!(system.parse("C5").is_err());
        assert!(system.parse("D2").is_err());
        assert!(system.parse("D4").is_err());
        assert!(system.parse("D6").is_err());

        assert!(system.parse("Y9").is_err());
        assert!(system.parse("Z10").is_err());
        assert!(system.parse("AA11").is_err());
        assert!(system.parse("AZ12").is_err());

        // Check that valid inputs round-trip through parse() and format().
        let inputs = [
            "A2", "A4", "A6", "B1", "B3", "B5", "C2", "C4", "C6", "D1", "D3",
            "D5", "Y8", "Z9", "AA10", "AZ11",
        ];
        for text in inputs {
            let addr = system.parse(text).unwrap();
            let out = system.format(&addr).unwrap();
            assert_eq!(text, out)
        }
    }

    /// ```text
    ///   / \   / \   / \   / \   / \   / \
    ///  /   \ /   \ /   \ /   \ /   \ /   \
    /// |     |     |     |     |     |     |
    /// |     |     |     |     |     |     |
    /// |     |     |     |     |     |     |
    ///  \   / \   / \   / \   / \   / \   / \
    ///   \ /   \ /   \ /   \ /   \ /   \ /   \
    ///    |     |     |     |     |     |     |
    ///    | B1  | D1  | F1  | H1  | J1  | L1  |
    ///    |     |     |     |     |     |     |
    ///   / \   / \   / \   / \   / \   / \   /
    ///  /   \ /   \ /   \ /   \ /   \ /   \ /
    /// |     |     |     |     |     |     |
    /// | A2  | C2  | E2  | G2  | I2  | K2 |
    /// |     |     |     |     |     |     |
    ///  \   / \   / \   / \   / \   / \   / \
    ///   \ /   \ /   \ /   \ /   \ /   \ /   \
    ///    |     |     |     |     |     |     |
    ///    | B3  | D3  | F3  | H3  | J3  | L3  |
    ///    |     |     |     |     |     |     |
    ///   / \   / \   / \   / \   / \   / \   /
    ///  /   \ /   \ /   \ /   \ /   \ /   \ /
    /// ```
    ///
    #[test]
    fn test_coords_pointed_cols_even() {
        let system = Coordinates::from((PointedTop, AsColumns, EvenColumns));

        // Check valid row/column combinations.
        assert_eq!(system.parse("A2").unwrap(), (2, 0).into());
        assert_eq!(system.parse("A4").unwrap(), (4, 0).into());
        assert_eq!(system.parse("A6").unwrap(), (6, 0).into());
        assert_eq!(system.parse("B1").unwrap(), (1, 0).into());
        assert_eq!(system.parse("B3").unwrap(), (3, 0).into());
        assert_eq!(system.parse("B5").unwrap(), (5, 0).into());
        assert_eq!(system.parse("C2").unwrap(), (2, 1).into());
        assert_eq!(system.parse("C4").unwrap(), (4, 1).into());
        assert_eq!(system.parse("C6").unwrap(), (6, 1).into());
        assert_eq!(system.parse("D1").unwrap(), (1, 1).into());
        assert_eq!(system.parse("D3").unwrap(), (3, 1).into());
        assert_eq!(system.parse("D5").unwrap(), (5, 1).into());

        assert_eq!(system.parse("Y8").unwrap(), (8, 12).into());
        assert_eq!(system.parse("Z9").unwrap(), (9, 12).into());
        assert_eq!(system.parse("AA10").unwrap(), (10, 13).into());
        assert_eq!(system.parse("AZ11").unwrap(), (11, 25).into());

        // Check invalid row/column combinations.
        assert!(system.parse("A1").is_err());
        assert!(system.parse("A3").is_err());
        assert!(system.parse("A5").is_err());
        assert!(system.parse("B2").is_err());
        assert!(system.parse("B4").is_err());
        assert!(system.parse("B6").is_err());
        assert!(system.parse("C1").is_err());
        assert!(system.parse("C3").is_err());
        assert!(system.parse("C5").is_err());
        assert!(system.parse("D2").is_err());
        assert!(system.parse("D4").is_err());
        assert!(system.parse("D6").is_err());

        assert!(system.parse("Y9").is_err());
        assert!(system.parse("Z10").is_err());
        assert!(system.parse("AA11").is_err());
        assert!(system.parse("AZ12").is_err());

        // Check that valid inputs round-trip through parse() and format().
        let inputs = [
            "A2", "A4", "A6", "B1", "B3", "B5", "C2", "C4", "C6", "D1", "D3",
            "D5", "Y8", "Z9", "AA10", "AZ11",
        ];
        for text in inputs {
            let addr = system.parse(text).unwrap();
            let out = system.format(&addr).unwrap();
            assert_eq!(text, out)
        }
    }

    /// ```text
    ///   ___       ___       ___       ___       ___       ___
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// /     \___/     \___/     \___/     \___/     \___/     \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/ A2  \___/ A4  \___/ A6  \___/ A8  \___/ A10 \___/
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// / B1  \___/ B3  \___/ B5  \___/ B7  \___/ B9  \___/ B11 \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/ C2  \___/ C4  \___/ C6  \___/ C8  \___/ C10 \___/
    ///  /   \     /   \     /   \     /   \     /   \     /   \
    /// / D1  \___/ D3  \___/ D5  \___/ D7  \___/ D9  \___/ D11 \
    /// \     /   \     /   \     /   \     /   \     /   \     /
    ///  \___/     \___/     \___/     \___/     \___/     \___/
    /// ```
    ///
    #[test]
    fn test_coords_flat_rows_even() {
        let system = Coordinates::from((FlatTop, AsRows, EvenColumns));

        // Check valid row/column combinations.
        assert_eq!(system.parse("A2").unwrap(), (0, 1).into());
        assert_eq!(system.parse("A4").unwrap(), (0, 3).into());
        assert_eq!(system.parse("A6").unwrap(), (0, 5).into());
        assert_eq!(system.parse("B1").unwrap(), (1, 0).into());
        assert_eq!(system.parse("B3").unwrap(), (1, 2).into());
        assert_eq!(system.parse("B5").unwrap(), (1, 4).into());
        assert_eq!(system.parse("C2").unwrap(), (1, 1).into());
        assert_eq!(system.parse("C4").unwrap(), (1, 3).into());
        assert_eq!(system.parse("C6").unwrap(), (1, 5).into());
        assert_eq!(system.parse("D1").unwrap(), (2, 0).into());
        assert_eq!(system.parse("D3").unwrap(), (2, 2).into());
        assert_eq!(system.parse("D5").unwrap(), (2, 4).into());

        assert_eq!(system.parse("Y8").unwrap(), (12, 7).into());
        assert_eq!(system.parse("Z9").unwrap(), (13, 8).into());
        assert_eq!(system.parse("AA10").unwrap(), (13, 9).into());
        assert_eq!(system.parse("AZ11").unwrap(), (26, 10).into());

        // Check invalid row/column combinations.
        assert!(system.parse("A1").is_err());
        assert!(system.parse("A3").is_err());
        assert!(system.parse("A5").is_err());
        assert!(system.parse("B2").is_err());
        assert!(system.parse("B4").is_err());
        assert!(system.parse("B6").is_err());
        assert!(system.parse("C1").is_err());
        assert!(system.parse("C3").is_err());
        assert!(system.parse("C5").is_err());
        assert!(system.parse("D2").is_err());
        assert!(system.parse("D4").is_err());
        assert!(system.parse("D6").is_err());

        assert!(system.parse("Y9").is_err());
        assert!(system.parse("Z10").is_err());
        assert!(system.parse("AA11").is_err());
        assert!(system.parse("AZ12").is_err());

        // Check that valid inputs round-trip through parse() and format().
        let inputs = [
            "A2", "A4", "A6", "B1", "B3", "B5", "C2", "C4", "C6", "D1", "D3",
            "D5", "Y8", "Z9", "AA10", "AZ11",
        ];
        for text in inputs {
            let addr = system.parse(text).unwrap();
            let out = system.format(&addr).unwrap();
            assert_eq!(text, out)
        }
    }

    /// ```text
    ///   / \   / \   / \   / \   / \   / \
    ///  /   \ /   \ /   \ /   \ /   \ /   \
    /// |     |     |     |     |     |     |
    /// |     |     |     |     |     |     |
    /// |     |     |     |     |     |     |
    ///  \   / \   / \   / \   / \   / \   / \
    ///   \ /   \ /   \ /   \ /   \ /   \ /   \
    ///    |     |     |     |     |     |     |
    ///    | A2  | A4  | A6  | A8  | A10 | A12 |
    ///    |     |     |     |     |     |     |
    ///   / \   / \   / \   / \   / \   / \   /
    ///  /   \ /   \ /   \ /   \ /   \ /   \ /
    /// |     |     |     |     |     |     |
    /// | B1  | B3  | B5  | B7  | B9  | B11 |
    /// |     |     |     |     |     |     |
    ///  \   / \   / \   / \   / \   / \   / \
    ///   \ /   \ /   \ /   \ /   \ /   \ /   \
    ///    |     |     |     |     |     |     |
    ///    | C2  | C4  | C6  | C8  | C10 | C12 |
    ///    |     |     |     |     |     |     |
    ///   / \   / \   / \   / \   / \   / \   /
    ///  /   \ /   \ /   \ /   \ /   \ /   \ /
    /// ```
    ///
    #[test]
    fn test_coords_pointed_rows_even() {
        let system = Coordinates::from((PointedTop, AsRows, EvenColumns));

        // Check valid row/column combinations.
        assert_eq!(system.parse("A2").unwrap(), (1, 0).into());
        assert_eq!(system.parse("A4").unwrap(), (1, 1).into());
        assert_eq!(system.parse("A6").unwrap(), (1, 2).into());
        assert_eq!(system.parse("B1").unwrap(), (2, 0).into());
        assert_eq!(system.parse("B3").unwrap(), (2, 1).into());
        assert_eq!(system.parse("B5").unwrap(), (2, 2).into());
        assert_eq!(system.parse("C2").unwrap(), (3, 0).into());
        assert_eq!(system.parse("C4").unwrap(), (3, 1).into());
        assert_eq!(system.parse("C6").unwrap(), (3, 2).into());
        assert_eq!(system.parse("D1").unwrap(), (4, 0).into());
        assert_eq!(system.parse("D3").unwrap(), (4, 1).into());
        assert_eq!(system.parse("D5").unwrap(), (4, 2).into());

        assert_eq!(system.parse("Y8").unwrap(), (25, 3).into());
        assert_eq!(system.parse("Z9").unwrap(), (26, 4).into());
        assert_eq!(system.parse("AA10").unwrap(), (27, 4).into());
        assert_eq!(system.parse("AZ11").unwrap(), (52, 5).into());

        // Check invalid row/column combinations.
        assert!(system.parse("A1").is_err());
        assert!(system.parse("A3").is_err());
        assert!(system.parse("A5").is_err());
        assert!(system.parse("B2").is_err());
        assert!(system.parse("B4").is_err());
        assert!(system.parse("B6").is_err());
        assert!(system.parse("C1").is_err());
        assert!(system.parse("C3").is_err());
        assert!(system.parse("C5").is_err());
        assert!(system.parse("D2").is_err());
        assert!(system.parse("D4").is_err());
        assert!(system.parse("D6").is_err());

        assert!(system.parse("Y9").is_err());
        assert!(system.parse("Z10").is_err());
        assert!(system.parse("AA11").is_err());
        assert!(system.parse("AZ12").is_err());

        // Check that valid inputs round-trip through parse() and format().
        let inputs = [
            "A2", "A4", "A6", "B1", "B3", "B5", "C2", "C4", "C6", "D1", "D3",
            "D5", "Y8", "Z9", "AA10", "AZ11",
        ];
        for text in inputs {
            let addr = system.parse(text).unwrap();
            let out = system.format(&addr).unwrap();
            assert_eq!(text, out)
        }
    }

    /// Tests that logical row numbers are consistent when the input row
    /// and/or column number is negative.
    #[test]
    fn test_logical_row_numbers() {
        // NOTE: this corresponds to "A1".
        let origin = HexAddress::from((0, 0));
        let orig_row = origin.logical_row();
        let orig_col = origin.logical_column();
        assert_eq!(orig_row, 0);
        assert_eq!(orig_col, 0);

        // Returns the column number, relative to the origin.
        let dcol = |addr: &HexAddress| addr.logical_column() - orig_col;

        // Returns the row number, relative to the origin, accounting for the
        // alternating up/down sequence along each row.
        let drow = |addr: &HexAddress, odd_column: bool| {
            let dr = addr.logical_row() - orig_row;
            if odd_column {
                // Odd column: hexes are one row below hexes in even rows.
                // Subtract one to shift this value up one row, to match the
                // row number for even columns.
                dr - 1
            } else {
                dr
            }
        };

        // Check that negating the column and/or row number used to construct
        // a HexAddress does not change the distance from the origin, after
        // accounting for the effect of the column number on the row number.
        let compare_addrs =
            |row: isize, col: isize, neg_row: bool, neg_col: bool| {
                let row_2 = if neg_row { -row } else { row };
                let col_2 = if neg_col { -col } else { col };
                let addr_1 = HexAddress::from((row, col));
                let addr_2 = HexAddress::from((row_2, col_2));
                let odd_column = col % 2 != 0;
                let dc_1 = dcol(&addr_1);
                let dr_1 = drow(&addr_1, odd_column);
                let dc_2 = dcol(&addr_2);
                let dr_2 = drow(&addr_2, odd_column);
                if neg_row {
                    assert_eq!(dr_1, -dr_2)
                } else {
                    assert_eq!(dr_1, dr_2)
                }
                if neg_col {
                    assert_eq!(dc_1, -dc_2)
                } else {
                    assert_eq!(dc_1, dc_2)
                }
            };

        // Check hexes in the 7x7 grid centred at the origin.
        let vals: Vec<isize> = vec![0, 1, 2, 3];
        for &row in &vals {
            for &col in &vals {
                // Compare the row and column numbers of the corresponding
                // HexAddress in each quadrant of this grid.
                compare_addrs(row, col, false, false);
                compare_addrs(row, col, true, false);
                compare_addrs(row, col, false, true);
                compare_addrs(row, col, true, true);
            }
        }
    }
}
