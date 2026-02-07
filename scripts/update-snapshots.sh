#!/bin/bash
# Script to update all snapshots after intentional UI changes
#
# Usage:
#   ./scripts/update-snapshots.sh           # Review and accept snapshots interactively
#   ./scripts/update-snapshots.sh --accept  # Accept all snapshot changes without review
#
# This script runs the snapshot tests and provides an interface to review changes.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored message
print_msg() {
    local color=$1
    shift
    echo -e "${color}$*${NC}"
}

# Print header
print_header() {
    echo
    print_msg "$BLUE" "════════════════════════════════════════════════════════════════"
    print_msg "$BLUE" "$*"
    print_msg "$BLUE" "════════════════════════════════════════════════════════════════"
    echo
}

# Check if insta CLI is installed
check_insta() {
    if ! command -v cargo-insta &> /dev/null; then
        print_msg "$YELLOW" "cargo-insta not found. Installing..."
        cargo install cargo-insta
    fi
}

# Main script
main() {
    print_header "Snapshot Update Script"

    # Parse arguments
    local accept_all=false
    local review_only=false

    for arg in "$@"; do
        case $arg in
            --accept|-a)
                accept_all=true
                ;;
            --review|-r)
                review_only=true
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo
                echo "Options:"
                echo "  --accept, -a    Accept all snapshot changes without review"
                echo "  --review, -r    Only review existing snapshot changes"
                echo "  --help, -h      Show this help message"
                echo
                echo "Examples:"
                echo "  $0                  # Run tests and review interactively"
                echo "  $0 --accept         # Run tests and accept all changes"
                echo "  $0 --review         # Only review existing pending snapshots"
                exit 0
                ;;
        esac
    done

    # Ensure insta is installed
    check_insta

    if [ "$review_only" = false ]; then
        print_msg "$YELLOW" "Running snapshot tests..."
        echo

        # Run snapshot tests with insta
        if [ "$accept_all" = true ]; then
            # Accept all changes
            print_msg "$GREEN" "Accepting all snapshot changes..."
            cargo insta test --accept --unreferenced=auto
        else
            # Run with review
            cargo insta test --review --unreferenced=auto
        fi
    else
        # Only review existing snapshots
        print_msg "$YELLOW" "Reviewing existing snapshot changes..."
        echo
        cargo insta review
    fi

    local exit_code=$?

    echo
    if [ $exit_code -eq 0 ]; then
        print_msg "$GREEN" "✓ Snapshots updated successfully!"
    else
        print_msg "$RED" "✗ Snapshot update failed with exit code $exit_code"
        exit $exit_code
    fi

    # Show snapshot statistics
    print_header "Snapshot Statistics"

    local snapshot_dir="tests/snapshots"
    if [ -d "$snapshot_dir" ]; then
        local snapshot_count=$(find "$snapshot_dir" -name "*.snap" | wc -l)
        print_msg "$BLUE" "Total snapshots: $snapshot_count"

        # List snapshot files
        echo
        print_msg "$BLUE" "Snapshot files:"
        find "$snapshot_dir" -name "*.snap" | sort | while read -r file; do
            local name=$(basename "$file" .snap)
            echo "  - $name"
        done
    else
        print_msg "$YELLOW" "No snapshot directory found (run tests first)"
    fi

    echo
    print_msg "$BLUE" "Next steps:"
    echo "  - Review snapshots in $snapshot_dir"
    echo "  - Commit snapshots to version control"
    echo "  - Run 'cargo test' to verify all tests pass"
    echo
}

# Run main
main "$@"
