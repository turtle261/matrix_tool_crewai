from crewai import Agent, Crew, Process, Task, LLM
from crewai.project import CrewBase, agent, crew, task, before_kickoff, after_kickoff
from moderator.tools.matrix_tool.matrix_tool import MatrixTool
import yaml
import os
import time
import pathlib
import threading
import signal
import sys
import atexit

# Global flag to control the monitoring loop
should_continue = True

def signal_handler(sig, frame):
    """Handle ctrl+c to gracefully exit"""
    global should_continue
    print("\nReceived signal to stop. Exiting gracefully...")
    should_continue = False
    sys.exit(0)

@CrewBase
class ModeratorCrew():
    """Matrix Chat Room Moderator Crew"""
    
    def __init__(self):
        """Initialize the moderator crew with agents and tasks."""
        # Find the absolute paths for config files
        module_dir = pathlib.Path(__file__).parent.absolute()
        self.agents_config_path = os.path.join(module_dir, "config", "agents.yaml")
        self.tasks_config_path = os.path.join(module_dir, "config", "tasks.yaml")
        
        # Load configs
        with open(self.agents_config_path, 'r') as f:
            self.agents_config = yaml.safe_load(f)
            
        with open(self.tasks_config_path, 'r') as f:
            self.tasks_config = yaml.safe_load(f)
        
        # Initialize the LLM with Gemini
        self.llm = LLM(
            model="gemini/gemini-1.5-flash",
            api_key=os.getenv("GEMINI_API_KEY")
        )
        
        # Initialize the Matrix tool - we'll only login once during initialization
        self.matrix_tool = MatrixTool()
        self.login_success = False
        
        # Keep track of violations - dictionary with user IDs as keys
        self.violations = {}
        
        # Set signal handler for Ctrl+C
        signal.signal(signal.SIGINT, signal_handler)
        atexit.register(self.cleanup_handler)
        
        # Flag for input monitoring
        self.input_monitored = False
        
        # Pre-login to Matrix - we do this only once
        print("Initializing Matrix tool and logging in...")
        try:
            self.matrix_tool._login()
            self.login_success = True
            print("Matrix tool initialized successfully!")
        except Exception as e:
            print(f"Warning: Matrix login initialization failed: {e}")
            print("Will attempt login again during task execution.")
    
    def cleanup_handler(self):
        """Cleanup handler for atexit"""
        global should_continue
        should_continue = False
        print("\nExiting moderator. Cleanup complete.")
    
    @before_kickoff
    def ensure_login(self, inputs):
        """Ensure we're logged in to Matrix before running any tasks"""
        # We don't need to log in again if we already did in __init__
        if self.login_success:
            return inputs
            
        # If login failed in __init__, try again
        max_attempts = 3
        attempt = 0
        
        while not self.login_success and attempt < max_attempts:
            attempt += 1
            print(f"Ensuring Matrix login before starting tasks (attempt {attempt}/{max_attempts})...")
            try:
                session_id = self.matrix_tool._login()
                if session_id:
                    self.login_success = True
                    print("Matrix login successful!")
                    break
            except Exception as e:
                print(f"Warning: Matrix login failed: {e}")
                time.sleep(2)  # Wait before retrying

        if not self.login_success:
            print("Warning: Could not log in to Matrix after multiple attempts.")
            print("Tasks may fail if they require Matrix access.")
        
        return inputs
    
    @after_kickoff
    def cleanup(self, result):
        """Cleanup after crew execution"""
        print("Crew tasks completed.")
        return result
    
    def check_message_for_pg14(self, message, sender_id):
        """
        Check if a message violates PG-14 standards using the LLM
        """
        try:
            prompt = f"""
            Your task is to determine if the following message violates PG-14 standards.
            
            Message: "{message}"
            Sender: {sender_id}
            
            PG-14 standards prohibit:
            - Explicit sexual content or strong sexual innuendo
            - Graphic violence or gore
            - Hate speech, discrimination, or harassment
            - Promotion of self-harm, suicide, or harmful activities
            - Detailed instructions for illegal activities
            - Strong profanity
            
            Think carefully about context and nuance. Consider whether the message contains:
            - Slurs or derogatory terms
            - Threatening language
            - Explicit descriptions of violence
            - Sexual content inappropriate for teenagers
            
            Respond with ONLY "YES" if it violates PG-14 standards, or "NO" if it doesn't.
            """
            
            # Use the correct method for CrewAI's LLM
            response = self.llm.call(prompt).strip().upper()
            
            if "YES" in response:
                return True, "LLM determined this violates PG-14 standards"
            return False, "Message is compliant with PG-14 standards"
            
        except Exception as e:
            print(f"Error using LLM for content moderation: {e}")
            # Since we shouldn't use explicit fallbacks, return an error message
            # but try one more time with a simpler prompt
            try:
                simple_prompt = f'Does this message "{message}" violate PG-14 standards? Answer YES or NO only.'
                response = self.llm.call(simple_prompt).strip().upper()
                if "YES" in response:
                    return True, "Message likely violates PG-14 standards"
                return False, "Message appears compliant with PG-14 standards"
            except:
                # Last resort - just log that we couldn't determine
                print("Critical error: Unable to perform content moderation!")
                return True, "Could not analyze content - flagging for human review"
    
    # Define the moderator agent
    @agent
    def moderator_agent(self) -> Agent:
        return Agent(
            config=self.agents_config['moderator_agent'],
            verbose=True,
            tools=[
                MatrixTool(task="join_room", session_id=self.matrix_tool.session_id),
                MatrixTool(task="watch_room", session_id=self.matrix_tool.session_id),
                MatrixTool(task="redact_message", session_id=self.matrix_tool.session_id),
                MatrixTool(task="ban_user", session_id=self.matrix_tool.session_id),
                MatrixTool(task="send_message", session_id=self.matrix_tool.session_id)
            ],
            llm=self.llm
        )
    
    @task
    def join_room_task(self) -> Task:
        return Task(
            config=self.tasks_config['join_room_task'],
            description_prefix="Join the target moderation room using the Matrix Tool. "
        )
    
    @task
    def monitor_room_task(self) -> Task:
        return Task(
            config=self.tasks_config['monitor_room_task'],
            description_prefix="Monitor the room for inappropriate messages that violate PG-14 standards. ",
            context=None  # Remove the context dependency as we'll use sequential process
        )
    
    @crew
    def crew(self) -> Crew:
        """Creates the Matrix Moderator crew"""
        return Crew(
            agents=self.agents,  # Use the automatically created agents
            tasks=self.tasks,    # Automatically created by the @task decorator
            process=Process.sequential,  # Use sequential process to ensure tasks run in order
            verbose=True,
            llm=self.llm,
        )
    
    def start_input_monitor(self):
        """Start monitoring for keyboard input in a separate thread"""
        if self.input_monitored:
            return
            
        def input_monitor():
            global should_continue
            while should_continue:
                try:
                    # This blocks until input is received
                    user_input = input()
                    if user_input.lower() == 'q':
                        print("\nReceived 'q'. Stopping moderation...")
                        should_continue = False
                        break
                except EOFError:
                    time.sleep(0.5)  # Sleep briefly before checking again
                except Exception as e:
                    print(f"Error in input monitor: {e}")
                    time.sleep(1)
            
        # Start the input monitor thread
        self.input_thread = threading.Thread(target=input_monitor, daemon=True)
        self.input_thread.start()
        self.input_monitored = True
    
    def continuous_monitor(self, room_id):
        """Continuously monitor the room for new messages until user presses 'q'"""
        global should_continue
        should_continue = True
        room_id = "!iYYuXGoKsPtMPlJEub:mozilla.org"  # Use the specified room ID
        
        # Ensure we're logged in
        if not self.login_success:
            try:
                self.matrix_tool._login()
                self.login_success = True
            except Exception as e:
                print(f"Error logging in: {e}")
                return
        
        print(f"\n==== Starting continuous moderation of room {room_id} ====")
        print("Press 'q' and Enter at any time to stop moderation")
        
        # Start monitoring for 'q' input
        self.start_input_monitor()
        
        # Initialize seen messages to avoid processing the same message repeatedly
        seen_messages = set()
        next_batch_token = None
        
        # Initially join the room to ensure we're a member
        join_result = self.matrix_tool.join_room(room_id)
        print(f"Join room result: {join_result}")
        
        last_activity_time = time.time()
        last_check_time = time.time()
        check_interval = 5  # Check every 5 seconds by default
        consecutive_empty_checks = 0
        
        while should_continue:
            try:
                current_time = time.time()
                time_since_last_check = current_time - last_check_time
                
                # Adjust check frequency based on activity
                if consecutive_empty_checks > 5:
                    # If we've had multiple empty checks, slow down to reduce load
                    check_interval = min(15, check_interval + 1)  # Max 15 seconds
                else:
                    # More active periods, check more frequently
                    check_interval = max(3, check_interval - 1)  # Min 3 seconds
                
                # Skip this iteration if it's not time to check yet
                if time_since_last_check < check_interval:
                    time.sleep(0.5)  # Short sleep to prevent CPU spinning
                    continue
                
                # Update last check time
                last_check_time = current_time
                
                # Show heartbeat message periodically
                if current_time - last_activity_time > 300:
                    # Show a heartbeat message every 5 minutes of inactivity
                    print(f"[{time.strftime('%H:%M:%S')}] Still monitoring room for new messages... (checking every {check_interval}s)")
                    last_activity_time = current_time
                
                # Try to watch for new messages
                print(f"[{time.strftime('%H:%M:%S')}] Checking for new messages...")
                watch_result = self.matrix_tool.watch_room(room_id, next_batch_token, 1)
                
                # Update the next_batch token for the next call
                next_batch_token = self.matrix_tool.next_batch
                
                # Check if we got an actual message
                if "New message in room" in watch_result:
                    print(f"\n[{time.strftime('%H:%M:%S')}] New message detected!")
                    print(watch_result)
                    last_activity_time = time.time()
                    consecutive_empty_checks = 0  # Reset counter
                    
                    # Extract information from the message
                    sender = None
                    content = None
                    event_id = None
                    
                    for line in watch_result.split('\n'):
                        if line.startswith("Sender:"):
                            sender = line.replace("Sender:", "").strip()
                        elif line.startswith("Content:"):
                            content = line.replace("Content:", "").strip()
                        elif line.startswith("Event ID:"):
                            event_id = line.replace("Event ID:", "").strip()
                    
                    if content and event_id:
                        # Check if we've already seen this message
                        message_key = f"{sender}:{event_id}"
                        if message_key in seen_messages:
                            print(f"Already processed message {event_id}, skipping...")
                            continue
                            
                        # Add to seen messages
                        seen_messages.add(message_key)
                        
                        # Check if the message violates PG-14 standards
                        violates, reason = self.check_message_for_pg14(content, sender)
                        
                        if violates:
                            print(f"⚠️ Message from {sender} violates PG-14 standards! Reason: {reason}")
                            
                            # Track violations
                            if sender not in self.violations:
                                self.violations[sender] = {
                                    'count': 0,
                                    'last_time': 0,
                                    'messages': []
                                }
                            
                            # Check if an hour has passed since last violation
                            current_time = time.time()
                            if current_time - self.violations[sender]['last_time'] > 3600:
                                # Reset count if more than an hour has passed
                                print(f"Resetting violation count for {sender} (over 1 hour since last violation)")
                                self.violations[sender]['count'] = 0
                            
                            # Increment violation count
                            self.violations[sender]['count'] += 1
                            self.violations[sender]['last_time'] = current_time
                            self.violations[sender]['messages'].append((event_id, content))
                            
                            # Redact the message
                            redact_result = self.matrix_tool.redact_message(
                                room_id, 
                                event_id, 
                                f"Violates PG-14 standards: {reason}"
                            )
                            print(f"🚫 Redaction result: {redact_result}")
                            
                            # Check if we should send a warning or ban
                            if self.violations[sender]['count'] == 2:
                                warning_msg = f"⚠️ @{sender} This is your second violation. One more illicit message within 1 hour will result in a ban."
                                send_result = self.matrix_tool.send_message_task(room_id, warning_msg)
                                print(f"⚠️ Warning sent to {sender}: {send_result}")
                            elif self.violations[sender]['count'] >= 3:
                                ban_reason = "Three violations of PG-14 standards within one hour"
                                print(f"🔨 Banning user {sender} for {ban_reason}")
                                ban_result = self.matrix_tool.ban_user(
                                    room_id,
                                    sender,
                                    ban_reason
                                )
                                print(f"🔨 Ban result: {ban_result}")
                        else:
                            print(f"✅ Message from {sender} is PG-14 compliant: {reason}")
                else:
                    # No new messages
                    consecutive_empty_checks += 1
                
            except Exception as e:
                print(f"Error during monitoring: {e}")
                time.sleep(5)  # Longer delay after an error
        
        print("Monitoring stopped.")

def main():
    """Run the moderator crew."""
    print("Starting Matrix Chat Room Moderator Crew...")
    moderator = ModeratorCrew()
    
    # First join the room using the crew
    crew_instance = moderator.crew()
    result = crew_instance.kickoff()
    print("Initial crew tasks completed.")
    
    # Now start continuous monitoring
    try:
        moderator.continuous_monitor("!iYYuXGoKsPtMPlJEub:mozilla.org")
    except KeyboardInterrupt:
        print("\nModeration stopped by user.")
    
    print("Moderation completed.")

if __name__ == "__main__":
    main() 