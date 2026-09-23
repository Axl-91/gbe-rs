#!/usr/bin/env bash

set -u

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SUITES_DIR="$ROOT_DIR/test_suites"
TIMEOUT_SECS=60

usage() {
    cat <<EOF
Usage: $(basename "$0") [options]

With no options, runs every suite (blargg and mooneye).

Options:
  --blargg     Run only the blargg tests
  --mooneye    Run only the mooneye tests
  -h, --help   Show this help message

Options can be combined: $(basename "$0") --blargg --mooneye
EOF
}

# --- Argument parsing ---
suites=()
for arg in "$@"; do
    case "$arg" in
        --blargg)  suites+=("blargg") ;;
        --mooneye) suites+=("mooneye") ;;
        -h|--help) usage; exit 0 ;;
        *)
            echo "Unknown option: $arg" >&2
            usage >&2
            exit 2
            ;;
    esac
done

# No arguments: run everything
if (( ${#suites[@]} == 0 )); then
    suites=("blargg" "mooneye")
fi

# --- Build once (release) ---
if ! cargo build --release --quiet --bin rom_tests; then
    echo "Failed to build rom_tests" >&2
    exit 1
fi
BIN="$ROOT_DIR/target/release/rom_tests"

# --- Timeout command detection (timeout on Linux, gtimeout on macOS via coreutils) ---
TIMEOUT_CMD=""
if command -v timeout > /dev/null; then
    TIMEOUT_CMD="timeout"
elif command -v gtimeout > /dev/null; then
    TIMEOUT_CMD="gtimeout"
else
    echo "Warning: no timeout command found, tests will run without a time limit" >&2
fi

# --- Execution ---
passed=0
failed=0
failed_tests=()

run_test() {
    local rom="$1"
    local relative_path="${rom#"$SUITES_DIR/"}"
    local test_name="${relative_path%.gb}"

    printf "%-55s" "$test_name"

    local cmd=("$BIN" "$rom")
    if [[ -n "$TIMEOUT_CMD" ]]; then
        cmd=("$TIMEOUT_CMD" "$TIMEOUT_SECS" "${cmd[@]}")
    fi

    "${cmd[@]}" > /dev/null 2>&1
    local status=$?

    if (( status == 0 )); then
        echo "PASS"
        passed=$((passed + 1))
    elif (( status == 124 )); then
        echo "TIMEOUT"
        failed=$((failed + 1))
        failed_tests+=("$test_name (timeout after ${TIMEOUT_SECS}s)")
    else
        echo "FAIL"
        failed=$((failed + 1))
        failed_tests+=("$test_name")
    fi
}

for suite in "${suites[@]}"; do
    suite_dir="$SUITES_DIR/$suite"

    if [[ ! -d "$suite_dir" ]]; then
        echo "Directory not found: $suite_dir" >&2
        exit 1
    fi

    echo "=== $suite ==="
    while IFS= read -r -d '' rom; do
        run_test "$rom"
    done < <(find "$suite_dir" -type f -name '*.gb' -print0 | sort -z)
    echo
done

echo "----------------------------------------"
echo "Passed: $passed"
echo "Failed: $failed"

if (( failed > 0 )); then
    echo
    echo "Failed tests:"
    for t in "${failed_tests[@]}"; do
        echo "  - $t"
    done
    exit 1
fi
