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
    - watch_room [room_id]: Watch a room for new messages
    - redact_message [room_id] [event_id] [reason]: Redact (delete) a message from a room
    - ban_user [room_id] [user_id] [reason]: Ban a user from a room
    """
    
    base_url: str = "http://localhost:8080"
    session_id: Optional[str] = None
    next_batch: Optional[str] = None
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
        # If we already have a session ID, verify it's still valid
        if self.session_id:
            try:
                status_response = requests.get(
                    f"{self.base_url}/login/status/{self.session_id}"
                )
                status_data = status_response.json()
                
                if status_data.get("status") == "logged_in":
                    print("Using existing Matrix session.")
                    return self.session_id
                else:
                    # Session not valid, reset it
                    print("Existing session is not valid. Starting new login...")
                    self.session_id = None
            except Exception as e:
                # On error, reset session and try login again
                print(f"Error checking session status: {e}")
                self.session_id = None
        
        # Start new login process
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
        
        try:
            join_response = requests.post(
                f"{self.base_url}/rooms/{self.session_id}/join/{room_id}"
            )
            
            if join_response.status_code == 200:
                return f"Successfully joined room {room_id}"
            else:
                result = join_response.json()
                return f"Failed to join room: {result.get('error', 'Unknown error')}"
        except Exception as e:
            return f"Error joining room: {e}"
    
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
            elif command == "watch_room":
                # Check for a loop count parameter
                loop_count = 1
                since_token = None
                # Check if we have a since token or loop count
                if len(task_parts) >= 3:
                    try:
                        loop_count = int(task_parts[2])
                    except ValueError:
                        # Not a number, treat it as a since token
                        since_token = task_parts[2]
                        # Check for both since token and loop count
                        if len(task_parts) >= 4:
                            try:
                                loop_count = int(task_parts[3])
                            except ValueError:
                                pass
                return self.watch_room(room_id, since_token, loop_count)
        
        # Commands requiring two arguments (room_id and message/event_id/user_id)
        if len(task_parts) >= 3:
            room_id = task_parts[1]
            second_param = task_parts[2]
            
            if command == "send_message":
                return self.send_message_task(room_id, second_param)
            elif command == "send_hi":  # For backward compatibility
                return self.send_message_task(room_id, "Hi from CrewAI!")
            elif command == "redact_message":
                # Check if we have a reason
                reason = None
                if " " in second_param:
                    event_id, reason = second_param.split(" ", 1)
                else:
                    event_id = second_param
                return self.redact_message(room_id, event_id, reason)
            elif command == "ban_user":
                # Check if we have a reason
                reason = None
                if " " in second_param:
                    user_id, reason = second_param.split(" ", 1)
                else:
                    user_id = second_param
                return self.ban_user(room_id, user_id, reason)
        
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
        - watch_room [room_id] [loop_count]: Watch a room for new messages
        - redact_message [room_id] [event_id] [reason]: Redact (delete) a message
        - ban_user [room_id] [user_id] [reason]: Ban a user from a room
        """
    
    def watch_room(self, room_id: str, since_token: Optional[str] = None, loop_count: int = 1) -> str:
        """Watch a room for new messages, returning when a new message arrives."""
        self._login()
        
        try:
            # Store the latest message details to detect changes
            latest_message_details = {}
            
            # Loop for the specified number of times
            for i in range(loop_count):
                # Build the URL with the optional since token
                url = f"{self.base_url}/rooms/{self.session_id}/{room_id}/watch"
                params = {}
                
                # Use next_batch if available, otherwise use provided since_token
                if self.next_batch:
                    params["since"] = self.next_batch
                elif since_token:
                    params["since"] = since_token
                
                # Set a reasonable timeout (20 seconds)
                params["timeout"] = 20
                
                # Make the request with timeout
                try:
                    response = requests.get(url, params=params, timeout=25)
                except requests.exceptions.Timeout:
                    # Handle timeout gracefully - just continue to the next iteration
                    print("Polling timed out, trying again...")
                    # Add a delay after timeout to avoid hammering the server
                    time.sleep(5)
                    continue
                except requests.exceptions.ConnectionError:
                    # Handle connection error - try to re-login
                    print("Connection error. Attempting to re-login...")
                    try:
                        self._login()
                        # Add a delay after connection error
                        time.sleep(5)
                        continue
                    except Exception as e:
                        print(f"Re-login failed: {e}")
                        time.sleep(10)  # Wait longer after login failure
                        continue
                
                if response.status_code != 200:
                    print(f"Error watching room: {response.status_code} - {response.text}")
                    # If we got a 401, try to re-login
                    if response.status_code == 401:
                        print("Session may have expired. Attempting to re-login...")
                        try:
                            self._login()
                        except Exception as e:
                            print(f"Re-login failed: {e}")
                    # Add a delay after error
                    time.sleep(5)
                    continue
                
                # Process the response
                try:
                    result = response.json()
                except ValueError as e:
                    print(f"Error parsing JSON: {e}")
                    time.sleep(5)  # Increased delay after JSON parsing error
                    continue
                
                # Store the next_batch token for future calls
                if "next_batch" in result:
                    self.next_batch = result.get("next_batch")
                
                # Check if we have new messages
                if result.get("has_new_messages", False):
                    messages = result.get("messages", [])
                    if not messages:
                        if i == loop_count - 1:
                            return "No new messages in the room."
                        else:
                            print("No messages in response, continuing to watch...")
                            time.sleep(3)  # Increased delay when no messages are found
                            continue
                    
                    # Format the latest message
                    latest_msg = messages[0]  # The most recent message should be first
                    sender = latest_msg.get("sender", "Unknown")
                    body = latest_msg.get("body", "No content")
                    event_id = latest_msg.get("event_id", "Unknown")
                    
                    # Check if this is a new message by comparing sender and event ID
                    msg_key = f"{sender}:{event_id}"
                    if msg_key in latest_message_details:
                        if i == loop_count - 1:
                            return "No new messages since last check."
                        else:
                            print(f"Already processed message {event_id}, skipping...")
                            time.sleep(3)  # Increased delay when seeing the same message
                            continue
                    
                    # Store the latest message details
                    latest_message_details[msg_key] = {
                        'sender': sender,
                        'body': body,
                        'event_id': event_id
                    }
                    
                    # Return a formatted response with the event_id for possible moderation
                    return f"New message in room {room_id}:\nSender: {sender}\nContent: {body}\nEvent ID: {event_id}"
                else:
                    if i == loop_count - 1:
                        return "No new messages since last check."
                    else:
                        print("No new messages, continuing to watch...")
                        time.sleep(3)  # Increased delay when no new messages
                        continue
            
            return "No new messages after multiple checks."
        except Exception as e:
            return f"Error watching room: {e}"
    
    def redact_message(self, room_id: str, event_id: str, reason: Optional[str] = None) -> str:
        """Redact (delete) a message from a room."""
        self._login()
        
        try:
            # Prepare the payload
            payload = {}
            if reason:
                payload["reason"] = reason
            
            # Make the request
            response = requests.post(
                f"{self.base_url}/rooms/{self.session_id}/{room_id}/redact/{event_id}",
                json=payload
            )
            
            if response.status_code == 200:
                result = response.json()
                return f"Successfully redacted message {event_id} from room {room_id}"
            else:
                error_text = response.text
                try:
                    error = response.json().get("error", "Unknown error")
                except:
                    error = error_text
                return f"Failed to redact message: {error}"
        except Exception as e:
            return f"Error redacting message: {e}"
    
    def ban_user(self, room_id: str, user_id: str, reason: Optional[str] = None) -> str:
        """Ban a user from a room."""
        self._login()
        
        try:
            # Prepare the payload
            payload = {}
            if reason:
                payload["reason"] = reason
            
            # Make the request
            response = requests.post(
                f"{self.base_url}/rooms/{self.session_id}/{room_id}/ban/{user_id}",
                json=payload
            )
            
            if response.status_code == 200:
                return f"Successfully banned user {user_id} from room {room_id}"
            else:
                error_text = response.text
                try:
                    error = response.json().get("error", "Unknown error")
                except:
                    error = error_text
                return f"Failed to ban user: {error}"
        except Exception as e:
            return f"Error banning user: {e}" 