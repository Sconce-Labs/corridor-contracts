.PHONY: test fmt fmt-check build clippy deploy demo

CONTRACTS = -p corridor-registry -p corridor-attestation -p verifier-mock

test:      ; cargo test --workspace
fmt:       ; cargo fmt --all
fmt-check: ; cargo fmt --all -- --check
build:     ; cargo build --release --target wasm32v1-none $(CONTRACTS)
clippy:    ; cargo clippy --workspace --all-targets -- -D warnings
deploy:    ; ./scripts/deploy_testnet.sh
demo:      ; ./scripts/demo.sh
