export RUSTFLAGS=-g
perf record --call-graph=lbr cargo run --release -- $1
