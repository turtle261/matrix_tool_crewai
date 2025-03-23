# PowerShell script to run the Matrix agent test

Write-Host "Compiling Rust API Backend" -ForegroundColor Green
cargo build
Write-Host "Starting Matrix test..." -ForegroundColor Green

# Check for .env file
if (-not (Test-Path ".env")) {
    Write-Host "Warning: .env file not found!" -ForegroundColor Red
    Write-Host "Creating a sample .env file - please edit with your actual API key" -ForegroundColor Yellow
    "GEMINI_API_KEY=your_api_key_here" | Out-File -FilePath ".env"
    Write-Host "Please edit the .env file with your Gemini API key and run this script again." -ForegroundColor Red
    exit
}

# Copy .env to the examples/ex1 directory
Write-Host "Copying .env file to examples/ex1" -ForegroundColor Yellow
Copy-Item ".env" -Destination "examples/ex1/" -Force
Write-Host "Copying matrix_tool.py file to examples/ex1/src/ex1/tools/" -ForegroundColor Yellow
Copy-Item "matrix_tool.py" -Destination "examples/ex1/src/ex1/tools/matrix_tool" -Force

# Start the API server in a background job
Write-Host "Starting Matrix API server..." -ForegroundColor Green
$apiProcess = Start-Process -FilePath "cargo" -ArgumentList "run" -NoNewWindow -PassThru

# Wait for the API to start
Write-Host "Waiting for API server to start (this may take up to 30 seconds)..." -ForegroundColor Yellow
$startTime = Get-Date
$timeout = 30 # seconds
$serverStarted = $false

while (((Get-Date) - $startTime).TotalSeconds -lt $timeout) {
    # Check if the API is responding
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
try {
    Write-Host "Running CrewAI Matrix example..." -ForegroundColor Green
    Set-Location -Path "examples/ex1"
    crewai run
} catch {
    Write-Host "Error running CrewAI example: $_" -ForegroundColor Red
} finally {
    # Return to the original directory
    Set-Location -Path "../../"
    
    # Stop the API server
    Write-Host "Stopping API server..." -ForegroundColor Yellow
    if ($apiProcess -ne $null) {
        Stop-Process -Id $apiProcess.Id -Force
    }
    Write-Host "API server stopped." -ForegroundColor Green
}
