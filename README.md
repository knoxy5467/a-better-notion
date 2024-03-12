# a-better-notion

Made in rust.

make sure you have rust installed for your specific platform (see: https://rustup.rs/)

Run client:

```
cargo run -p client
```

Run server:

```
cargo run -p server
```

Run all tests and generate coverage reports

```
cargo llvm-cov --lcov --output-path lcov.info
```

To format all of your rust code

```
cargo fmt
```

# Pull requests

Pull requests should have 100% coverage for tests and should be formatted and have no warnings from linting.

therefore before creating a pull request please run these commands.

Run all tests and generate coverage reports

```
cargo llvm-cov --lcov --output-path lcov.info
```

To format all of your rust code

```
cargo fmt
```
