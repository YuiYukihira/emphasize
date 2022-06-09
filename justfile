_default:
	@just --list


# Runs clippy on the source
check:
	cargo clippy --locked -- -D warnings

# Run unit tests
test:
	cargo test -- --nocapture

# Runs emphasize in local development mode
run:
	RUSTLOG="emphasize=info" cargo run .emphasize/config.yml