# PowerShell script to run filebrowser-tui in Docker with proper terminal support

#Requires -Version 5.1

[CmdletBinding()]
param(
    [switch]$BuildOnly,
    [switch]$UseCompose,
    [switch]$DryRun,
    [string]$MountPath = $PWD,
    [switch]$Help
)

# Color output functions
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

# Show help
if ($Help) {
    Write-ColorOutput "File Browser TUI - Docker Runner" "Cyan"
    Write-Host ""
    Write-Host "Usage: .\run-docker.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -BuildOnly          Only build the image, don't run"
    Write-Host "  -UseCompose         Use docker-compose instead of docker run"
    Write-Host "  -DryRun             Show the command without running"
    Write-Host "  -MountPath PATH     Mount specific path (default: current directory)"
    Write-Host "  -Help               Show this help message"
    exit 0
}

# Show banner
Write-ColorOutput "╔══════════════════════════════════════════╗" "Cyan"
Write-ColorOutput "║   File Browser TUI - Docker Runner       ║" "Cyan"
Write-ColorOutput "╚══════════════════════════════════════════╝" "Cyan"
Write-Host ""
Write-ColorOutput "Mount path: $MountPath" "Green"
Write-ColorOutput "Terminal: $env:TERM" "Green"
Write-Host ""

# Check if Docker is installed
try {
    $null = Get-Command docker -ErrorAction Stop
} catch {
    Write-ColorOutput "Error: Docker is not installed" "Red"
    Write-Host "Please install Docker Desktop from https://www.docker.com/products/docker-desktop"
    exit 1
}

# Check if Docker daemon is running
try {
    $null = docker info 2>&1 | Out-Null
} catch {
    Write-ColorOutput "Error: Docker daemon is not running" "Red"
    Write-Host "Please start Docker Desktop"
    exit 1
}

# Configuration
$ImageName = "filebrowser-tui"
$ContainerName = "fbt"

# Convert Windows path to Docker path
if ($IsWindows -or $null -eq $IsWindows) {
    $DockerMountPath = $MountPath.Replace('\', '/')
    if ($DockerMountPath -match '^([A-Z]):(.*)$') {
        $DriveLetter = $matches[1].ToLower()
        $RestPath = $matches[2]
        $DockerMountPath = "/$DriveLetter$RestPath"
    }
} else {
    $DockerMountPath = $MountPath
}

# Build and run
if ($UseCompose) {
    Write-ColorOutput "Using docker-compose..." "Yellow"

    if ($DryRun) {
        Write-ColorOutput "Would run: docker-compose build && docker-compose run --rm filebrowser" "Yellow"
    } else {
        docker-compose build
        if ($LASTEXITCODE -ne 0) {
            Write-ColorOutput "Error: Docker build failed" "Red"
            exit 1
        }

        if (-not $BuildOnly) {
            Write-ColorOutput "Starting file browser..." "Green"
            docker-compose run --rm filebrowser
        }
    }
} else {
    # Build the image if needed
    $existingImage = docker images --format "{{.Repository}}" | Select-String "^${ImageName}$"
    if (-not $existingImage) {
        Write-ColorOutput "Building Docker image..." "Yellow"
        docker build -t "${ImageName}:latest" .
        if ($LASTEXITCODE -ne 0) {
            Write-ColorOutput "Error: Docker build failed" "Red"
            exit 1
        }
    }

    if ($BuildOnly) {
        Write-ColorOutput "Build complete!" "Green"
        exit 0
    }

    # Build docker run command
    $DockerCmd = @(
        "docker", "run", "-it", "--rm"
        "--name", $ContainerName
        "-v", "${DockerMountPath}:/data:rw"
        "-v", "$((Get-Location).Path.Replace('\', '/')):/workspace:ro"
        "-w", "/data"
        "-e", "TERM=$env:TERM"
        "-e", "LANG=C.UTF-8"
        "-e", "LC_ALL=C.UTF-8"
        "${ImageName}:latest"
    )

    if ($DryRun) {
        Write-ColorOutput "Would run: $($DockerCmd -join ' ')" "Yellow"
    } else {
        Write-ColorOutput "Starting file browser..." "Green"
        & $DockerCmd[0] $DockerCmd[1..($DockerCmd.Length-1)]
        exit $LASTEXITCODE
    }
}
