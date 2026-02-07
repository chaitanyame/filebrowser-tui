#!/bin/bash
# E2E Test Runner Script for File Browser TUI
#
# This script provides a convenient way to run E2E tests with various options.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default options
VERBOSE=false
RUN_IGNORED=false
SPECIFIC_TEST=""
TEST_THREADS=1
BUILD_FIRST=false

# Function to print usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Run E2E tests for the file browser TUI.

OPTIONS:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    -i, --ignored       Run ignored tests (manual testing)
    -t, --test TEST     Run specific test
    -j, --threads N     Number of test threads (default: 1)
    -b, --build         Build binary before running tests

EXAMPLES:
    # Run all E2E tests
    $0

    # Run with verbose output
    $0 -v

    # Run ignored tests (for manual testing)
    $0 -i

    # Run specific test
    $0 -t test_navigate_directories

    # Build first, then run tests
    $0 -b

EOF
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -i|--ignored)
            RUN_IGNORED=true
            shift
            ;;
        -t|--test)
            SPECIFIC_TEST="$2"
            shift 2
            ;;
        -j|--threads)
            TEST_THREADS="$2"
            shift 2
            ;;
        -b|--build)
            BUILD_FIRST=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
    esac
done

# Set environment variables based on options
if [ "$VERBOSE" = true ]; then
    export TUI_TEST_VERBOSE=1
    echo -e "${YELLOW}Verbose mode enabled${NC}"
fi

# Build binary if requested
if [ "$BUILD_FIRST" = true ]; then
    echo -e "${YELLOW}Building binary...${NC}"
    cargo build --release
    echo -e "${GREEN}Build complete${NC}"
fi

# Construct test command
TEST_CMD="cargo test --test e2e_tests"

# Add specific test if provided
if [ -n "$SPECIFIC_TEST" ]; then
    TEST_CMD="$TEST_CMD $SPECIFIC_TEST"
fi

# Add ignored flag if requested
if [ "$RUN_IGNORED" = true ]; then
    TEST_CMD="$TEST_CMD -- --ignored"
fi

# Add test threads
TEST_CMD="$TEST_CMD -- --test-threads $TEST_THREADS"

# Print what we're about to do
echo -e "${YELLOW}Running E2E tests...${NC}"
if [ -n "$SPECIFIC_TEST" ]; then
    echo -e "${YELLOW}Test: $SPECIFIC_TEST${NC}"
fi
echo -e "${YELLOW}Threads: $TEST_THREADS${NC}"
echo ""

# Run the tests
if eval $TEST_CMD; then
    echo ""
    echo -e "${GREEN}All E2E tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}Some E2E tests failed${NC}"
    echo ""
    echo "Tips for debugging:"
    echo "  1. Run with verbose mode: $0 -v"
    echo "  2. Run tests sequentially: $0 -j 1"
    echo "  3. Run specific test: $0 -t <test_name>"
    echo "  4. Build first: $0 -b"
    exit 1
fi
