.PHONY: test fmt fmt-check build clippy deploy demo

CONTRACTS = -p corridor-registry -p corridor-attestation -p verifier-mock
# ultrahonk-verifier builds on soroban-sdk 28 and needs a spec-shaking-aware
# build system: stellar-cli >= 25.2 (CI installs v28).
test:           ; cargo test --workspace --locked
fmt:            ; cargo fmt --all
fmt-check:      ; cargo fmt --all -- --check
build:          ; cargo build --release --target wasm32v1-none $(CONTRACTS)
build-verifier: ; stellar contract build --package ultrahonk-verifier
clippy:    ; cargo clippy --workspace --all-targets -- -D warnings
deploy:    ; ./scripts/deploy_testnet.sh
demo:      ; ./scripts/demo.sh
