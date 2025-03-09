// tests/set_ops_tests.rs
use sql_redis::SqlToRedisTransformer;

#[test]
fn test_set_operations() {
    let transformer = SqlToRedisTransformer::new().unwrap();
    
    // Test set SMEMBERS
    let result = transformer.transform("SELECT * FROM followers__set WHERE key = 'user:1001:followers'").unwrap();
    assert_eq!(result, "SMEMBERS user:1001:followers");
    
    // Test set SISMEMBER
    let result = transformer.transform("SELECT * FROM followers__set WHERE key = 'user:1001:followers' AND member = 'user:1002'").unwrap();
    assert_eq!(result, "SISMEMBER user:1001:followers user:1002");
    
    // Test set SADD
    let result = transformer.transform("INSERT INTO followers__set (key, member) VALUES ('user:1001:followers', 'user:1002')").unwrap();
    assert_eq!(result, "SADD user:1001:followers user:1002");
    
    // Test set delete
    let result = transformer.transform("DELETE FROM followers__set WHERE key = 'user:1001:followers'").unwrap();
    assert_eq!(result, "DEL user:1001:followers");
    
    // Test set member delete
    let result = transformer.transform("DELETE FROM followers__set WHERE key = 'user:1001:followers' AND member = 'user:1002'").unwrap();
    assert_eq!(result, "SREM user:1001:followers user:1002");
}