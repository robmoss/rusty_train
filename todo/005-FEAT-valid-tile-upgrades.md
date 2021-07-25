## Valid tile upgrades

Across most (all?) 18xx games there are [three different rules for upgrading track](https://www.railsonboards.com/2020/12/26/permissive-restrictive-semi-restrictive-what-it-means/):

- Permissive;
- Semi-Restrictive; and
- Restrictive.

We could define an `UpgradeRule` enum with three variants.
This would allow us to:

1. Identify candidate upgrade tiles (noting that candidates may not be valid for all possible tile rotations);

2. Validate the chosen tile and rotation; and

3. Place each token from the original tile in an appropriate token space.

**NOTE:** the valid candidates may depend on which company is performing the upgrade, so candidate selection and validation will both require passing a valid `Token` to identify the company.

Are there any situations where the `Game` itself should have some say in choosing candidates and/or validating the chosen replacement?

- Given that some 18xx games include events that alter the map, such as the North-West Rebellion in 1882: Assiniboia, it is probably best to provide a default implementation (e.g., as a default method for `n18game::Game`) and allow individual games to override this as required.

- Or should we require each game to define the valid upgrade tiles, and only provide a default implementation for validation?
  This method could look something like:

  ```rust
  fn upgrades_for(&self, tile_name: &str) -> Option<Vec<&str>>
  ```

- Games can then hard-code the upgrade options, and we only need to implement the validation logic and token placement for each of the `UpgradeRule` variants.

### Representation

We would need to define a data structure that characterises the current tile's topology and connections with respect to the company performing the upgrade.

### UI states

The current `ReplaceTile` state should be separated into two states:

- A `ReplaceTile` state that only handles replacements, not upgrades (i.e., ignores placed tokens and upgrade concerns); and

- An `UpgradeTile` state that describes the current tile's properties (see above) and only accepts valid upgrades.
  Whenever the candidate tile is changed or rotated, the hex border colour could be used to indicate whether the current configuration is a valid upgrade.
