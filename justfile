run *target:
    @echo 'Building {{target}}…'
    @echo '{{ if target == "" { "No ROM given: opening the file picker… [DEBUG MODE]" } else { "Running " + target + "… [DEBUG MODE]" } }}'
    cargo run -- {{target}}

# Same as `run`, but running a boot ROM first.
run-boot boot *target:
    @echo '{{ if target == "" { "Boot ROM " + boot + ", no ROM given: opening the file picker… [RELEASE MODE]" } else { "Running " + target + " with boot ROM " + boot + "… [RELEASE MODE]" } }}'
    cargo run --release -- --boot {{boot}} {{target}}

run-release *target:
    @echo '{{ if target == "" { "No ROM given: opening the file picker… [RELEASE MODE]" } else { "Running " + target + "… [RELEASE MODE]" } }}'
    cargo run --release -- {{target}}

ok: test
    cargo clippy -- -D warnings --verbose

test:
    cargo test
