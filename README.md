# ABN: A Better Notion

Made in rust.

Make sure you have rust installed for your specific platform (see: https://rustup.rs/)

Run client:

```
cargo run -p client
```

Run server:

```
cargo run -p server
```

to read the documentation of a crate, run:

```
cargo rustdoc -p <crate_name> --open
```

currently we have 3 crates: `client`, `common`, and `server`.

# Contributing

Pull requests should have 100% coverage for tests and should be formatted and have no warnings from linting.

Run clippy and generate coverage reports for tests:

```
zsh pr-checks.sh
```

Please also format your rust code before submitting a PR:

```
cargo fmt
```
