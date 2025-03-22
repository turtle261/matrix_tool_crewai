//! Main test file for the Matrix API
//!
//! This file runs all the tests for the Matrix API and provides detailed output.

use super::api_tests;
use super::integration_tests;

/// Run all tests for the Matrix API
#[actix_web::test]
async fn run_all_matrix_api_tests() {
    println!("==========================================");
    println!("Running all Matrix API tests...");
    println!("==========================================");
    
    // Run the API tests
    println!("\n## Running API endpoint tests...");
    api_tests::run_all_tests().await;
    
    // Run the integration tests
    println!("\n## Running integration tests...");
    integration_tests::run_all_integration_tests().await;
    
    println!("\n==========================================");
    println!("All Matrix API tests completed!");
    println!("==========================================");
}