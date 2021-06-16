# The problem

What combination of train routes will earn the most revenue for a company?
And why is this a difficult question to answer?

+ The number of valid train routes can be extremely large.

  + As the maximum route length grows (e.g., running a 3-train instead of a 2-train) the number of valid train routes can grow exponentially.

+ A valid train route must pass through (at least) one of the company's station markers.

  + Train routes do not necessarily start or end at a station marker.

  + Train routes can be constructed by first generating all routes that start at each station marker, and then joining pairs of routes that start or end at the same station marker.

  + Note that this approach will generate duplicate routes when a single route can reach or pass through multiple station markers (but this duplication can be avoided by defining an ordering for the station markers).

+ When a company owns multiple trains, all valid combinations of train routes must be considered.

  + The highest-earning route for each train will typically share track segments, and cannot be operated at the same time.
    So this is not as simple as finding the highest-earning route for each train.

+ The number of route combinations can be extremely large.

  + Consider a company that has 50,000 valid train routes.
    If the company owns 2 trains, there are **1,249,975,000** route combinations.
    If the company owns 3 trains, there are **20,832,083,350,000** routes combinations.

+ For each route combination, we need to determine whether it is valid (i.e., the routes can be operated without any conflicts).
  This is currently [the most time-consuming step](./dev_guide/performance.md).

+ For each valid route combination, the earned revenue may depend on how the company's trains are allocated to the routes.

+ For each valid route combination, there may be additional bonus revenue.

  + The company might own a private company that confers bonus revenue.

  + The game rules might confer bonus revenue to routes that connect two specific locations.

+ Each step can be divided into a number of independent tasks and run in parallel, although this only provides a modest performance benefit (a linear reduction in exponential growth).

See the [optimal routes](./dev_guide/routes.md) chapter of the Developer guide for further details.
