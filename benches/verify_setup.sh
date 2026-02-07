#!/bin/bash
# Verification script to ensure benchmark suite is properly configured

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    local status=$1
    local message=$2

    if [ "$status" = "OK" ]; then
        echo -e "${GREEN}✓${NC} $message"
    elif [ "$status" = "WARN" ]; then
        echo -e "${YELLOW}⚠${NC} $message"
    else
        echo -e "${RED}✗${NC} $message"
    fi
}

echo "=== Filebrowser-tui Benchmark Suite Verification ==="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_status "ERROR" "Not in project root (Cargo.toml not found)"
    exit 1
fi
print_status "OK" "In project root"

# Check Cargo.toml configuration
echo ""
echo "Checking Cargo.toml configuration..."

if grep -q "criterion = \"0.5\"" Cargo.toml; then
    print_status "OK" "Criterion dependency found"
else
    print_status "ERROR" "Criterion dependency missing"
fi

if grep -q "\[\[bench\]\]" Cargo.toml && grep -q "name = \"filebrowser_bench\"" Cargo.toml; then
    print_status "OK" "Benchmark configuration found"
else
    print_status "ERROR" "Benchmark configuration missing"
fi

if grep -q "\[profile.bench\]" Cargo.toml; then
    print_status "OK" "Bench profile configured"
else
    print_status "WARN" "Bench profile not configured (using default)"
fi

# Check benchmark files
echo ""
echo "Checking benchmark files..."

files=(
    "benches/mod.rs"
    "benches/filebrowser_bench.rs"
    "benches/README.md"
    "benches/IMPLEMENTATION.md"
    "benches/ADDING_BENCHMARKS.md"
    "benches/SUMMARY.md"
    "benches/run_benches.sh"
    "benches/.critter.toml"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        print_status "OK" "$file exists"
    else
        print_status "ERROR" "$file missing"
    fi
done

# Check script permissions
echo ""
echo "Checking script permissions..."

if [ -x "benches/run_benches.sh" ]; then
    print_status "OK" "run_benches.sh is executable"
else
    print_status "WARN" "run_benches.sh is not executable"
    echo "  Run: chmod +x benches/run_benches.sh"
fi

# Count benchmarks
echo ""
echo "Analyzing benchmark suite..."

if [ -f "benches/filebrowser_bench.rs" ]; then
    benchmark_count=$(grep -c "bench_function\|bench_with_input" benches/filebrowser_bench.rs || true)
    group_count=$(grep -c "c.benchmark_group" benches/filebrowser_bench.rs || true)
    print_status "OK" "Found $group_count benchmark groups"
    print_status "OK" "Found $benchmark_count individual benchmarks"
fi

# Check documentation
echo ""
echo "Checking documentation..."

docs=(
    "benches/README.md"
    "benches/IMPLEMENTATION.md"
    "benches/ADDING_BENCHMARKS.md"
)

for doc in "${docs[@]}"; do
    if [ -f "$doc" ]; then
        lines=$(wc -l < "$doc")
        print_status "OK" "$doc ($lines lines)"
    fi
done

# Check for common issues
echo ""
echo "Checking for common issues..."

# Check if criterion is in dev-dependencies
if grep -A 10 "\[dev-dependencies\]" Cargo.toml | grep -q "criterion"; then
    print_status "OK" "Criterion in dev-dependencies"
else
    print_status "ERROR" "Criterion not in dev-dependencies"
fi

# Check if harness is disabled
if grep -A 2 "\[\[bench\]\]" Cargo.toml | grep -q "harness = false"; then
    print_status "OK" "Benchmark harness disabled"
else
    print_status "WARN" "Benchmark harness may not be disabled"
fi

# Summary
echo ""
echo "=== Summary ==="
echo ""
echo "Total files: $(ls -1 benches/*.{rs,md,sh,toml} 2>/dev/null | wc -l)"
echo "Total code lines: $(cat benches/*.rs 2>/dev/null | wc -l)"
echo "Total documentation lines: $(cat benches/*.md 2>/dev/null | wc -l)"
echo ""

# Provide next steps
echo "=== Next Steps ==="
echo ""
echo "1. Build and check benchmarks:"
echo "   cargo check --benches"
echo ""
echo "2. Run a quick benchmark test:"
echo "   ./benches/run_benches.sh quick"
echo ""
echo "3. View documentation:"
echo "   cat benches/README.md"
echo ""
echo "4. Run all benchmarks:"
echo "   cargo bench"
echo ""

# Final status
echo "=== Verification Complete ==="

# Exit with appropriate code
if [ -f "benches/filebrowser_bench.rs" ] && grep -q "criterion = \"0.5\"" Cargo.toml; then
    exit 0
else
    exit 1
fi
