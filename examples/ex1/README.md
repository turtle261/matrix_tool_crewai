# Ex1 Crew

Welcome to the Ex1 Crew project, powered by [crewAI](https://crewai.com). This template is designed to help you set up a multi-agent AI system with ease, leveraging the powerful and flexible framework provided by crewAI. Our goal is to enable your agents to collaborate effectively on complex tasks, maximizing their collective intelligence and capabilities.

# Matrix Tool Example

This is an example project that demonstrates how to use CrewAI with the Matrix API tool. This example shows how to:

1. Login to Matrix via SSO
2. Explore available rooms
3. Analyze messages in rooms
4. Send messages to rooms

## Prerequisites

- Python 3.10 or newer
- Rust (for the Matrix API backend)
- A Gemini API key
- SQLite

## Setup

1. Create a `.env` file in this directory with your Gemini API key:
   ```
   GEMINI_API_KEY=your_api_key_here
   ```

2. Make sure you're in the root directory of the matrixtool project, not in this examples directory.

3. The `run.ps1` script in the root directory will:
   - Build the Rust API backend
   - Start the Matrix API server
   - Run this CrewAI example

## Running the Example

From the root directory of the matrixtool project, run:

```powershell
./run.ps1
```

When the example runs:
1. A Matrix API server will start on localhost:8080
2. A browser window will open for Matrix SSO login
3. Complete the login process in your browser
4. The CrewAI agents will then analyze your Matrix rooms and communicate using the Matrix API

## What This Example Demonstrates

- How to integrate external APIs as CrewAI tools
- How to manage authentication flows in agents
- How to create agents that can analyze data and communicate via APIs
- How to properly structure a CrewAI project with tools
