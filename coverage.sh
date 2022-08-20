# Requires:
# cargo install cargo-binutils && rustup component add llvm-tools-preview


RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="json5format-%m.profraw" cargo test --tests && { 
    rust-profdata merge -sparse=true json5format-*.profraw -o json5format.profdata &&
    rust-cov show ./json5format.profdata -instr-profile=json5format.profdata -use-color -format=html -output-dir=coverage
}

rm json5format-*.profraw

