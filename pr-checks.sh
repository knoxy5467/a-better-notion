cargo clippy
cargo llvm-cov --all-features --workspace --lcov --output-path target/lcov.info && lcov --summary target/lcov.info