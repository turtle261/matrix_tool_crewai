#!/usr/bin/env python3
"""
Matrix Chat Room Moderator using CrewAI

This script runs a moderation agent that enforces PG-14 standards in a Matrix chat room.
"""
import sys
import warnings
import os
from dotenv import load_dotenv

from moderator.crew import ModeratorCrew

warnings.filterwarnings("ignore", category=SyntaxWarning, module="pysbd")

# Load environment variables from .env file
load_dotenv()

# Check for required API key
def check_api_key():
    api_key = os.getenv("GEMINI_API_KEY")
    if not api_key:
        print("Error: GEMINI_API_KEY environment variable is not set.")
        print("Please add a valid API key to your .env file.")
        sys.exit(1)

def run():
    """
    Run the Matrix Moderator crew.
    """
    check_api_key()
    
    # Note: No specific inputs needed for this crew
    print("Starting Matrix Moderator crew...")
    print("Note: A browser window will open for Matrix login. Please complete the login process.")
    print("If no browser opens automatically, please manually open the SSO URL that will be displayed.")
    
    moderator = ModeratorCrew()
    
    try:
        # First join the room using the crew
        crew_instance = moderator.crew()
        result = crew_instance.kickoff()
        print("Initial tasks completed successfully.")
        
        # Now start continuous monitoring - this will run until user presses 'q'
        print("\n=== STARTING CONTINUOUS MODERATION ===")
        print("Press 'q' at any time to stop moderation and exit.")
        moderator.continuous_monitor("!iYYuXGoKsPtMPlJEub:mozilla.org")
    except KeyboardInterrupt:
        print("\nModeration stopped by user (Ctrl+C).")
    except Exception as e:
        print(f"An error occurred during moderation: {e}")
    
    print("Matrix moderation completed.")
    return result

if __name__ == "__main__":
    run() 