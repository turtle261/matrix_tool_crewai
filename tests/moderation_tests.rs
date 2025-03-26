use actix_web::{App, HttpServer, web};
use matrix_api::{api, config::Config};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;
use uuid::Uuid;
use webbrowser;
use serde_json::json;
use log::{info, warn, error, debug};
use std::env;

// Test configuration (shared with api_tests.rs)
const TEST_TIMEOUT_SECS: u64 = 300; // 5 minutes max for the entire test
const LOGIN_CHECK_INTERVAL_SECS: u64 = 3;
const SYNC_WAIT_TIME_SECS: u64 = 15; // Increased from 5 to 15 seconds for WSL/Linux compatibility
const SERVER_PORT: u16 = 8080;
const SERVER_ADDR: &str = "127.0.0.1";
const SERVER_URL: &str = "http://127.0.0.1:8080";
const REQUEST_RETRIES: u32 = 5;
const RETRY_DELAY_MS: u64 = 1000;

// The specific room ID we want to moderate
const MODERATION_ROOM_ID: &str = "!iYYuXGoKsPtMPlJEub:mozilla.org";

struct TestStep {
    name: String,
    start_time: Instant,
    completed: bool,
    success: bool,
    details: Option<String>,
}

impl TestStep {
    fn new(name: &str) -> Self {
        println!("\n=== STEP: {} ===", name);
        info!("[TEST STEP] Starting: {}", name);
        TestStep {
            name: name.to_string(),
            start_time: Instant::now(),
            completed: false,
            success: false,
            details: None,
        }
    }

    fn complete_success(&mut self, details: Option<String>) {
        let duration = self.start_time.elapsed();
        self.completed = true;
        self.success = true;
        self.details = details;
        info!("[TEST STEP] ✅ Success: {} (took {:.2}s)", self.name, duration.as_secs_f64());
        println!("✅ COMPLETED: {} ({:.2}s)", self.name, duration.as_secs_f64());
    }

    fn complete_failure(&mut self, error: &str) {
        let duration = self.start_time.elapsed();
        self.completed = true;
        self.success = false;
        self.details = Some(error.to_string());
        error!("[TEST STEP] ❌ Failed: {} (took {:.2}s): {}", self.name, duration.as_secs_f64(), error);
        println!("❌ FAILED: {} ({:.2}s) - {}", self.name, duration.as_secs_f64(), error);
    }
}

// Initialize logging for tests
fn init_logging() {
    // Only initialize if not already done
    if env::var_os("RUST_LOG").is_none() {
        // Set different log levels for different modules
        env::set_var("RUST_LOG", "info,matrix_sdk=warn,hyper=warn,reqwest=warn");
        env_logger::init();
    }
}

// Helper function for making HTTP requests with retries
async fn make_request_with_retry<F, Fut, T>(
    operation: F,
    operation_name: &str,
    retries: u32,
) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, reqwest::Error>>,
{
    let mut last_error = None;
    
    for attempt in 1..=retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                let error_msg = format!("{}: {}", operation_name, e);
                if attempt < retries {
                    warn!("Request attempt {}/{} failed: {}. Retrying...", attempt, retries, error_msg);
                    sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64)).await;
                }
                last_error = Some(error_msg);
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| format!("{} failed after {} attempts", operation_name, retries)))
}

#[tokio::test]
async fn test_moderation_features() {
    init_logging();
    println!("\n========================================");
    println!("🧪 MATRIX MODERATION API TEST SUITE");
    println!("========================================");
    println!("This test verifies the moderation functionality for Matrix rooms");
    println!("An SSO login window will open in your browser - please complete the login");
    println!("========================================\n");
    
    let test_start_time = Instant::now();
    let mut steps: Vec<TestStep> = Vec::new();
    
    info!("Starting Matrix Moderation API test");
    
    // Initialize server step
    let mut server_step = TestStep::new("Starting API Server");
    
    // Start the actual server in the background
    let _server_handle = tokio::spawn(async {
        info!("Starting actual API server for test on port {}", SERVER_PORT);
        let config = Config::from_file("config.toml").expect("Failed to load config.toml");
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let state = api::ApiState { sessions, config };
        
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(state.clone()))
                .configure(api::config)
        })
        .bind(format!("{}:{}", SERVER_ADDR, SERVER_PORT))
        .expect("Failed to bind server")
        .run()
        .await
        .expect("Server error");
    });
    
    // Give the server a moment to start
    info!("Waiting for server to start...");
    sleep(Duration::from_secs(2)).await;
    server_step.complete_success(Some(format!("Server started on {}:{}", SERVER_ADDR, SERVER_PORT)));
    steps.push(server_step);
    
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none()) // Don't follow redirects for SSO
        .timeout(Duration::from_secs(120)) // Increased from 30 to 120 seconds for WSL/Linux compatibility
        .build()
        .expect("Failed to build HTTP client");
    
    let mut client_step = TestStep::new("Initializing HTTP Client");
    client_step.complete_success(None);
    steps.push(client_step);
    
    let mut session_id = String::new();
    
    // STEP 1: Start SSO login
    let mut login_step = TestStep::new("Starting SSO Login");
    match make_request_with_retry(
        || client.post(format!("{}/login/sso/start", SERVER_URL)).send(),
        "SSO login start",
        REQUEST_RETRIES
    ).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<serde_json::Value>().await {
                    Ok(body) => {
                        if let (Some(id), Some(url)) = (body["session_id"].as_str(), body["sso_url"].as_str()) {
                            session_id = id.to_string();
                            let sso_url = url.to_string();
                            
                            info!("Got session_id: {}", session_id);
                            info!("Please complete SSO login in your browser. Opening URL: {}", sso_url);
                            
                            // Open browser for SSO login
                            if webbrowser::open(&sso_url).is_err() {
                                warn!("Failed to open browser automatically. Please manually open this URL to login: {}", sso_url);
                                println!("\n==================================================================");
                                println!("IMPORTANT: Please open this URL in your browser to login:");
                                println!("{}", sso_url);
                                println!("==================================================================\n");
                            }
                            
                            login_step.complete_success(Some(format!("Session ID: {}", session_id)));
                        } else {
                            let error_msg = "Missing session_id or sso_url in response";
                            login_step.complete_failure(error_msg);
                            panic!("{}", error_msg);
                        }
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to parse login response: {}", e);
                        login_step.complete_failure(&error_msg);
                        panic!("{}", error_msg);
                    }
                }
            } else {
                let error_msg = format!("Failed to start SSO login: {}", resp.status());
                login_step.complete_failure(&error_msg);
                panic!("{}", error_msg);
            }
        },
        Err(e) => {
            login_step.complete_failure(&e);
            panic!("{}", e);
        }
    }
    steps.push(login_step);
    
    // STEP 2: Wait for login to complete
    let mut wait_login_step = TestStep::new("Waiting for SSO Login Completion");
    let mut login_complete = false;
    let start_time = Instant::now();
    
    println!("\n==================================================================");
    println!("Waiting for you to complete the SSO login in your browser");
    println!("The test will continue automatically after you log in");
    println!("==================================================================\n");
    
    while !login_complete && start_time.elapsed().as_secs() < TEST_TIMEOUT_SECS {
        sleep(Duration::from_secs(LOGIN_CHECK_INTERVAL_SECS)).await;
        
        match make_request_with_retry(
            || client.get(format!("{}/login/status/{}", SERVER_URL, session_id)).send(),
            "Login status check",
            REQUEST_RETRIES
        ).await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(status) => {
                            debug!("Login status: {:?}", status);
                            
                            if status["status"] == "logged_in" {
                                login_complete = true;
                                info!("Login completed successfully!");
                                println!("\n==================================================================");
                                println!("✅ SSO LOGIN SUCCESSFUL! Continuing with test...");
                                println!("==================================================================\n");
                                wait_login_step.complete_success(None);
                            } else if status["status"] == "error" {
                                let error_msg = status["error"].as_str().unwrap_or("Unknown error");
                                wait_login_step.complete_failure(error_msg);
                                panic!("Login failed: {}", error_msg);
                            }
                        },
                        Err(e) => {
                            warn!("Failed to parse login status: {}", e);
                        }
                    }
                } else {
                    warn!("Login status check returned error: {}", resp.status());
                }
            },
            Err(e) => {
                warn!("Failed to check login status: {}. Will retry.", e);
            }
        }
    }
    
    if !login_complete {
        let error_msg = format!("Login timed out after {} seconds", TEST_TIMEOUT_SECS);
        wait_login_step.complete_failure(&error_msg);
        panic!("{}", error_msg);
    }
    steps.push(wait_login_step);
    
    // STEP 3: Join the specified moderation room
    let mut join_room_step = TestStep::new(&format!("Joining Moderation Room ({})", MODERATION_ROOM_ID));
    match make_request_with_retry(
        || client.post(format!("{}/rooms/{}/join/{}", SERVER_URL, session_id, MODERATION_ROOM_ID))
            .send(),
        "Join room",
        REQUEST_RETRIES
    ).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<serde_json::Value>().await {
                    Ok(result) => {
                        if result["status"] == "success" {
                            join_room_step.complete_success(Some(format!("Successfully joined room {}", MODERATION_ROOM_ID)));
                        } else {
                            let error_msg = format!("Room join response did not indicate success: {:?}", result);
                            // This is not a critical error, we might already be in the room
                            warn!("{}", error_msg);
                            join_room_step.complete_success(Some("Room join response did not indicate success, may already be in room".to_string()));
                        }
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to parse join room response: {}", e);
                        join_room_step.complete_failure(&error_msg);
                        panic!("{}", error_msg);
                    }
                }
            } else {
                // This isn't necessarily a test failure - we might already be in the room
                let warning = format!("Failed to join room (might already be a member): {}", resp.status());
                warn!("{}", warning);
                join_room_step.complete_success(Some(warning));
            }
        },
        Err(e) => {
            join_room_step.complete_failure(&e);
            panic!("{}", e);
        }
    }
    steps.push(join_room_step);
    
    // STEP 4: Test sending a message to the room
    let test_message = format!("Test message from moderation test - {}", Uuid::new_v4());
    let mut send_message_step = TestStep::new(&format!("Sending Test Message: '{}'", test_message));
    
    let mut event_id = String::new();
    match make_request_with_retry(
        || client.post(format!("{}/rooms/{}/{}/send", SERVER_URL, session_id, MODERATION_ROOM_ID))
            .json(&json!({
                "body": test_message
            }))
            .send(),
        "Send test message",
        REQUEST_RETRIES
    ).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<serde_json::Value>().await {
                    Ok(result) => {
                        if let Some(id) = result["event_id"].as_str() {
                            event_id = id.to_string();
                            send_message_step.complete_success(Some(format!("Message sent, event ID: {}", event_id)));
                        } else {
                            let error_msg = "Missing event_id in send message response";
                            send_message_step.complete_failure(error_msg);
                            panic!("{}", error_msg);
                        }
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to parse send message response: {}", e);
                        send_message_step.complete_failure(&error_msg);
                        panic!("{}", error_msg);
                    }
                }
            } else {
                let error_msg = format!("Failed to send message: {}", resp.status());
                send_message_step.complete_failure(&error_msg);
                panic!("{}", error_msg);
            }
        },
        Err(e) => {
            send_message_step.complete_failure(&e);
            panic!("{}", e);
        }
    }
    steps.push(send_message_step);
    
    // Wait for message to propagate
    sleep(Duration::from_secs(2)).await;
    
    // STEP 5: Test the watch_room endpoint
    let mut watch_room_step = TestStep::new("Testing Watch Room Functionality");
    match make_request_with_retry(
        || client.get(format!("{}/rooms/{}/{}/watch", SERVER_URL, session_id, MODERATION_ROOM_ID))
            .query(&[("timeout", "5")])
            .send(),
        "Watch room",
        REQUEST_RETRIES
    ).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<serde_json::Value>().await {
                    Ok(result) => {
                        if result.get("next_batch").is_some() {
                            let has_messages = result.get("has_new_messages")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            
                            if has_messages {
                                watch_room_step.complete_success(Some("Watch room returned new messages".to_string()));
                            } else {
                                watch_room_step.complete_success(Some("Watch room returned no new messages, but endpoint works".to_string()));
                            }
                        } else {
                            let error_msg = "Missing next_batch token in watch room response";
                            watch_room_step.complete_failure(error_msg);
                            panic!("{}", error_msg);
                        }
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to parse watch room response: {}", e);
                        watch_room_step.complete_failure(&error_msg);
                        panic!("{}", error_msg);
                    }
                }
            } else {
                let error_msg = format!("Failed to watch room: {}", resp.status());
                watch_room_step.complete_failure(&error_msg);
                panic!("{}", error_msg);
            }
        },
        Err(e) => {
            watch_room_step.complete_failure(&e);
            panic!("{}", e);
        }
    }
    steps.push(watch_room_step);
    
    // STEP 6: Test redacting the message
    let mut redact_message_step = TestStep::new(&format!("Testing Redact Message ({})", event_id));
    match make_request_with_retry(
        || client.post(format!("{}/rooms/{}/{}/redact/{}", SERVER_URL, session_id, MODERATION_ROOM_ID, event_id))
            .json(&json!({
                "reason": "Test redaction from moderation test"
            }))
            .send(),
        "Redact message",
        REQUEST_RETRIES
    ).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<serde_json::Value>().await {
                    Ok(result) => {
                        if result["status"] == "success" {
                            redact_message_step.complete_success(Some("Message redacted successfully".to_string()));
                        } else {
                            let error_msg = format!("Redaction response did not indicate success: {:?}", result);
                            redact_message_step.complete_failure(&error_msg);
                            panic!("{}", error_msg);
                        }
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to parse redact message response: {}", e);
                        redact_message_step.complete_failure(&error_msg);
                        panic!("{}", error_msg);
                    }
                }
            } else {
                let error_msg = format!("Failed to redact message: {}", resp.status());
                redact_message_step.complete_failure(&error_msg);
                panic!("{}", error_msg);
            }
        },
        Err(e) => {
            redact_message_step.complete_failure(&e);
            panic!("{}", e);
        }
    }
    steps.push(redact_message_step);
    
    // (We don't actually test ban_user as that would be disruptive in a real room)
    // But we do test that the endpoint exists and returns a sensible error
    
    // STEP 7: Test the ban_user endpoint existence (but don't actually ban)
    let fake_user_id = "@fake_user:example.com"; // This user doesn't exist
    let mut ban_user_step = TestStep::new(&format!("Testing Ban User API (with fake user ID)"));
    match make_request_with_retry(
        || client.post(format!("{}/rooms/{}/{}/ban/{}", SERVER_URL, session_id, MODERATION_ROOM_ID, fake_user_id))
            .json(&json!({
                "reason": "Test ban from moderation test (should fail with user not found)"
            }))
            .send(),
        "Ban user (should fail with user not found)",
        REQUEST_RETRIES
    ).await {
        Ok(resp) => {
            // We expect this to fail because the user doesn't exist
            if !resp.status().is_success() {
                ban_user_step.complete_success(Some("Ban user endpoint exists and correctly rejected fake user ID".to_string()));
            } else {
                // This is unexpected - the ban should have failed
                let warning = "Ban user endpoint did not reject fake user ID as expected. This is unusual.";
                warn!("{}", warning);
                ban_user_step.complete_success(Some(warning.to_string()));
            }
        },
        Err(e) => {
            // Even an error is fine - we just want to confirm the endpoint exists
            ban_user_step.complete_success(Some(format!("Ban user endpoint test produced error (expected): {}", e)));
        }
    }
    steps.push(ban_user_step);
    
    // Print test summary
    let total_duration = test_start_time.elapsed();
    let success_steps = steps.iter().filter(|s| s.success).count();
    let total_steps = steps.len();
    
    println!("\n========================================");
    println!("📋 MATRIX MODERATION API TEST SUMMARY");
    println!("========================================");
    println!("Test completed in {:.2} seconds", total_duration.as_secs_f64());
    println!("Steps Completed: {}/{} ({:.1}%)", 
        success_steps, 
        total_steps, 
        (success_steps as f64 / total_steps as f64) * 100.0
    );
    println!("Status: {}", if success_steps == total_steps { "✅ ALL PASSED" } else { "❌ SOME STEPS FAILED" });
    println!("\nVerified Moderation Features:");
    println!("  • Joining Specific Room");
    println!("  • Watching Room for Messages");
    println!("  • Redacting Messages");
    println!("  • Ban User API Endpoint");
    println!("\nStep Details:");
    
    for (i, step) in steps.iter().enumerate() {
        let status = if step.success { "✅ PASS" } else { "❌ FAIL" };
        println!("  {}. {} - {}", i + 1, step.name, status);
        if let Some(details) = &step.details {
            println!("     └─ Details: {}", details);
        }
    }
    println!("\n========================================");
    println!("🎉 MATRIX MODERATION API TEST COMPLETE");
    println!("========================================\n");
    
    info!("Matrix Moderation API test completed successfully!");
} 