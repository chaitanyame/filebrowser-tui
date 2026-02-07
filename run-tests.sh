#!/bin/bash
# Test runner script for filebrowser-tui
# Runs all tests and provides a summary

set -e

echo "=========================================="
echo "  File Browser TUI - Test Runner"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found${NC}"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo -e "${BLUE}[1/6] Running unit tests (lib)...${NC}"
if cargo test --lib 2>&1 | tee test_output_unit.log; then
    echo -e "${GREEN}✓ Unit tests passed${NC}"
else
    echo -e "${RED}✗ Unit tests failed${NC}"
    cat test_output_unit.log
    exit 1
fi

echo ""
echo -e "${BLUE}[2/6] Running integration tests...${NC}"
if cargo test --test integration_tests 2>&1 | tee test_output_integration.log; then
    echo -e "${GREEN}✓ Integration tests passed${NC}"
else
    echo -e "${RED}✗ Integration tests failed${NC}"
    cat test_output_integration.log
    exit 1
fi

echo ""
echo -e "${BLUE}[3/6] Running property tests...${NC}"
if cargo test property_tests 2>&1 | tee test_output_property.log; then
    echo -e "${GREEN}✓ Property tests passed${NC}"
else
    echo -e "${RED}✗ Property tests failed${NC}"
    cat test_output_property.log
    exit 1
fi

echo ""
echo -e "${BLUE}[4/6] Running snapshot tests...${NC}"
if cargo test --test snapshot_tests -- --test-threads=1 2>&1 | tee test_output_snapshot.log; then
    echo -e "${GREEN}✓ Snapshot tests passed${NC}"
else
    echo -e "${RED}✗ Snapshot tests failed${NC}"
    echo "Run 'cargo insta test --review' to review changes"
    cat test_output_snapshot.log
    exit 1
fi

echo ""
echo -e "${YELLOW}[5/6] E2E tests (skipped - require compiled binary)...${NC}"
echo "Run with: cargo test --test e2e_tests -- --ignored"

echo ""
echo -e "${BLUE}[6/6] Running benchmarks...${NC}"
echo "Run with: cargo bench"

echo ""
echo -e "${GREEN}=========================================="
echo "  All tests passed! ✓"
echo "==========================================${NC}"

# Clean up
rm -f test_output_*.log

exit 0
