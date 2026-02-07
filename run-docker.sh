#!/bin/bash
# Script to run filebrowser-tui in Docker with proper terminal support

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker is not installed${NC}"
    echo "Please install Docker from https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Docker daemon is running
if ! docker info &> /dev/null; then
    echo -e "${RED}Error: Docker daemon is not running${NC}"
    echo "Please start Docker Desktop or the Docker daemon"
    exit 1
fi

# Configuration
IMAGE_NAME="filebrowser-tui"
CONTAINER_NAME="fbt"
MOUNT_PATH="${MOUNT_PATH:-$PWD}"

# Parse arguments
BUILD_ONLY=false
USE_COMPOSE=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --build-only|-b)
            BUILD_ONLY=true
            shift
            ;;
        --compose|-c)
            USE_COMPOSE=true
            shift
            ;;
        --dry-run|-n)
            DRY_RUN=true
            shift
            ;;
        --path|-p)
            MOUNT_PATH="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -b, --build-only    Only build the image, don't run"
            echo "  -c, --compose       Use docker-compose instead of docker run"
            echo "  -n, --dry-run       Show the command without running"
            echo "  -p, --path PATH     Mount specific path (default: current directory)"
            echo "  -h, --help          Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   File Browser TUI - Docker Runner       ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Mount path:${NC} $MOUNT_PATH"
echo -e "${GREEN}Terminal:${NC} $TERM"
echo ""

# Build command
if [ "$USE_COMPOSE" = true ]; then
    echo -e "${YELLOW}Using docker-compose...${NC}"
    if [ "$DRY_RUN" = true ]; then
        echo "Would run: docker-compose build && docker-compose run --rm filebrowser"
    else
        docker-compose build
        if [ "$BUILD_ONLY" = false ]; then
            exec docker-compose run --rm filebrowser
        fi
    fi
else
    # Build the image if needed
    if ! docker image inspect "$IMAGE_NAME:latest" &> /dev/null; then
        echo -e "${YELLOW}Building Docker image...${NC}"
        docker build -t "$IMAGE_NAME:latest" .
    fi

    if [ "$BUILD_ONLY" = true ]; then
        echo -e "${GREEN}Build complete!${NC}"
        exit 0
    fi

    # Build docker run command
    DOCKER_CMD="docker run -it --rm"
    DOCKER_CMD="$DOCKER_CMD --name $CONTAINER_NAME"
    DOCKER_CMD="$DOCKER_CMD -v \"$MOUNT_PATH:/data:rw\""
    DOCKER_CMD="$DOCKER_CMD -v \"$(pwd):/workspace:ro\""
    DOCKER_CMD="$DOCKER_CMD -w \"/data\""
    DOCKER_CMD="$DOCKER_CMD -e TERM=\"$TERM\""
    DOCKER_CMD="$DOCKER_CMD -e LANG=C.UTF-8"
    DOCKER_CMD="$DOCKER_CMD -e LC_ALL=C.UTF-8"
    DOCKER_CMD="$DOCKER_CMD $IMAGE_NAME:latest"

    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}Would run:${NC} $DOCKER_CMD"
    else
        echo -e "${GREEN}Starting file browser...${NC}"
        eval "exec $DOCKER_CMD"
    fi
fi
