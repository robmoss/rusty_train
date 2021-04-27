# Profiling

The [Rust Performance Book](https://nnethercote.github.io/perf-book/) lists a number of different [profilers](https://nnethercote.github.io/perf-book/profiling.html).
I have used [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) and [perf](https://perf.wiki.kernel.org/index.php/Main_Page) to profile `navig18xx`.
This allowed me to identify that when finding the optimal pairing of trains to routes (`Trains::select_routes`) the majority of time was being spent determining whether pairs of paths had any conflicts or could be operated together.
This analysis can be reproduced with the following steps:

1. Install the necessary packages; e.g., on Debian Linux:

   ```sh
   sudo apt install linux-perf
   cargo install flamegraph
   ```

2. Delete the cached routes so that the `1867_bc` example will have to identify them:

   ```sh
   rm ./examples/output/1867_bc_*.json
   ```

3. Profile the `1867_bc` example and generate a flamegraph:

   ```sh
   cargo flamegraph --example 1867_bc --output flamegraph-1867_bc.svg
   ```

## Collecting profiling information

Note that you may need to temporarily decrease the value of `perf_event_paranoid` in order to collect profiling information.
You should then restore its original value.

```sh
echo 2 | sudo tee /proc/sys/kernel/perf_event_paranoid
cargo flamegraph --example 1867_bc --output flamegraph-1867_bc.svg
echo 3 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

## Improved output for release builds

If profiling a release build, you may want to ensure that full debugging information is collected so that the profiling output is easier to interpret.
To do this, either set the environment variable `CARGO_PROFILE_RELEASE_DEBUG=true` or *temporarily* add the following lines to `Cargo.toml`:

```toml
[profile.release]
debug = true
```
