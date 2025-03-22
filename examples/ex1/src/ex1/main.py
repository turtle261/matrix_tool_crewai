#!/usr/bin/env python
import sys
import warnings
import os
from dotenv import load_dotenv

from ex1.crew import Ex1

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
    Run the Matrix Tool example crew.
    """
    check_api_key()
    
    # Note: No specific inputs needed for this crew
    print("Starting Matrix Tool example crew...")
    print("Note: A browser window will open for Matrix login. Please complete the login process.")
    print("If no browser opens automatically, please manually open the SSO URL that will be displayed.")
    
    Ex1().crew().kickoff()


def train():
    """
    Train the crew for a given number of iterations.
    """
    check_api_key()
    
    try:
        Ex1().crew().train(n_iterations=int(sys.argv[1]), filename=sys.argv[2])
    except Exception as e:
        raise Exception(f"An error occurred while training the crew: {e}")

def replay():
    """
    Replay the crew execution from a specific task.
    """
    check_api_key()
    
    try:
        Ex1().crew().replay(task_id=sys.argv[1])
    except Exception as e:
        raise Exception(f"An error occurred while replaying the crew: {e}")

def test():
    """
    Test the crew execution and returns the results.
    """
    check_api_key()
    
    try:
        Ex1().crew().test(n_iterations=int(sys.argv[1]), openai_model_name=sys.argv[2])
    except Exception as e:
        raise Exception(f"An error occurred while testing the crew: {e}")
