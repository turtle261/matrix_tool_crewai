#!/bin/bash
# POSIX shell script to run the Matrix agent test

# Configuration - Edit these variables as needed
# Set PYTHON_VENV_PATH to your virtual environment path, or leave empty to use system Python
PYTHON_VENV_PATH="../mo"
# Example: PYTHON_VENV_PATH="./venv"  # Relative path
# Example: PYTHON_VENV_PATH="/home/user/myenv"  # Absolute path

# Function to display colored text
print_colored() {
  case "$2" in
    "green") color="\033[0;32m" ;;
    "yellow") color="\033[0;33m" ;;
    "red") color="\033[0;31m" ;;
    *) color="\033[0m" ;;
  esac
  
  echo -e "${color}$1\033[0m"
}

# Setup Python environment
if [ -n "$PYTHON_VENV_PATH" ]; then
  if [ -f "$PYTHON_VENV_PATH/bin/activate" ]; then
    print_colored "Using Python virtual environment at: $PYTHON_VENV_PATH" "green"
    source "$PYTHON_VENV_PATH/bin/activate"
  else
    print_colored "Warning: Virtual environment not found at $PYTHON_VENV_PATH" "yellow"
    print_colored "Falling back to system Python" "yellow"
  fi
fi

print_colored "Compiling Rust API Backend" "green"
cargo build

print_colored "Starting Matrix test..." "green"

# Check for .env file
if [ ! -f ".env" ]; then
  print_colored "Warning: .env file not found!" "red"
  print_colored "Creating a sample .env file - please edit with your actual API key" "yellow"
  echo "GEMINI_API_KEY=your_api_key_here" > .env
  print_colored "Please edit the .env file with your Gemini API key and run this script again." "red"
  exit 1
fi

# Copy .env to the examples/ex1 directory
print_colored "Copying .env file to examples/ex1" "yellow"
mkdir -p examples/ex1
cp .env examples/ex1/

print_colored "Copying matrix_tool.py file to examples/ex1/src/ex1/tools/" "yellow"
cp matrix_tool.py examples/ex1/src/ex1/tools/matrix_tool/matrix_tool.py

# Start the API server in the background
print_colored "Starting Matrix API server..." "green"
cargo run &
API_PID=$!

# Clean up the API server process when the script exits
cleanup() {
  cd "$ORIG_DIR" 2>/dev/null || true
  
  print_colored "Stopping API server..." "yellow"
  if [ -n "$API_PID" ]; then
    kill $API_PID 2>/dev/null || true
  fi
  
  # Deactivate Python virtual environment if it was activated
  if [ -n "$PYTHON_VENV_PATH" ] && [ -f "$PYTHON_VENV_PATH/bin/activate" ]; then
    if type deactivate >/dev/null 2>&1; then
      deactivate
    fi
  fi
  
  print_colored "API server stopped." "green"
}

# Save original directory
ORIG_DIR="$(pwd)"

# Register the cleanup function to run on script exit
trap cleanup EXIT

# Wait for the API to start
print_colored "Waiting for API server to start (this may take up to 30 seconds)..." "yellow"
start_time=$(date +%s)
timeout=30 # seconds
server_started=false

while [ $(($(date +%s) - start_time)) -lt $timeout ]; do
  # Check if the API is responding
  if curl -s "http://localhost:8080/status" 2>/dev/null | grep -q "running"; then
    server_started=true
    print_colored "API server started successfully!" "green"
    break
  fi
  # Server not responding yet, which is expected
  sleep 0.5
done

if [ "$server_started" = false ]; then
  print_colored "Warning: Could not confirm API server started within timeout." "yellow"
  print_colored "The server might still be starting up. Will attempt to continue." "yellow"
fi

# Run the CrewAI example - simpler approach like in run.ps1
print_colored "Running CrewAI Matrix example..." "green"
cd examples/ex1

# Simple approach to run CrewAI
if command -v crewai >/dev/null 2>&1; then
  crewai run || print_colored "Error running CrewAI example" "red"
else
  print_colored "CrewAI command not found. Please install CrewAI:" "red"
  print_colored "pip install crewai[tools]" "yellow"
fi

# Return to the original directory
cd "$ORIG_DIR"

print_colored "Done!" "green" 
