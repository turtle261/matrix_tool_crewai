import requests
import webbrowser
import time
import platform
import subprocess
import os

BASE_URL = "http://localhost:8080"

# Start SSO login
response = requests.post(f"{BASE_URL}/login/sso/start")
if response.status_code != 200:
    print(f"Failed to start SSO: {response.text}")
    exit(1)
data = response.json()
session_id = data["session_id"]
sso_url = data["sso_url"]

print(f"Opening SSO URL in browser: {sso_url}")

# Different browser opening methods based on platform, with Windows-specific handling
if platform.system() == "Windows":
    try:
        # Try to use the default browser
        os.startfile(sso_url)
    except Exception as e:
        print(f"Failed to open with os.startfile: {e}")
        try:
            # Fallback to webbrowser module
            webbrowser.open(sso_url)
        except Exception as e2:
            print(f"Failed to open with webbrowser: {e2}")
            # Last resort - use start command directly
            subprocess.run(['start', sso_url], shell=True)
else:
    # For non-Windows platforms
    webbrowser.open(sso_url)

# Poll for login completion
print("Waiting for SSO login to complete (please complete login in browser)...")
while True:
    response = requests.get(f"{BASE_URL}/login/status/{session_id}")
    if response.status_code != 200:
        print(f"Status check failed: {response.text}")
        exit(1)
    status = response.json()["status"]
    if status == "logged_in":
        print("Login successful!")
        break
    elif status == "error":
        print(f"Login failed: {response.json().get('error', 'Unknown error')}")
        exit(1)
    time.sleep(1)

# Sync
print("Synchronizing with Matrix server (this may take a moment)...")
response = requests.get(f"{BASE_URL}/sync/{session_id}")
if response.status_code != 200:
    print(f"Sync failed: {response.text}")
    exit(1)

sync_data = response.json()
if "error" in sync_data:
    print(f"Warning during sync: {sync_data['error']}")
    print("Continuing with limited functionality...")
else:
    print("Sync completed successfully")

# Get rooms
print("\nFetching rooms...")
response = requests.get(f"{BASE_URL}/rooms/{session_id}")
if response.status_code != 200:
    print(f"Failed to get rooms: {response.text}")
    exit(1)
rooms = response.json()
print(f"Number of rooms: {len(rooms)}")

if len(rooms) == 0:
    print("No rooms found. This could be because you haven't joined any rooms or due to sync limitations.")
    exit(0)

# Process rooms with better error handling
for room in rooms:
    room_id = room["room_id"]
    name = room.get("name", "Unknown Room")
    print(f"\nRoom: {name} ({room_id})")
    
    # Get messages with timeout handling
    print("  Fetching messages...")
    try:
        response = requests.get(f"{BASE_URL}/rooms/{session_id}/{room_id}/messages", timeout=15)
        if response.status_code != 200:
            print(f"  Failed to get messages: {response.text}")
            continue
            
        messages = response.json()
        if not messages:
            print("  No messages found in this room.")
            continue
            
        print(f"  Found {len(messages)} messages:")
        for msg in messages:
            sender = msg['sender']
            body = msg['body']
            # Truncate very long messages for display
            if len(body) > 100:
                body = body[:97] + "..."
            print(f"    {sender}: {body}")
    except requests.exceptions.Timeout:
        print("  Timed out while fetching messages. Try again later.")
    except Exception as e:
        print(f"  Error fetching messages: {e}")

print("\nTest completed!")