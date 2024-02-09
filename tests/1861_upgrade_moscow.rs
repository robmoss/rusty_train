use std::collections::BTreeMap;

use navig18xx::map::map::try_placing_tokens;
use navig18xx::prelude::*;
use navig18xx::tile::TokenSpace;

#[test]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use RotateCW::*;

    let game = navig18xx::game::new_1861();
    let cat = game.catalogue();

    // This tile has 3 single-token cities;
    // - The first is connected to the bottom hex face;
    // - The second is connected to the upper-left hex face; and
    // - The third is connected to the upper-right hex face.
    let moscow = cat.tile("Moscow").unwrap();
    let conns_moscow =
        [HexFace::Bottom, HexFace::UpperLeft, HexFace::UpperRight];

    // This tile has 3 single-token cities:
    // - The first is connected to the bottom and lower-left faces;
    // - The second is connected to the upper-left and top faces; and
    // - The third is connected to the upper-right and lower-right faces.
    let tile_637 = cat.tile("637").unwrap();

    let token_names = ["KB", "KK", "KR"];
    let tok_kb = game.token("KB");
    let tok_kk = game.token("KK");
    let tok_kr = game.token("KR");
    let placed_tokens = vec![*tok_kb, *tok_kk, *tok_kr];
    let token_spaces = moscow.token_spaces();

    assert_eq!(token_spaces.len(), 3);
    let tokens_table: BTreeMap<TokenSpace, Token> =
        token_spaces.into_iter().zip(placed_tokens).collect();

    let rotns: Vec<RotateCW> = vec![Zero, One, Two, Three, Four, Five];
    for orig_rotn in &rotns {
        // Identify the face that each token, in turn, must be connected to.
        let need_face: Vec<HexFace> =
            conns_moscow.iter().map(|&face| face + orig_rotn).collect();

        for new_rotn in &rotns {
            let new_tokens = try_placing_tokens(
                moscow,
                orig_rotn,
                &tokens_table,
                tile_637,
                new_rotn,
            );
            // Check that the three tokens were successfully placed.
            assert!(new_tokens.is_some());
            let new_tokens = new_tokens.unwrap();
            assert_eq!(new_tokens.len(), 3);

            for (new_tok_space, token) in new_tokens.iter() {
                // Ensure that each placed token matches an original token.
                let token_ix =
                    token_names.iter().enumerate().find_map(|(ix, name)| {
                        if game.token(name) == token {
                            Some(ix)
                        } else {
                            None
                        }
                    });
                assert!(token_ix.is_some());
                let token_ix = token_ix.unwrap();

                // Check that the placed token is connected to the required face.
                let start = Connection::City {
                    ix: new_tok_space.city_ix(),
                };
                let faces: std::collections::BTreeSet<HexFace> = tile_637
                    .connected_faces(start)
                    .into_iter()
                    .map(|face| face + new_rotn)
                    .collect();
                assert!(faces.contains(&need_face[token_ix]));
            }
        }
    }

    Ok(())
}
