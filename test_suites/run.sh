#!/usr/bin/env bash
#
# Runs the Game Boy test ROM suites (blargg, mooneye) in parallel and
# reports one result line per ROM, followed by a summary.
#
# Runner exit codes:
#   0     = PASS
#   1     = FAIL
#   2     = TIMEOUT (the runner hit its internal step limit)
#   124   = TIMEOUT (the external `timeout` command killed the runner)
#   other = CRASH   (panic, signal, ...)
#
# The script exits with 0 only if every ROM passed.

set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SUITES_DIR="$ROOT_DIR/test_suites"
BIN_DIR="$ROOT_DIR/target/release"

# Every suite is a directory under $SUITES_DIR with a runner binary of the
# same name in $BIN_DIR.
ALL_SUITES=(blargg mooneye microtests)

DEFAULT_TIMEOUT_SECS=60

usage() {
    cat <<EOF
Usage: $(basename "$0") [options]

With no suite options, runs every suite (${ALL_SUITES[*]}).

Suites:
  --blargg           Run only the blargg tests
  --mooneye          Run only the mooneye tests
  --microtests       Run only the gbmicro tests

Options:
  -j, --jobs N       Number of ROMs to run in parallel (default: CPU count)
  -t, --timeout N    Per-ROM time limit in seconds (default: $DEFAULT_TIMEOUT_SECS)
  -v, --verbose      Print the runner output of every non-passing ROM
  -h, --help         Show this help message

Suite options can be combined:
  $(basename "$0") --blargg --mooneye
EOF
}

die_usage() {
    echo "$1" >&2
    usage >&2
    exit 2
}

# ------------------------------------------------------------
# Argument parsing
# ------------------------------------------------------------

suites=()
max_jobs="$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 1)"
TIMEOUT_SECS="$DEFAULT_TIMEOUT_SECS"
verbose=0

# Adds a suite to the selection, ignoring duplicates.
add_suite() {
    if (( ${#suites[@]} > 0 )) && [[ " ${suites[*]} " == *" $1 "* ]]; then
        return 0
    fi

    suites+=("$1")
}

# Exits with a usage error unless $2 is a positive integer.
require_positive_int() {
    [[ "$2" =~ ^[1-9][0-9]*$ ]] || die_usage "Invalid value for $1: $2"
}

while (( $# > 0 )); do
    case "$1" in
        --blargg)
            add_suite "blargg"
            ;;

        --mooneye)
            add_suite "mooneye"
            ;;
            
        --microtests)
            add_suite "microtests"
            ;;

        -j|--jobs)
            (( $# >= 2 )) || die_usage "Missing value for $1"
            require_positive_int "$1" "$2"
            max_jobs="$2"
            shift
            ;;

        -t|--timeout)
            (( $# >= 2 )) || die_usage "Missing value for $1"
            require_positive_int "$1" "$2"
            TIMEOUT_SECS="$2"
            shift
            ;;

        -v|--verbose)
            verbose=1
            ;;

        -h|--help)
            usage
            exit 0
            ;;

        *)
            die_usage "Unknown option: $1"
            ;;
    esac

    shift
done

# No suite selected: run everything.
if (( ${#suites[@]} == 0 )); then
    suites=("${ALL_SUITES[@]}")
fi

# ------------------------------------------------------------
# Validate suite directories (fail fast, before building)
# ------------------------------------------------------------

suite_dirs=()

for suite in "${suites[@]}"; do
    suite_dir="$SUITES_DIR/$suite"

    if [[ ! -d "$suite_dir" ]]; then
        echo "Directory not found: $suite_dir" >&2
        exit 1
    fi

    suite_dirs+=("$suite_dir")
done

# ------------------------------------------------------------
# Build only the runners that are needed
# ------------------------------------------------------------

build_args=()

for suite in "${suites[@]}"; do
    build_args+=(--bin "$suite")
done

echo "Building test runners..."

if ! cargo build --release --quiet "${build_args[@]}"; then
    echo "Failed to build test runners" >&2
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
    echo "Warning: no timeout command found; tests will run without a time limit." >&2
fi

# ------------------------------------------------------------
# Colors (only when writing to a terminal)
# ------------------------------------------------------------

if [[ -t 1 ]]; then
    C_RESET=$'\033[0m'
    C_GREEN=$'\033[32m'
    C_RED=$'\033[31m'
    C_YELLOW=$'\033[33m'
else
    C_RESET=""
    C_GREEN=""
    C_RED=""
    C_YELLOW=""
fi

# ------------------------------------------------------------
# Scratch space for results and runner logs
# ------------------------------------------------------------

RESULTS_DIR="$(mktemp -d "${TMPDIR:-/tmp}/gb-tests.XXXXXX")"
RESULTS_FILE="$RESULTS_DIR/results.tsv"

mkdir -p "$RESULTS_DIR/logs"
: > "$RESULTS_FILE"

trap 'rm -rf "$RESULTS_DIR"' EXIT
trap 'exit 130' INT TERM

# ------------------------------------------------------------
# Run one test
#
# Runs in a child bash spawned by xargs, so everything it needs is
# exported below. Each call appends a single line to the results file
# and prints a single progress line; both are small enough to be
# written atomically when several jobs finish at the same time.
# ------------------------------------------------------------

# Path of the log file that holds the runner output of a test.
log_path_for() {
    local test_name="$1"

    echo "$RESULTS_DIR/logs/${test_name//\//__}.log"
}

run_test() {
    local rom="$1"

    # "blargg/cpu_instrs/01-special" from ".../test_suites/blargg/.../01-special.gb"
    local test_name="${rom#"$SUITES_DIR/"}"
    test_name="${test_name%.gb}"

    # The first path component is the suite, which is also the runner name.
    local suite="${test_name%%/*}"
    local log_file
    log_file="$(log_path_for "$test_name")"

    local cmd=("$BIN_DIR/$suite" "$rom")

    if [[ -n "$TIMEOUT_CMD" ]]; then
        cmd=("$TIMEOUT_CMD" "$TIMEOUT_SECS" "${cmd[@]}")
    fi

    "${cmd[@]}" > "$log_file" 2>&1
    local status=$?

    local label color
    case "$status" in
        0)
            label="PASS"
            color="$C_GREEN"
            ;;

        1)
            label="FAIL"
            color="$C_RED"
            ;;

        2|124)
            label="TIMEOUT"
            color="$C_YELLOW"
            ;;

        *)
            label="CRASH"
            color="$C_RED"
            ;;
    esac

    printf '%s\t%s\n' "$label" "$test_name" >> "$RESULTS_FILE"
    printf '%-60s %s%s%s\n' "$test_name" "$color" "$label" "$C_RESET"
}

export -f log_path_for run_test
export SUITES_DIR BIN_DIR TIMEOUT_CMD TIMEOUT_SECS RESULTS_DIR RESULTS_FILE
export C_RESET C_GREEN C_RED C_YELLOW

# ------------------------------------------------------------
# Run all suites through a single worker pool
#
# Using one pool for every suite keeps all cores busy until the very
# end. Results are printed as they complete, so their order may vary
# between runs; the summary below is always sorted.
# ------------------------------------------------------------

echo "Running tests with $max_jobs parallel job(s)..."
echo

SECONDS=0

find "${suite_dirs[@]}" -type f -name '*.gb' -print0 \
    | sort -z \
    | xargs -0 -r -n 1 -P "$max_jobs" bash -c 'run_test "$1"' _

elapsed=$SECONDS

# ------------------------------------------------------------
# Summary
# ------------------------------------------------------------

# Number of results with the given label.
count_label() {
    grep -c "^$1"$'\t' "$RESULTS_FILE" || true
}

# Prints the sorted list of tests with the given label under a title.
# With --verbose, also prints the runner output of each of them.
print_group() {
    local label="$1"
    local title="$2"

    local names
    names="$(awk -F'\t' -v label="$label" '$1 == label { print $2 }' "$RESULTS_FILE" | sort)"

    [[ -z "$names" ]] && return

    echo
    echo "$title:"

    local name
    while IFS= read -r name; do
        echo "  - $name"

        if (( verbose )); then
            sed 's/^/      | /' "$(log_path_for "$name")"
            echo
        fi
    done <<< "$names"
}

passed="$(count_label PASS)"
failed="$(count_label FAIL)"
timeouts="$(count_label TIMEOUT)"
crashes="$(count_label CRASH)"
total=$(( passed + failed + timeouts + crashes ))

if (( total == 0 )); then
    echo "No test ROMs found in: ${suite_dirs[*]}" >&2
    exit 1
fi

echo
echo "----------------------------------------"
echo "Passed:   $passed"
echo "Failed:   $failed"
echo "Timeouts: $timeouts"
echo "Crashes:  $crashes"
echo "Total:    $total (${elapsed}s)"

print_group FAIL "Failed tests"
print_group TIMEOUT "Timed out tests"
print_group CRASH "Crashed tests"

# ------------------------------------------------------------
# Final status
# ------------------------------------------------------------

(( passed == total ))
