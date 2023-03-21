# Changelog

## Rusty Train 0.2.0 (2023-MM-DD)

- Add support for 1889: History of Shikoku Railways (Shikoku 1889).

- Add support for train-specific route bonuses (`VisitWithTrainBonus`).

- Migrate to GTK 4.

## Rusty Train 0.1.0 (2021-10-08)

Initial release.

### Supported games

Maps, tiles, and trains for the following games are implemented:

- 1830: Railways and Robber Barons
- 1861: The Railways of the Russian Empire
- 1867: The Railways of Canada

### Known issues

- Documentation is incomplete.

- Most errors result in panics, rather than returning `Result<T,E>` values.
