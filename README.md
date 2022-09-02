<div align="center">
  <h1><code>csv_ledger</code></h1>
  <strong>
    A project to learn zero-copy parsing and improve my Rust performance profiling and coverage tooling knowledge.
  </strong>
</div>
<br><br>

## 🛠 Installation

```sh
cargo install csv_ledger
```

## 🔋 Usage

**Print output to console:**

```sh
csv_ledger foo.csv
```

**Save output to file:**
```sh
csv_ledger --output output.csv foo.csv
```

**To see helpful information:**

```sh
csv_ledger --help
```

## 📚 Documentation

Further documentation can be found [here](https://docs.rs/csv_ledger).

## 🔬 Test using `cargo test`

```sh
cargo test --features test_args
```

## 📝 Code Coverage

This project aimed to have [a near 100% code-coverage](https://unazoomer.net/csv-ledger/coverage/html). Whilst Rust provides first-class error checking, it cannot easily protect against logic errors. With strong test coverage in combination with Rust's error checking, you can have a high degree of confidence. However, I have found that getting to 100% coverage can be very difficult whilst using `llvm-cov`. LLVM's coverage tooling is far more precise than other coverage tools that I have worked with in the past (such as Jest), requiring all lines, branches, derived traits and implementations to be covered.

A pre-generated coverage report can be found in: [`/coverage/html`](https://unazoomer.net/csv-ledger/coverage/html).


### Run Coverage Locally

**Setup**

```sh
rustup component add llvm-tools-preview &&
cargo install cargo-llvm-cov
```

**Usage**

To create a coverage report:

```sh
cargo llvm-cov --features test_args
```

To debug a coverage report:

```sh
cargo llvm-cov --features test_args --html --output-dir coverage
```