#!/usr/bin/env bash

set -u

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

passed=0
failed=0

run_test() {
    local rom="$1"
    local relative_path="${rom#"$ROOT_DIR/test_suites/"}"
    local test_name="${relative_path%.gb}"

    printf "%-55s" "$test_name"

    if cargo run --quiet --bin rom_tests -- "$rom" > /dev/null; then
        echo "PASS"
        passed=$((passed + 1))
    else
        echo "FAIL"
        failed=$((failed + 1))
    fi
}

while IFS= read -r -d '' rom; do
    run_test "$rom"
done < <(find "$ROOT_DIR/test_suites" -type f -name '*.gb' -print0 | sort -z)

echo
echo "----------------------------------------"
echo "Passed: $passed"
echo "Failed: $failed"

if (( failed > 0 )); then
    exit 1
fi
