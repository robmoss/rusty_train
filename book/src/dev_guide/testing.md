# Testing

Run **most** of the test cases (unit tests, documentation tests, integration tests, and examples) with the following command:

```shell
cargo test --all-targets
```

This will skip **ignored** tests, such as the `1867_bc` example which can take several minutes to run.
Run these ignored tests with the following command:

```shell
cargo test --all-targets -- --ignored
```

As of Rust 1.51, you can run **all** tests by passing `--include-ignored` to the test binaries:

```shell
cargo test --all-targets -- --include-ignored
```

**Note:** you may want to build the ignored tests in release mode (i.e., with optimisations enabled) so that they take less time to run.

## Comparing output images

Compare changed output images by making a copy of the original image and identifying changed pixels in red:

```shell
git show HEAD:path/to/image.png > original_image.png
compare -compose src original_image.png path/to/image.png diff.png
```

Alternatively, you can use the provided `img-diff.sh` script:

```shell
./img-diff.sh path/to/image.png
```
