# PowerShell script to run the Matrix agent test

# Configuration - Edit these variables as needed
# Set PYTHON_VENV_PATH to your virtual environment path, or leave empty to use system Python
$PYTHON_VENV_PATH = ""  # Example: "../mo", "./venv", or "C:\path\to\venv"
# Choose which example to run - "ex1" for basic test, "moderator" for moderation agent
$EXAMPLE = "moderator"  # Change to "ex1" to run the basic example

Write-Host "Starting Matrix test..." -ForegroundColor Green

# Check for .env file
if (-not (Test-Path ".env")) {
    Write-Host "Warning: .env file not found!" -ForegroundColor Red
    Write-Host "Creating a sample .env file - please edit with your actual API key" -ForegroundColor Yellow
    # Use UTF8NoBOM encoding to prevent BOM issues
    "GEMINI_API_KEY=your_api_key_here" | Out-File -FilePath ".env" -Encoding utf8
    Write-Host "Please edit the .env file with your Gemini API key and run this script again." -ForegroundColor Red
    exit
}

# Copy .env to the examples directory - ensure proper encoding during copy
Write-Host "Copying .env file to examples/$EXAMPLE" -ForegroundColor Yellow
$exampleDir = "examples/$EXAMPLE"
if (-not (Test-Path $exampleDir)) {
    New-Item -Path $exampleDir -ItemType Directory -Force | Out-Null
}
# Read and write with explicit encoding to avoid BOM issues
Get-Content ".env" -Encoding utf8 | Out-File -FilePath "$exampleDir/.env" -Encoding utf8 -Force

# Copy matrix_tool.py to the appropriate tools directory
if ($EXAMPLE -eq "ex1") {
    Write-Host "Copying matrix_tool.py file to examples/ex1/src/ex1/tools/matrix_tool/" -ForegroundColor Yellow
    $toolDir = "examples/ex1/src/ex1/tools/matrix_tool"
    if (-not (Test-Path $toolDir)) {
        New-Item -Path $toolDir -ItemType Directory -Force | Out-Null
    }
    Copy-Item "matrix_tool.py" -Destination "$toolDir/matrix_tool.py" -Force
} elseif ($EXAMPLE -eq "moderator") {
    Write-Host "Copying matrix_tool.py file to examples/moderator/src/moderator/tools/matrix_tool/" -ForegroundColor Yellow
    $toolDir = "examples/moderator/src/moderator/tools/matrix_tool"
    if (-not (Test-Path $toolDir)) {
        New-Item -Path $toolDir -ItemType Directory -Force | Out-Null
    }
    Copy-Item "matrix_tool.py" -Destination "$toolDir/matrix_tool.py" -Force
}

# Activate Python virtual environment if specified
if ($PYTHON_VENV_PATH -and (Test-Path "$PYTHON_VENV_PATH/Scripts/activate.ps1")) {
    Write-Host "Using Python virtual environment at: $PYTHON_VENV_PATH" -ForegroundColor Green
    & "$PYTHON_VENV_PATH/Scripts/activate.ps1"
} elseif ($PYTHON_VENV_PATH) {
    Write-Host "Warning: Virtual environment not found at $PYTHON_VENV_PATH" -ForegroundColor Yellow
    Write-Host "Falling back to system Python" -ForegroundColor Yellow
}

# Start the API server in the background
Write-Host "Starting Matrix API server..." -ForegroundColor Green
$apiProcess = Start-Process -FilePath "cargo" -ArgumentList "run" -NoNewWindow -PassThru

# Function to clean up on exit
function Cleanup {
    # Return to the original directory
    Set-Location -Path $PSScriptRoot

    # Stop the API server
    Write-Host "Stopping API server..." -ForegroundColor Yellow
    if ($apiProcess -ne $null -and -not $apiProcess.HasExited) {
        Stop-Process -Id $apiProcess.Id -Force -ErrorAction SilentlyContinue
    }

    # Deactivate virtual environment if activated
    if (Get-Command "deactivate" -ErrorAction SilentlyContinue) {
        deactivate
    }

    Write-Host "API server stopped." -ForegroundColor Green
}

# Register cleanup to run on script exit
$null = Register-EngineEvent -SourceIdentifier PowerShell.Exiting -Action { Cleanup }

# Wait for the API to start
Write-Host "Waiting for API server to start (this may take up to 30 seconds)..." -ForegroundColor Yellow
$startTime = Get-Date
$timeout = 30  # seconds
$serverStarted = $false

while (((Get-Date) - $startTime).TotalSeconds -lt $timeout) {
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:8080/status" -Method Get -TimeoutSec 1
        if ($response.status -eq "running") {
            $serverStarted = $true
            Write-Host "API server started successfully!" -ForegroundColor Green
            break
        }
    } catch {
        # Server not responding yet, which is expected
        Start-Sleep -Milliseconds 500
    }
}

if (-not $serverStarted) {
    Write-Host "Warning: Could not confirm API server started within timeout." -ForegroundColor Yellow
    Write-Host "The server might still be starting up. Will attempt to continue." -ForegroundColor Yellow
}

# Run the CrewAI example
Write-Host "Running CrewAI $EXAMPLE example..." -ForegroundColor Green
try {
    Set-Location -Path "examples/$EXAMPLE"
    if (Get-Command "crewai" -ErrorAction SilentlyContinue) {
        crewai run
    } else {
        Write-Host "CrewAI command not found. Please install CrewAI:" -ForegroundColor Red
        Write-Host "pip install crewai[tools]" -ForegroundColor Yellow
    }
} catch {
    Write-Host "Error running CrewAI example: $_" -ForegroundColor Red
} finally {
    # Ensure cleanup happens even if there's an error
    Cleanup
}

Write-Host "Done!" -ForegroundColor Green
