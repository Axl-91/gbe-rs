run target:
    @echo 'Building {{target}}…'
    cargo run -- {{target}}

# Same as `run`, but running a boot ROM first.
run-boot boot target:
    @echo 'Building {{target}} with boot ROM {{boot}}…'
    cargo run -- --boot {{boot}} {{target}}

run-release target:
    @echo 'Building {{target}} (release)…'
    cargo run --release -- {{target}}

ok: test
    cargo clippy -- -D warnings --verbose

test:
    cargo test
