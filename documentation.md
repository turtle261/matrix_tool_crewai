# Matrix Tool for CrewAI Documentation

This document provides detailed information about the Matrix Tool for CrewAI, a tool that allows CrewAI agents to interact with Matrix chat rooms.

## Table of Contents

- [Architecture](#architecture)
- [API Reference](#api-reference)
- [Configuration](#configuration)
- [Advanced Usage](#advanced-usage)
- [Troubleshooting](#troubleshooting)
- [Development Guide](#development-guide)

## Architecture

The Matrix Tool for CrewAI consists of two main components:

1. **Rust Backend**: A high-performance backend that handles the Matrix protocol communication, authentication, and message processing.
2. **Python Bindings**: A Python interface that integrates with CrewAI and provides a simple API for agents to use.

The architecture follows this flow:

```
CrewAI Agent → Python MatrixTool → Rust Backend → Matrix Homeserver
```

## API Reference

### MatrixTool Class

The main class that CrewAI agents interact with.

```python
from matrix_tool import MatrixTool

# Initialize the tool
matrix_tool = MatrixTool(base_url="http://localhost:8080")
```

#### Parameters

- `base_url` (str, optional): The URL of the Matrix API backend. Defaults to "http://localhost:8080".
- `session_id` (str, optional): An existing session ID for authentication. If not provided, a new session will be created.

#### Methods

##### `list_rooms()`

Lists all rooms the agent is a member of.

**Returns**: A string containing the list of rooms.

##### `count_rooms()`

Counts the number of rooms the agent is in.

**Returns**: A string containing the count of rooms.

##### `get_room_details(room_id: str)`

Gets details about a specific room.

**Parameters**:
- `room_id` (str): The ID of the room to get details for.

**Returns**: A string containing the room details.

##### `get_messages(room_id: str, limit: int = 20)`

Gets messages from a specific room.

**Parameters**:
- `room_id` (str): The ID of the room to get messages from.
- `limit` (int, optional): The maximum number of messages to retrieve. Defaults to 20.

**Returns**: A list of message dictionaries.

##### `receive_messages(room_id: str)`

Receives and formats messages from a specific room.

**Parameters**:
- `room_id` (str): The ID of the room to receive messages from.

**Returns**: A formatted string containing the messages.

##### `send_message(room_id: str, message: str)`

Sends a message to a specific room.

**Parameters**:
- `room_id` (str): The ID of the room to send the message to.
- `message` (str): The message to send.

**Returns**: A dictionary containing the result of the send operation.

##### `join_room(room_id: str)`

Joins a specific Matrix room.

**Parameters**:
- `room_id` (str): The ID of the room to join.

**Returns**: A string indicating the result of the join operation.

##### `leave_room(room_id: str)`

Leaves a specific Matrix room.

**Parameters**:
- `room_id` (str): The ID of the room to leave.

**Returns**: A string indicating the result of the leave operation.

### Rust API Endpoints

The Rust backend provides the following HTTP endpoints:

- `POST /login/sso/start`: Starts the SSO login process.
- `GET /login/status/{session_id}`: Checks the status of a login session.
- `GET /sync/{session_id}`: Syncs the client state with the server.
- `GET /rooms/{session_id}`: Gets the list of rooms the user is in.
- `GET /rooms/{session_id}/{room_id}/messages`: Gets messages from a specific room.
- `POST /rooms/{session_id}/{room_id}/send`: Sends a message to a specific room.

## Configuration

### config.toml

The Matrix Tool can be configured using a `config.toml` file:

```toml
[homeserver]
url = "https://matrix.org"  # URL of your Matrix homeserver
```

This simple configuration specifies the Matrix homeserver URL that the tool will connect to.

## Advanced Usage

### Integration with Multiple Agents

You can create multiple instances of the MatrixTool for different agents:

```python
from crewai import Agent, Crew
from matrix_tool import MatrixTool

# Create tools for each agent
support_tool = MatrixTool()
analytics_tool = MatrixTool()

# Create agents
support_agent = Agent(
    name="Support Agent",
    role="Handle support requests",
    tools=[support_tool],
    # ...
)

analytics_agent = Agent(
    name="Analytics Agent",
    role="Analyze chat patterns",
    tools=[analytics_tool],
    # ...
)

# Create a crew with both agents
crew = Crew(
    agents=[support_agent, analytics_agent],
    # ...
)
```

### Custom Message Processing

You can implement custom message processing by extending the MatrixTool class:

```python
from matrix_tool import MatrixTool

class CustomMatrixTool(MatrixTool):
    def process_message(self, message):
        """Custom message processing logic."""
        # Extract message content
        sender = message.get("sender", "Unknown")
        body = message.get("body", "")
        
        # Implement custom logic
        if "help" in body.lower():
            return self.send_message(message["room_id"], "How can I assist you?")
        
        # Default processing
        return super().process_message(message)
```

## Troubleshooting

### Common Issues

#### Login Failures

If you're experiencing login failures:

1. Ensure your Matrix homeserver is accessible
2. Check that SSO is properly configured on your homeserver
3. Verify network connectivity to the homeserver
4. Check the logs for detailed error messages

#### Message Sending Failures

If messages aren't being sent:

1. Verify you have the correct room ID
2. Ensure you have permission to send messages in the room
3. Check your authentication status
4. Verify the Matrix API backend is running

### Logging

To enable detailed logging, set the log level in your config.toml:

```toml
[logging]
level = "debug"
file = "matrix_tool.log"
```

## Development Guide

### Building from Source

#### Prerequisites

- Rust 1.56 or newer
- Cargo
- Python 3.10 or newer
- pip

#### Dependencies

The project requires the following Python dependencies:

```
crewai==0.108.0
python-dotenv==1.0.0
requests==2.31.0
google-generativeai>=0.3.0
```

#### Build Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/turtle261/matrix_tool_crewai.git
   cd matrix_tool_crewai
   ```

2. Build the Rust backend:
   ```bash
   cargo build --release
   ```

3. Install Python dependencies:
   ```bash
   pip install -r requirements.txt
   ```

### Running Tests

To run the Rust tests:

```bash
cargo test
```

To run the Python tests:

```bash
pytest
```

### Contributing

We welcome contributions to the Matrix Tool for CrewAI! Here are some areas where help is needed:

- Implementing additional Matrix API features
- Improving error handling and recovery
- Enhancing documentation
- Adding more examples
- Performance optimizations

Please see the [Contributing Guide](CONTRIBUTING.md) for more information on how to contribute.

### Project Roadmap

Future plans for the Matrix Tool include:

- End-to-end encryption support
- File transfer capabilities
- Voice and video call integration
- Enhanced room management features
- Direct messaging support
- Multi-homeserver support