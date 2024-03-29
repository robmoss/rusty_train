# Performance

We will use the final operating round of the Bankruptcy Club's recorded game of 1867 as an example, and consider the following three companies:

- Canadian Northern Railway (CNR), which has a 5-train and a 5+5E-train, and runs for $102 per share ([link](https://youtu.be/vE0UNDA4qQQ?t=13365));
- Great Western Railway (GW), which has a 5-train and an 8-train, and runs for $76 per share ([link](https://youtu.be/vE0UNDA4qQQ?t=13580)); and
- Chesapeake and Ohio Railway (C&O), which has a 6-train and an 8-train, and runs for $89 per share ([link](https://youtu.be/vE0UNDA4qQQ?t=14010), [final decision](https://youtu.be/vE0UNDA4qQQ?t=14470)).

For all three companies, the ``navig18xx`` crate allows us to find better routes that earn more revenue.

- Canadian Northern Railway can earn $113 per share, an increase of $11.
- Great Western Railway can earn $84 per share, an increase of $8.
- Chesapeake and Ohio Railway can earn $90 per share, an increase of $1.

Each company also serves as a good benchmark for measuring performance.
They can operate tens of thousands of paths, and with 2 trains this results in hundreds of millions to billions of potential path combinations:

| Company | Number of paths | Number of path combinations |
|---------|----------------:|----------------------------:|
| GW      |          15,008 |                 112,612,528 |
| C&O     |          46,176 |               1,066,088,400 |
| CNR     |          67,948 |               2,308,431,378 |

Profiling revealed that the overwhelming majority of time was being spent determining whether each combination of paths could be operated together (i.e., checking for route conflicts).

The following optimisations have been introduced:

- **Record fewer conflicts:**
  - We do not need to record track segment conflicts, since every track segment connects to a hex face, and two track segments that connect to the same hex face are considered to share track.
  - We only need to record one hex face conflict for each pair of adjacent hex faces.
- **Sort conflicts:** store route conflicts in sorted vectors, to minimise the number of comparisons required to identify whether two paths conflict.
- **Parallel iterator:** iterate over the huge numbers of path combinations in parallel using [rayon](https://github.com/rayon-rs/rayon).
- **B-Trees:** ensure [deterministic results](./determinism.md) by using B-Trees instead of hashed data structures.

|                   |      GW |     C&O |     CNR |
|-------------------|--------:|--------:|--------:|
| Initial           |    0:37 |    5:23 |   13:02 |
| Fewer conflicts   |    0:22 |    4:08 |    9:35 |
| Sorted conflicts  |    0:12 |    1:42 |    4:58 |
| Parallel iterator |    0:06 |    0:51 |    2:26 |
| B-Trees           |    0:05 |    0:45 |    2:01 |
| **Improvement:**  | **86%** | **86%** | **85%** |

These times were obtained by running `cargo test --release 1867_bc -- --include-ignored` using Rust 1.48.0 on Debian Buster (Linux kernel 5.10.28) with 8 GB RAM and an Intel Core i7-5600U CPU (2 cores, 4 MB cache).
The times reported for the **B-Trees** optimisation were obtained using Rust 1.54.0 and Linux kernel 5.10.46, but these software updates did not change the times obtained for the **Parallel iterator** optimisation.
