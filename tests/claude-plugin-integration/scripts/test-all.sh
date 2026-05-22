#!/bin/bash
# Test all supported language samples

set -euo pipefail

cd "$(dirname "$0")/.."

echo "===== Rocket Index Claude Plugin Integration Tests ====="
echo

# Create results directory
mkdir -p results

# All supported language samples
LANGUAGES=(
    "fsharp-sample"
    "ruby-sample"
    "python-sample"
    "rust-sample"
    "go-sample"
    "java-sample"
    "javascript-sample"
    "typescript-sample"
    "csharp-sample"
    "php-sample"
    "c-sample"
    "cpp-sample"
    "swift-sample"
    "kotlin-sample"
    "objc-sample"
    "haxe-sample"
)

PASS=0
FAIL=0

for lang in "${LANGUAGES[@]}"; do
    echo "Testing $lang..."
    if ./scripts/test-language.sh "$lang"; then
        PASS=$((PASS + 1))
        echo "  ✓ PASS"
    else
        FAIL=$((FAIL + 1))
        echo "  ✗ FAIL"
    fi
    echo
done

echo "===== Test Summary ====="
echo "Passed: $PASS/$((PASS + FAIL))"
echo "Failed: $FAIL/$((PASS + FAIL))"
echo
echo "Results saved in results/"
echo
echo "Next steps:"
echo "1. Review results/*.json files"
echo "2. Manually verify tool usage and accuracy"
echo "3. Run: python3 scripts/analyze-results.py"
