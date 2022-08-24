<div align="center">
  <h1><code>csv-ledger</code></h1>
  <strong>
    A project to learn zero-copy parsing and improve my upon my Rust performance profiling and coverage tooling knowledge.
  </strong>
</div>

*Please note: this crate requires `nightly` toolchain as it makes use of let chains.*

## Installation

```sh
cargo install csv-ledger
```

## Usage

**Print output to console:**

```sh
csv-ledger foo.csv
```

**Save output to file:**
```sh
csv-ledger --output output.csv foo.csv
```

**To see helpful infomation:**

```sh
csv-ledger --help
```

## Assumptions


## Implementation Details



## Code Coverage

This project has a 100% code-coverage. Whilst Rust provides first-class error checking, it cannot protect against logic errors. With strong test coverage in combination with Rust's error checking, you can have a high degree of confidence of correctness.

### To run test coverage locally

**Setup:**

```sh
rustup component add llvm-tools-preview &&
cargo install cargo-llvm-cov
```

**Usage:**

```sh
cargo llvm-cov
```

## QA


