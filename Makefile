.PHONY: remote-gate
remote-gate:
	./scripts/pqc-remote.sh zkfmi-crypto 'cargo fmt --all --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --release --all-features'
