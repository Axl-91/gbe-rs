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

Options can be combined:
  $(basename "$0") --blargg --mooneye
EOF
}

# ------------------------------------------------------------
# Argument parsing
# ------------------------------------------------------------

suites=()

for arg in "$@"; do
    case "$arg" in
        --blargg)
            suites+=("blargg")
            ;;

        --mooneye)
            suites+=("mooneye")
            ;;

        -h|--help)
            usage
            exit 0
            ;;

        *)
            echo "Unknown option: $arg" >&2
            usage >&2
            exit 2
            ;;
    esac
done

# No arguments: run everything.
if (( ${#suites[@]} == 0 )); then
    suites=("blargg" "mooneye")
fi

# ------------------------------------------------------------
# Build runners
# ------------------------------------------------------------

echo "Building test runners..."

if ! cargo build --release --quiet \
    --bin blargg \
    --bin mooneye; then

    echo "Failed to build test runners" >&2
    exit 1
fi

BLARGG_BIN="$ROOT_DIR/target/release/blargg"
MOONEYE_BIN="$ROOT_DIR/target/release/mooneye"

# ------------------------------------------------------------
# Check binaries
# ------------------------------------------------------------

if [[ ! -x "$BLARGG_BIN" ]]; then
    echo "Blargg runner not found: $BLARGG_BIN" >&2
    exit 1
fi

if [[ ! -x "$MOONEYE_BIN" ]]; then
    echo "Mooneye runner not found: $MOONEYE_BIN" >&2
    exit 1
fi

# ------------------------------------------------------------
# Timeout command detection
#
# Linux:
#   timeout
#
# macOS with coreutils:
#   gtimeout
# ------------------------------------------------------------

TIMEOUT_CMD=""

if command -v timeout > /dev/null 2>&1; then
    TIMEOUT_CMD="timeout"
elif command -v gtimeout > /dev/null 2>&1; then
    TIMEOUT_CMD="gtimeout"
else
    echo "Warning: no timeout command found."
    echo "Tests will run without a time limit." >&2
fi

# ------------------------------------------------------------
# Statistics
# ------------------------------------------------------------

passed=0
failed=0
timeouts=0

failed_tests=()
timeout_tests=()

# ------------------------------------------------------------
# Run one test
# ------------------------------------------------------------

run_test() {
    local suite="$1"
    local rom="$2"

    local relative_path="${rom#"$SUITES_DIR/"}"
    local test_name="${relative_path%.gb}"

    local bin=""

    case "$suite" in
        blargg)
            bin="$BLARGG_BIN"
            ;;

        mooneye)
            bin="$MOONEYE_BIN"
            ;;

        *)
            echo "Unknown suite: $suite" >&2
            return 1
            ;;
    esac

    printf "%-60s" "$test_name"

    local cmd=("$bin" "$rom")

    if [[ -n "$TIMEOUT_CMD" ]]; then
        cmd=("$TIMEOUT_CMD" "$TIMEOUT_SECS" "${cmd[@]}")
    fi

    # We don't print runner output here.
    #
    # The runner's exit code is enough:
    #
    #   0 = PASS
    #   1 = FAIL
    #   2 = TIMEOUT/internal timeout
    #
    # External timeout normally returns 124.
    "${cmd[@]}" > /dev/null 2>&1
    local status=$?

    if (( status == 0 )); then
        echo "PASS"
        passed=$((passed + 1))

    elif (( status == 124 )); then
        echo "TIMEOUT"
        failed=$((failed + 1))
        timeouts=$((timeouts + 1))
        timeout_tests+=("$test_name")

    else
        echo "FAIL"
        failed=$((failed + 1))
        failed_tests+=("$test_name")
    fi
}

# ------------------------------------------------------------
# Run suites
# ------------------------------------------------------------

for suite in "${suites[@]}"; do
    suite_dir="$SUITES_DIR/$suite"

    if [[ ! -d "$suite_dir" ]]; then
        echo "Directory not found: $suite_dir" >&2
        exit 1
    fi

    echo
    echo "=== $suite ==="

    while IFS= read -r -d '' rom; do
        run_test "$suite" "$rom"
    done < <(
        find "$suite_dir" \
            -type f \
            -name '*.gb' \
            -print0 |
        sort -z
    )

    echo
done

# ------------------------------------------------------------
# Summary
# ------------------------------------------------------------

echo "----------------------------------------"
echo "Passed:   $passed"
echo "Failed:   $failed"
echo "Timeouts: $timeouts"

# ------------------------------------------------------------
# Failed tests
# ------------------------------------------------------------

if (( ${#failed_tests[@]} > 0 )); then
    echo
    echo "Failed tests:"

    for test in "${failed_tests[@]}"; do
        echo "  - $test"
    done
fi

# ------------------------------------------------------------
# Timeout tests
# ------------------------------------------------------------

if (( ${#timeout_tests[@]} > 0 )); then
    echo
    echo "Timed out tests:"

    for test in "${timeout_tests[@]}"; do
        echo "  - $test"
    done
fi

# ------------------------------------------------------------
# Final status
# ------------------------------------------------------------

if (( failed > 0 )); then
    exit 1
fi

exit 0
