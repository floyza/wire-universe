#!/usr/bin/env sh

# while sleep 2; do fd . client common | entr -dnr wasm-pack build client -t web --no-typescript -d ../assets/pkg; done &
# while sleep 2; do fd . server common | entr -dnr cargo run; done &
cargo watch -w common -w client -s "wasm-pack build client -t web --no-typescript -d ../assets/pkg" &
cargo watch -w common -w server -x run &

wait
