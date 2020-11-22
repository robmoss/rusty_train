/*!
Explains how we find the best set of routes.

# Overview

Before describing how routes are constructed and evaluated, the following
distinction must be made:

+ A **path** defines the cities, dits, and track segments that a single train
  may pass through; and

+ A **route** defines the stops that a single train makes along a given path,
  and the revenue that is earned by doing so.

# How paths are constructed

1. For a given company, all paths must pass through a city that contains one
   of the company's token.

2. For each of the company's tokens on the map, we first construct all of the
   valid paths that start from the token's location.

3. To allow for paths that pass through a token, rather than starting at the
   token, we join pairs of paths  that start at the token to form a new path,
   ensuring that the two paths do not have any conflicts (i.e., they don't
   have any elements in common except for the token's location) and that the
   resulting path respects any constraints on length, number of cities, etc.

4. We avoid duplicating paths that pass through multiple tokens by defining
   an ordering on token spaces (i.e., we derive
   [``Ord``](https://doc.rust-lang.org/std/cmp/trait.Ord.html) for
   [``HexAddress``](n18map::map::HexAddress) and this gives
   us an ordering for ``(HexAddress, usize)`` tuples).
   We only construct paths that connect token spaces in increasing order.

5. Since express trains may skip over any number of cities, we construct
   paths of arbitrary length, where the express train **must** stop at the
   path's start and end locations, and may skip over any number of cities in
   between.
   Note that this means that, depending on the type of train that is being
   run, a path represents either:

   + A single route across the track network; or

   + Multiple routes that traverse this path but stop at different locations.

This is implemented by
[`paths_for_token`](crate::search::paths_for_token).

# How route combinations are evaluated

**NOTE:** support for route bonuses is not yet implemented.
Optimal pairings of trains to routes **without** considering route bonuses is
implemented by
[`Trains::select_routes`](crate::train::Trains::select_routes).

Once we have collected all of the possible paths for a company, we need to
find the allocation of trains to routes that yields the greatest revenue.
There are a number of complications to consider:

1. For a given set of routes, the revenue may depend on how we allocate these
   routes to the company's trains.
   For example, if there are two routes that visit exactly two cities and the
   company owns a `2` train **and** a `2+2` train, the `2+2` train should run
   on the route that earns the greatest revenue.

2. We need to consider operating fewer routes than the company has trains.

3. For express trains, we must consider routes of all possible lengths, and
   determine the combination of visiting and skipping cities along each route
   that earns the greatest revenue.

   + So for an express train that can make up to `N` stops, it must stop at
     the first and last stops on the path, and up to `N - 2` stops anywhere
     else along the path.

   + Note that route bonuses may affect which of the `N - 2` stops earn the
     most revenue, so we need to evaluate every combination of stopping and
     and skipping ... **unless** we can make stronger assumptions about the
     nature of the route bonuses?

4. Routes may earn bonus revenue from a variety of sources, such as:

   + By owning private companies that provide bonus revenue when visiting a
     specific location.

   + By visiting a specific combination of cities.
     For example, in 1867 the city of Timmins normally earns $40, but if the
     route also includes at least one of Toronto, Montréal, or Québec, its
     revenue is doubled ($80).

   These bonuses are game-specific and context-dependent.
   So rather than hard-coding them into the route-finding logic, it would be
   much nicer to accept an
   [immutable closure](https://doc.rust-lang.org/std/ops/trait.Fn.html) that
   returns the bonus revenue (if any) for a given pairing of train type and
   route.
   For example, this could be provided as ``F: Fn(Train, Route) -> usize`` or
   ``F: Fn(Train, Route) -> Option<usize>``.

5. Where a company owns trains of multiple types, we may either want to
   generate all routes at once (i.e., the most permissive set of routes) or
   instead generate routes specific to each train type (noting that there may
   be a substantial overlap in these routes, such as for a `2` train and a `3`
   train).
   If we allow each train type to define its own search criteria, we can
   either:

   1. Combine these criteria to construct the most general criteria and
      enumerate all possible routes for all of the train types with one call
      to ``paths_for_token``; or

   2. Call ``paths_for_token`` multiple times, once for each train type.
      This will likely result in many duplicate routes, but should also
      simplify the process of iterating over route combinations and allocating
      the company's trains for each of these combinations.

      In this case, we'd need to decide how much of the logic should be
      implemented by each train type, and how much by the generic
      route-finding module ([`search`](crate::search)).

      At this stage, I *think* it would be preferable for ``route::search`` to
      construct Paths, and have each Train implement a method along the lines
      of ``Train::best_route(path: &Path) -> Option(Route)``.
      Then the responsibility of (a) selecting which places to stop at and
      which places to skip; and (b) applying route bonuses; to the ``Train``.

# Train types

  There are many different train types in the 18xx family of board games; see
  [this discussion thread](https://boardgamegeek.com/thread/1305250/) and
  these
  [rule differences](http://www.fwtwr.com/18xx/rules_difference_list/7.htm)
  for details.

  For now I intend to focus on the trains from 1861 and 1867, and then
  consider adding trains from 1830 and/or 1889.
*/
