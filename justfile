run target:
    @echo 'Building {{target}}…'
    cargo run -- {{target}}

run-release target:
    @echo 'Building {{target}} (release)…'
    cargo run --release -- {{target}}

ok: test
    cargo clippy -- -D warnings --verbose

test:
    cargo test
