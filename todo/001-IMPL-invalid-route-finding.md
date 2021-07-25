## Invalid route-finding options

The route-finding algorithm assumes that routes can be constructed from one or more paths, where each path starts at a token T1 and never proceeds past a token T2 where T2 > T1, and we join pairs of paths that both start from the same token T and have no conflicts.

The path-building algorithm and route-finding algorithm both assume that a single path cannot pass through the same revenue centre more than once.
Note that the path-building algorithm stops whenever it encounters any kind of connection (hex face, track segment, revenue centre) that it has already visited.

The search criteria (`n18route::search::Criteria`) cannot allow `conflict_rule` to be `ConflictRule::TrackOnly` (i.e., no track segment in common), because this would mean that a single route could visit the same revenue centre multiple times, if there are sufficient track connections.

- So the implementation should panic, or return an `Error` value.

- Are there any games where this is a relevant concern?
  Note that this does not apply to "Flood" trains, which earn revenue from every revenue centre that can be reached from a single token (i.e., only requires a search from each matching token, and selecting the token that earns the most revenue).

See the `n18route::search` and `n18route::train` modules for the implementation.
