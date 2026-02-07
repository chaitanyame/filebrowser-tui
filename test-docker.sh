#!/bin/bash
# Test script to verify Docker setup is working correctly

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Docker Setup Test                     ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
echo ""

PASSED=0
FAILED=0

# Test helper
test_step() {
    local name="$1"
    local command="$2"

    echo -n "Testing: $name... "

    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        ((FAILED++))
        return 1
    fi
}

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"
test_step "Docker is installed" "command -v docker"
test_step "Docker daemon is running" "docker info"

echo ""
echo -e "${YELLOW}Testing Docker build...${NC}"

# Clean build
echo -n "Cleaning old images... "
docker rmi filebrowser-tui:latest 2>/dev/null || true
echo -e "${GREEN}Done${NC}"

# Build image
echo -n "Building Docker image... "
if docker build -t filebrowser-tui:latest . > /tmp/docker-build.log 2>&1; then
    echo -e "${GREEN}✓ PASSED${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ FAILED${NC}"
    cat /tmp/docker-build.log
    ((FAILED++))
    exit 1
fi

echo ""
echo -e "${YELLOW}Testing Docker image...${NC}"

test_step "Image was created" "docker image inspect filebrowser-tui:latest"
test_step "Binary exists in container" "docker run --rm filebrowser-tui:latest test -f /app/fbt"
test_step "Binary is executable" "docker run --rm filebrowser-tui:latest test -x /app/fbt"
test_step "Container has non-root user" "docker run --rm filebrowser-tui:latest id fbt"

echo ""
echo -e "${YELLOW}Testing container runtime...${NC}"

# Test basic container execution
echo -n "Container starts and exits cleanly... "
if docker run --rm filebrowser-tui:latest sh -c "exit 0" 2>/dev/null; then
    echo -e "${GREEN}✓ PASSED${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ FAILED${NC}"
    ((FAILED++))
fi

# Test volume mount
echo -n "Volume mount works... "
if docker run --rm -v "$(pwd):/data:ro" filebrowser-tui:latest test -d /data; then
    echo -e "${GREEN}✓ PASSED${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ FAILED${NC}"
    ((FAILED++))
fi

# Test environment variables
echo -n "Environment variables are set... "
if docker run --rm filebrowser-tui:latest sh -c "test -n \$HOME && test -n \$TERM"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ FAILED${NC}"
    ((FAILED++))
fi

# Test working directory
echo -n "Working directory is /data... "
if docker run --rm filebrowser-tui:latest sh -c "pwd | grep -q /data"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ FAILED${NC}"
    ((FAILED++))
fi

echo ""
echo -e "${YELLOW}Testing docker-compose...${NC}"

test_step "docker-compose config is valid" "docker-compose config"

echo ""
echo -e "${BLUE}══════════════════════════════════════════${NC}"
echo -e "${GREEN}Tests Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Tests Failed: $FAILED${NC}"
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo ""
    echo -e "${GREEN}Docker setup is ready to use!${NC}"
    echo ""
    echo "Run with:"
    echo "  docker run -it --rm -v \"\$PWD:/data:rw\" filebrowser-tui:latest"
    echo "  docker-compose run --rm filebrowser"
    echo "  make docker-run"
    exit 0
fi
