use crate::helpers::TestApp;

#[tokio::test]
async fn test_database_cleanup() {
    let mut app = TestApp::new().await;
    
    // Verify the test database was created with a unique name
    assert!(!app.db_name.is_empty());
    assert!(!app.cleanup_called);
    
    // Make a simple request to ensure the app is working
    let response = app.get_root().await;
    assert_eq!(response.status().as_u16(), 200);
    
    // Call cleanup explicitly
    app.cleanup().await;
    assert!(app.cleanup_called);
    
    // Calling cleanup again should be safe (idempotent)
    app.cleanup().await;
    assert!(app.cleanup_called);
}

#[tokio::test] 
async fn test_multiple_test_apps_have_unique_databases() {
    let app1 = TestApp::new().await;
    let app2 = TestApp::new().await;
    
    // Each test app should have a unique database name
    assert_ne!(app1.db_name, app2.db_name);
    assert!(!app1.cleanup_called);
    assert!(!app2.cleanup_called);
    
    // Both should work independently
    let response1 = app1.get_root().await;
    let response2 = app2.get_root().await;
    
    assert_eq!(response1.status().as_u16(), 200);
    assert_eq!(response2.status().as_u16(), 200);
}