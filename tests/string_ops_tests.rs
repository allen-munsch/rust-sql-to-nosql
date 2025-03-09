// tests/string_ops_tests.rs
use sql_redis::SqlToRedisTransformer;

#[test]
fn test_string_operations() {
    let transformer = SqlToRedisTransformer::new().unwrap();
    
    // Test string GET
    let result = transformer.transform("SELECT * FROM users WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "GET user:1001");
    
    // Test string GET value
    let result = transformer.transform("SELECT value FROM users WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "GET user:1001");
    
    // Test string SET
    let result = transformer.transform("INSERT INTO users (key, value) VALUES ('user:1001', 'John Doe')").unwrap();
    assert_eq!(result, "SET user:1001 John Doe");
    
    // Test string update
    let result = transformer.transform("UPDATE users SET value = 'Jane Doe' WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "SET user:1001 Jane Doe");
    
    // Test string delete
    let result = transformer.transform("DELETE FROM users WHERE key = 'user:1001'").unwrap();
    assert_eq!(result, "DEL user:1001");
}

