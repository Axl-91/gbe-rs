run target:
    @echo 'Building {{target}}…'
    cargo run -- {{target}}
    
ok: test
    cargo clippy -- -D warnings --verbose

test:
    cargo test
