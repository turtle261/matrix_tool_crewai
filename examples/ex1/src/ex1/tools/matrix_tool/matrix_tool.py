import requests
import webbrowser
import time
import platform
import subprocess
import os
from typing import Dict, List, Optional, Any, Union
from crewai.tools import BaseTool
from pydantic import Field, validator

class MatrixTool(BaseTool):
    """A tool for interacting with Matrix using the local API."""
    
    name: str = "MatrixTool"
    description: str = """
    Use this tool to interact with Matrix chat rooms.
    
    Available tasks:
    - list_rooms: List all rooms you're a member of
    - count_rooms: Count the number of rooms you're in
    - get_room_details [room_id]: Get details about a specific room
    - get_messages [room_id]: Get messages from a specific room
    - send_message [room_id] [message]: Send a message to a specific room
    - join_room [room_id]: Join a specific Matrix room
    - leave_room [room_id]: Leave a specific Matrix room
    """
    
    base_url: str = "http://localhost:8080"
    session_id: Optional[str] = None
    task: Any = Field(
        default="list_rooms",
        description="The Matrix task to perform. See tool description for available options."
    )
    
    @validator('task')
    def validate_task(cls, v):
        """Ensure task is a string, handling different input formats."""
        # If it's already a string, return it
        if isinstance(v, str):
            return v
            
        # If it's a dictionary
        if isinstance(v, dict):
            # Extract from CrewAI format (common pattern)
            if 'description' in v:
                return v['description']
                
            # If there's only one key and value is a string, use that
            if len(v) == 1 and isinstance(list(v.values())[0], str):
                return list(v.values())[0]
        
        # For any other format, convert to string
        return str(v)
    
    def _login(self) -> str:
        """Start SSO login and return session_id."""
        if self.session_id:
            # Check if session is still valid
            try:
                requests.get(f"{self.base_url}/login/status/{self.session_id}")
                return self.session_id
            except:
                # Session might be invalid, reset and relogin
                self.session_id = None
        
        print("Starting Matrix login process...")
        try:
            # Start SSO login
            login_response = requests.post(f"{self.base_url}/login/sso/start")
            login_data = login_response.json()
            
            self.session_id = login_data["session_id"]
            sso_url = login_data["sso_url"]
            
            # Open browser for SSO login
            print(f"\n{'='*80}")
            print(f"Please complete SSO login in your browser at this URL:")
            print(f"{sso_url}")
            print(f"{'='*80}\n")
            webbrowser.open(sso_url)
            
            # Wait for login to complete with timeout
            timeout = 300  # seconds
            start_time = time.time()
            print("Waiting for login to complete...")
            while time.time() - start_time < timeout:
                time.sleep(3)  # Check every 3 seconds
                status_response = requests.get(
                    f"{self.base_url}/login/status/{self.session_id}"
                )
                status_data = status_response.json()
                
                if status_data.get("status") == "logged_in":
                    print("Login successful!")
                    return self.session_id
                elif status_data.get("status") == "error":
                    raise Exception(f"Login error: {status_data.get('error')}")
                
                print("Waiting for login to complete... Please finish the login in your browser.")
            
            raise Exception("Login timed out. Please try again.")
            
        except Exception as e:
            print(f"Error during login: {e}")
            raise
    
    def list_rooms(self) -> str:
        """List all rooms the user is in."""
        self._login()
        
        try:
            rooms = self.get_rooms()
            
            if not rooms:
                return "You are not in any Matrix rooms."
            
            room_details = []
            for room in rooms:
                room_id = room["room_id"]
                name = room.get("name", "Unnamed room")
                room_details.append(f"Room ID: {room_id}, Name: {name}")
            
            return f"You are in {len(rooms)} Matrix rooms:\n" + "\n".join(room_details)
        except Exception as e:
            return f"Error listing rooms: {e}"
    
    def count_rooms(self) -> str:
        """Count the number of rooms the user is in."""
        self._login()
        
        try:
            rooms = self.get_rooms()
            return f"You are in {len(rooms)} Matrix rooms."
        except Exception as e:
            return f"Error counting rooms: {e}"
    
    def get_room_details(self, room_id: str) -> str:
        """Get details about a specific room."""
        self._login()
        
        try:
            rooms = self.get_rooms()
            
            for room in rooms:
                if room["room_id"] == room_id:
                    name = room.get("name", "Unnamed room")
                    return f"Room ID: {room_id}\nName: {name}"
            
            return f"Room {room_id} not found or you are not a member."
        except Exception as e:
            return f"Error getting room details: {e}"
    
    def join_room(self, room_id: str) -> str:
        """Join a specific Matrix room."""
        self._login()
        
        # Note: The Matrix API backend doesn't currently support joining rooms directly
        # This is a placeholder for when that functionality is added
        return f"Join room functionality not implemented in the backend API. Cannot join room {room_id} at this time."
    
    def leave_room(self, room_id: str) -> str:
        """Leave a specific Matrix room."""
        self._login()
        
        # Note: The Matrix API backend doesn't currently support leaving rooms directly
        # This is a placeholder for when that functionality is added
        return f"Leave room functionality not implemented in the backend API. Cannot leave room {room_id} at this time."
    
    def get_messages(self, room_id: str, limit: int = 20) -> List[Dict[str, Any]]:
        """Get messages from a specific room."""
        if not self.session_id:
            self._login()
        
        try:
            messages_response = requests.get(
                f"{self.base_url}/rooms/{self.session_id}/{room_id}/messages"
            )
            return messages_response.json()
        except Exception as e:
            print(f"Error getting messages: {e}")
            raise
    
    def receive_messages(self, room_id: str) -> str:
        """Receive and format messages from a specific room."""
        self._login()
        
        try:
            messages = self.get_messages(room_id)
            
            if not messages:
                return f"No messages found in room {room_id}."
            
            formatted_messages = []
            for msg in messages:
                sender = msg.get("sender", "Unknown")
                body = msg.get("body", "No content")
                timestamp = msg.get("timestamp", 0)
                
                # Convert timestamp to readable format
                try:
                    time_str = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(timestamp/1000))
                except:
                    time_str = "Unknown time"
                
                formatted_messages.append(f"[{time_str}] {sender}: {body}")
            
            return f"Messages from room {room_id}:\n" + "\n".join(formatted_messages)
        except Exception as e:
            return f"Error receiving messages: {e}"
    
    def send_message(self, room_id: str, message: str) -> Dict[str, Any]:
        """Send a message to a specific room."""
        if not self.session_id:
            self._login()
        
        try:
            message_response = requests.post(
                f"{self.base_url}/rooms/{self.session_id}/{room_id}/send",
                json={"body": message}
            )
            return message_response.json()
        except Exception as e:
            print(f"Error sending message: {e}")
            raise
    
    def send_message_task(self, room_id: str, message: str) -> str:
        """Send a message to a specific room and return a formatted response."""
        self._login()
        
        try:
            result = self.send_message(room_id, message)
            return f"Message sent successfully to room {room_id}. Event ID: {result.get('event_id', 'unknown')}"
        except Exception as e:
            return f"Error sending message: {e}"
    
    def get_rooms(self) -> List[Dict[str, str]]:
        """Get the list of rooms the user is in."""
        if not self.session_id:
            self._login()
        
        try:
            # Sync to ensure we have the latest room list
            sync_response = requests.get(
                f"{self.base_url}/sync/{self.session_id}"
            )
            
            # Now get rooms
            rooms_response = requests.get(
                f"{self.base_url}/rooms/{self.session_id}"
            )
            return rooms_response.json()
        except Exception as e:
            print(f"Error getting rooms: {e}")
            raise
    
    def run(self) -> str:
        """Run a Matrix task."""
        # Log invocation for debugging
        print(f"MatrixTool called with task: {self.task}")
        
        # Process task - handle different formats
        try:
            # Ensure login happens first
            self._login()
            
            # Already processed by validator but double check
            if isinstance(self.task, dict):
                # Try to extract a string from the dictionary
                if 'description' in self.task:
                    task_str = self.task['description']
                else:
                    # Just convert to string if nothing better
                    task_str = str(self.task)
            else:
                task_str = str(self.task)
                
            print(f"Executing Matrix task: {task_str}")
            return self._run(task_str)
        except Exception as e:
            error_msg = f"Error: {str(e)}"
            print(error_msg)
            return error_msg
    
    def _run(self, task: str) -> str:
        """Run a specific Matrix task."""
        # Parse the task 
        task_parts = task.strip().split(" ", 2)  # Split into max 3 parts: command, param1, rest
        if not task_parts:
            return self._help_message()
        
        command = task_parts[0].lower()
        
        # Map commands to methods
        command_map = {
            "list_rooms": self.list_rooms,
            "count_rooms": self.count_rooms,
        }
        
        # Simple commands (no arguments)
        if command in command_map:
            return command_map[command]()
        
        # Commands requiring one argument (room_id)
        if len(task_parts) >= 2:
            room_id = task_parts[1]
            
            if command == "get_room_details":
                return self.get_room_details(room_id)
            elif command == "receive_messages" or command == "get_messages":
                return self.receive_messages(room_id)
            elif command == "join_room":
                return self.join_room(room_id)
            elif command == "leave_room":
                return self.leave_room(room_id)
        
        # Commands requiring two arguments (room_id and message)
        if len(task_parts) >= 3:
            room_id = task_parts[1]
            message = task_parts[2]
            
            if command == "send_message":
                return self.send_message_task(room_id, message)
            elif command == "send_hi":  # For backward compatibility
                return self.send_message_task(room_id, "Hi from CrewAI!")
        
        # If we get here, the command wasn't recognized
        return self._help_message()
    
    def _help_message(self) -> str:
        """Return a help message listing available commands."""
        return """
        Available Matrix commands:
        - list_rooms: List all rooms you're a member of
        - count_rooms: Count the number of rooms you're in
        - get_room_details [room_id]: Get details about a specific room
        - get_messages [room_id]: Get messages from a specific room
        - receive_messages [room_id]: Same as get_messages
        - send_message [room_id] [message]: Send a message to a specific room
        - join_room [room_id]: Join a specific Matrix room
        - leave_room [room_id]: Leave a specific Matrix room
        """ 