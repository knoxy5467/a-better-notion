cargo llvm-cov --all-features --workspace --lcov --output-path target/lcov.info &&
genhtml target/lcov.info --output-directory target/coverage_report &&
echo "Opening... $(pwd)/target/coverage_report/index.html"
xdg-open target/coverage_report/index.html