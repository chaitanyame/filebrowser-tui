# Docker Guide for File Browser TUI

Complete guide for running the File Browser TUI application in Docker containers.

## Quick Start

```bash
# Option 1: Using the test script (tests + setup)
./test-docker.sh

# Option 2: Using docker-compose
docker-compose build
docker-compose run --rm filebrowser

# Option 3: Using docker directly
docker build -t filebrowser-tui:latest .
docker run -it --rm -v "$PWD:/data:rw" filebrowser-tui:latest

# Option 4: Using Make
make docker-run
```

## Pre-flight Checks

The test script verifies everything is set up correctly:

```bash
./test-docker.sh
```

This checks:
- ✓ Docker is installed
- ✓ Docker daemon is running
- ✓ Image builds successfully
- ✓ Binary exists and is executable
- ✓ Volume mounts work
- ✓ Environment variables are set

## Running Options

### 1. Browse Current Directory

```bash
docker run -it --rm -v "$PWD:/data:rw" filebrowser-tui:latest
```

### 2. Browse Home Directory

```bash
docker run -it --rm -v "$HOME:/data:rw" filebrowser-tui:latest
```

### 3. Browse Specific Directory

```bash
docker run -it --rm -v "/path/to/browse:/data:rw" filebrowser-tui:latest
```

### 4. Multiple Mount Points

```bash
docker run -it --rm \
  -v "$HOME:/home:rw" \
  -v "/var/www:/www:rw" \
  -w "/home" \
  filebrowser-tui:latest
```

## Using docker-compose

Edit `docker-compose.yml` to customize mount paths, then:

```bash
# Build and run
docker-compose build
docker-compose run --rm filebrowser

# Or just use make
make docker-compose
```

### Custom Mounts

Create `docker-compose.override.yml`:

```yaml
version: '3.8'

services:
  filebrowser:
    volumes:
      - .:/workspace:ro
      - /home/user/Documents:/data:rw
      - /home/user/Downloads:/downloads:rw
```

## Using the Run Scripts

### Linux/Mac

```bash
# Show help
./run-docker.sh --help

# Run with defaults (current directory)
./run-docker.sh

# Run with specific path
./run-docker.sh --path /home/user/Documents

# Build only
./run-docker.sh --build-only

# Use docker-compose
./run-docker.sh --compose

# Dry run (show command without running)
./run-docker.sh --dry-run
```

### Windows (PowerShell)

```powershell
# Show help
.\run-docker.ps1 -Help

# Run with defaults
.\run-docker.ps1

# Run with specific path
.\run-docker.ps1 -MountPath "C:\Users\user\Documents"

# Build only
.\run-docker.ps1 -BuildOnly
```

## Troubleshooting

### "Permission denied" errors

On Linux, run with your user ID:

```bash
docker run -it --rm \
  -v "$PWD:/data:rw" \
  --user $(id -u):$(id -g) \
  filebrowser-tui:latest
```

### Container exits immediately

Make sure you're using `-it` flags:

```bash
docker run -it --rm filebrowser-tui:latest
#     ^^ Required!
```

### Terminal looks weird

Pass the TERM variable:

```bash
docker run -it --rm -e TERM=$TERM filebrowser-tui:latest
```

### Can't see files

Check the mount path is correct:

```bash
# Wrong
docker run -v /wrong/path:/data ...

# Correct
docker run -v /correct/path:/data ...
```

### Build failures

Clean rebuild:

```bash
docker rmi filebrowser-tui:latest
docker build --no-cache -t filebrowser-tui:latest .
```

## Platform-Specific Notes

### Windows (PowerShell)

```powershell
# Mount home directory
docker run -it --rm `
  -v "${env:USERPROFILE}:/data:rw" `
  -e TERM=$env:TERM `
  filebrowser-tui:latest
```

### Windows (Git Bash)

```bash
docker run -it --rm \
  -v "$(pwd):/data:rw" \
  -e TERM=$TERM \
  filebrowser-tui:latest
```

### macOS

Docker Desktop may need folder access permissions:
1. Open Docker Desktop
2. Go to Settings → Resources → File Sharing
3. Add the directories you want to mount

### Linux

May need to add user to docker group:

```bash
sudo usermod -aG docker $USER
newgrp docker
```

## Development

### Building the Image

```bash
docker build -t filebrowser-tui:latest .
```

### Rebuilding After Changes

```bash
docker build --no-cache -t filebrowser-tui:latest .
```

### Checking Image Size

```bash
docker images filebrowser-tui
```

### Running a Shell in the Container

```bash
docker run -it --rm --entrypoint sh filebrowser-tui:latest
```

## Clean Up

```bash
# Remove the image
docker rmi filebrowser-tui:latest

# Remove all build artifacts
make docker-clean
```

## Security Notes

- Container runs as non-root user (fbt:1000)
- Only mounts directories you specify
- No network access required
- No privileged operations needed

## Performance Tips

1. **Use named volumes** for frequently accessed directories
2. **On macOS**: Add mounts to Docker Desktop file sharing settings
3. **On Windows**: Use WSL2 for better performance

## Getting Help

```bash
# Show all Make targets
make help

# Show run script help
./run-docker.sh --help
.\run-docker.ps1 -Help

# Test the Docker setup
./test-docker.sh
```
