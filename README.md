# Matrix Tool for CrewAI

![Version](https://img.shields.io/badge/version-0.1.2-blue.svg)
![License](https://img.shields.io/badge/license-AGPL--3.0-green.svg)
![Tests](https://img.shields.io/badge/build-passing-green.svg)

A fully featured Matrix Chat tool allowing CrewAI agents (LLM agnostic) to send, receive messages, join and leave rooms, etc. as if they were a person using a client.

## Features

- **Matrix Integration**: Connect CrewAI agents to Matrix chat rooms
- **Message Operations**: Send and receive messages in Matrix rooms
- **Room Management**: List, join, and leave Matrix rooms
- **SSO Authentication**: Secure login via Single Sign-On
- **LLM Agnostic**: Works with any LLM supported by CrewAI
- **Rust Backend**: High-performance Rust backend with Python bindings

## Installation

```bash
# Clone the repository
git clone https://github.com/turtle261/matrix_tool_crewai.git

# Navigate to the project directory
cd matrix_tool_crewai

# Install Python dependencies
pip install -r requirements.txt

# Build the Rust backend
cargo build --release
```

## Usage

### Basic Setup

```python
from crewai import Agent, Task
from matrix_tool import MatrixTool

# Initialize the Matrix tool
matrix_tool = MatrixTool()

# Create an agent with the Matrix tool
agent = Agent(
    name="Matrix Agent",
    role="An agent that interacts with Matrix chat",
    goal="Monitor and respond to Matrix messages",
    backstory="I am an AI assistant that helps manage Matrix communications",
    tools=[matrix_tool],
    verbose=True
)

# Create a task for the agent
task = Task(
    description="Check for new messages in the Matrix room and respond appropriately",
    agent=agent
)

# The agent will now be able to interact with Matrix
```

### Available Operations

The Matrix tool supports the following operations:

- `list_rooms`: List all rooms the agent is a member of
- `count_rooms`: Count the number of rooms the agent is in
- `get_room_details [room_id]`: Get details about a specific room
- `get_messages [room_id]`: Get messages from a specific room
- `send_message [room_id] [message]`: Send a message to a specific room
- `join_room [room_id]`: Join a specific Matrix room
- `leave_room [room_id]`: Leave a specific Matrix room

## Examples

Check out the [examples](./examples) directory for complete working examples:

- [Ex1](./examples/ex1): Basic Matrix integration with CrewAI Agent using Gemini

## Configuration

Create a `config.toml` file with your Matrix server settings:

```toml
[matrix]
homeserver = "https://matrix.org"
# Additional configuration options...
```

## License

This project is licensed under the [AGPL-3.0 License](LICENSE).

## Contributing

Contributions are welcome! This is a community-driven project, and we appreciate any help in making it better.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup
For debian: sudo apt install build-essential libssl-dev pkg-config libsqlite3-dev 
For windows (using chocolatey): choco install sqlite3
```bash
pip install -r requirements.txt

# Build the Rust backend in debug mode
cargo build
# Test that everything works properly, SSO opens browser, etc.
cargo test
./run.sh # or ./run.ps1
```
For reference: `run.ps1` will run the crewai example agent, which performs simple tasks on matrix. 
`cargo test` will run the API backend tests, to ensure that the agent tool will not have backend issues. 

---

Made by 'turtle261'
