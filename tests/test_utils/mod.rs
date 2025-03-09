// tests/test_utils/mod.rs
use sql_redis::SqlToRedisTransformer;

/// Test a SQL statement against an expected Redis command
pub fn assert_sql_to_redis(sql: &str, expected_redis: &str) {
    let transformer = SqlToRedisTransformer::new().unwrap();
    let result = transformer.transform(sql);
    
    // Print detailed error message if the transformation fails
    if let Err(err) = &result {
        println!("Failed to transform SQL: {}", sql);
        println!("Error: {}", err);
    }
    
    assert!(result.is_ok(), "Failed to transform SQL: {}", sql);
    assert_eq!(result.unwrap(), expected_redis, "SQL: {}", sql);
}

/// Test multiple SQL statements against expected Redis commands
pub fn assert_multiple_transformations(test_cases: &[(&str, &str)]) {
    for (sql, expected) in test_cases {
        assert_sql_to_redis(sql, expected);
    }
}

/// Create a transformer for benchmarking
pub fn create_transformer() -> SqlToRedisTransformer {
    SqlToRedisTransformer::new().unwrap()
}