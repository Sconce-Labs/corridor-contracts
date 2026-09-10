contracts := "-p corridor-registry -p corridor-attestation -p verifier-mock -p ultrahonk-verifier"

default:
    @just --list

test:
    cargo test --workspace

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets

build:
    cargo build --release --target wasm32v1-none {{contracts}}

deploy:
    ./scripts/deploy_testnet.sh

demo:
    ./scripts/demo.sh
