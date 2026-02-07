#!/bin/bash
# Helper script for running specific benchmark scenarios
# Usage: ./run_benches.sh <scenario>

set -e

BENCH_NAME="filebrowser_bench"
CARGO_CMD="cargo bench --bench $BENCH_NAME"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${GREEN}=== $1 ===${NC}"
}

print_info() {
    echo -e "${YELLOW}Running: $1${NC}"
}

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found. Please install Rust toolchain.${NC}"
    exit 1
fi

# Parse arguments
SCENARIO=${1:-"all"}

case "$SCENARIO" in
    all)
        print_header "Running all benchmarks"
        $CARGO_CMD
        ;;

    file_listing)
        print_header "File Listing Benchmarks"
        print_info "Directory reading at various scales"
        $CARGO_CMD -- file_listing
        ;;

    sorting)
        print_header "Sorting Benchmarks"
        print_info "Sort performance by different criteria"
        $CARGO_CMD -- sorting
        ;;

    search)
        print_header "Search Benchmarks"
        print_info "Filename and content search"
        $CARGO_CMD -- search
        ;;

    ui_rendering)
        print_header "UI Rendering Benchmarks"
        print_info "Simulated rendering performance"
        $CARGO_CMD -- ui_rendering
        ;;

    file_ops)
        print_header "File Operations Benchmarks"
        print_info "Copy and traversal operations"
        $CARGO_CMD -- file_operations
        ;;

    bulk_rename)
        print_header "Bulk Rename Benchmarks"
        print_info "Pattern-based rename operations"
        $CARGO_CMD -- bulk_rename
        ;;

    filtering)
        print_header "Filtering Benchmarks"
        print_info "File filtering performance"
        $CARGO_CMD -- filtering
        ;;

    memory)
        print_header "Memory Benchmarks"
        print_info "Memory allocation patterns"
        $CARGO_CMD -- memory
        ;;

    sort_algorithms)
        print_header "Sort Algorithm Comparison"
        print_info "Compare different sorting algorithms"
        $CARGO_CMD -- sort_algorithms
        ;;

    throughput)
        print_header "Throughput Benchmarks"
        print_info "Data processing throughput"
        $CARGO_CMD -- throughput
        ;;

    quick)
        print_header "Quick Performance Check"
        print_info "Subset of critical benchmarks"
        $CARGO_CMD -- \
            file_listing/read_directory/100 \
            sorting/by_name/100 \
            search/filename_search/3 \
            ui_rendering/render_file_list/10 \
            bulk_rename/preview_simple_replace/100
        ;;

    baseline)
        print_header "Creating Baseline"
        print_info "Saving current performance as baseline"
        $CARGO_CMD -- --save-baseline main
        echo -e "${GREEN}Baseline saved to target/criterion/main${NC}"
        ;;

    compare)
        print_header "Comparing to Baseline"
        print_info "Comparing current performance to saved baseline"
        if [ -d "target/criterion/main" ]; then
            $CARGO_CMD -- --baseline main
        else
            echo -e "${RED}Error: No baseline found. Run './run_benches.sh baseline' first.${NC}"
            exit 1
        fi
        ;;

    report)
        print_header "Opening Benchmark Report"
        REPORT_PATH="target/criterion/report/index.html"
        if [ -f "$REPORT_PATH" ]; then
            echo "Opening $REPORT_PATH"
            # Try different commands based on OS
            if command -v xdg-open &> /dev/null; then
                xdg-open "$REPORT_PATH"
            elif command -v open &> /dev/null; then
                open "$REPORT_PATH"
            elif command -v start &> /dev/null; then
                start "$REPORT_PATH"
            else
                echo -e "${YELLOW}Could not open browser automatically${NC}"
                echo "Please open $REPORT_PATH in your browser"
            fi
        else
            echo -e "${RED}Error: No benchmark report found. Run benchmarks first.${NC}"
            exit 1
        fi
        ;;

    clean)
        print_header "Cleaning Benchmark Data"
        rm -rf target/criterion
        echo -e "${GREEN}Benchmark data cleaned${NC}"
        ;;

    help|--help|-h)
        echo "Benchmark runner script for filebrowser-tui"
        echo ""
        echo "Usage: ./run_benches.sh <scenario>"
        echo ""
        echo "Scenarios:"
        echo "  all              Run all benchmarks (default)"
        echo "  file_listing     File listing operations"
        echo "  sorting          Sorting performance"
        echo "  search           Search operations"
        echo "  ui_rendering     UI rendering simulation"
        echo "  file_ops         File operations (copy, traversal)"
        echo "  bulk_rename      Bulk rename operations"
        echo "  filtering        File filtering"
        echo "  memory           Memory allocation patterns"
        echo "  sort_algorithms  Sort algorithm comparison"
        echo "  throughput       Data processing throughput"
        echo "  quick            Quick performance check (subset)"
        echo "  baseline         Create performance baseline"
        echo "  compare          Compare to baseline"
        echo "  report           Open HTML report"
        echo "  clean            Clean benchmark data"
        echo "  help             Show this help message"
        echo ""
        echo "Examples:"
        echo "  ./run_benches.sh quick"
        echo "  ./run_benches.sh sorting"
        echo "  ./run_benches.sh baseline"
        echo "  ./run_benches.sh compare"
        ;;

    *)
        echo -e "${RED}Unknown scenario: $SCENARIO${NC}"
        echo "Run './run_benches.sh help' for usage information"
        exit 1
        ;;
esac

echo -e "${GREEN}Done!${NC}"
