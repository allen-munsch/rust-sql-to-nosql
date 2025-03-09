// examples/simple.rs - Example of SQL to Redis transformer
use sql_redis::SqlToRedisTransformer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a transformer - the constructor doesn't return a Result
    // let transformer = SqlToRedisTransformer::new();
    
    // Define a list of SQL queries based on our BNF grammar
    let examples = vec![
        // ========== SELECT Operations ==========
        // String operations
        "SELECT * FROM users WHERE key = 'user:1001'",
        "SELECT value FROM users WHERE key = 'user:1001'",
        
        // Hash operations
        "SELECT * FROM users__hash WHERE key = 'user:1001'",
        "SELECT name FROM users__hash WHERE key = 'user:1001'",
        "SELECT name, email, age FROM users__hash WHERE key = 'user:1001'",
        "SELECT name FROM users__hash WHERE key IN ('user:1001', 'user:1002')",
        
        // List operations
        "SELECT * FROM tweets__list WHERE key = 'user:1001:tweets'",
        "SELECT * FROM tweets__list WHERE key = 'user:1001:tweets' AND index = 0",
        "SELECT * FROM tweets__list WHERE key = 'user:1001:tweets' LIMIT 10",
        "SELECT * FROM tweets__list WHERE key = 'user:1001:tweets' AND index BETWEEN 0 AND 5",
        
        // Set operations
        "SELECT * FROM followers__set WHERE key = 'user:1001:followers'",
        "SELECT * FROM followers__set WHERE key = 'user:1001:followers' AND member = 'user:1002'",
        "SELECT * FROM followers__set WHERE key IN ('user:1001:followers', 'user:1002:followers')",
        
        // Sorted Set operations
        "SELECT * FROM leaderboard__zset WHERE key = 'games:leaderboard'",
        "SELECT * FROM leaderboard__zset WHERE key = 'games:leaderboard' AND score > 1000",
        "SELECT * FROM leaderboard__zset WHERE key = 'games:leaderboard' ORDER BY score DESC",
        "SELECT * FROM leaderboard__zset WHERE key = 'games:leaderboard' AND score BETWEEN 1000 AND 2000",
        "SELECT * FROM leaderboard__zset WHERE key = 'games:leaderboard' LIMIT 10",
        
        // ========== INSERT Operations ==========
        // String operations
        "INSERT INTO users (key, value) VALUES ('user:1001', 'John Doe')",
        "INSERT INTO users (key, value) VALUES ('user:1002', 'Jane Smith')",
        
        // Hash operations
        "INSERT INTO users__hash (key, name, email, age) VALUES ('user:1001', 'John Doe', 'john@example.com', '30')",
        "INSERT INTO users__hash (key, name, email) VALUES ('user:1002', 'Jane Smith', 'jane@example.com')",
        
        // List operations
        "INSERT INTO tweets__list (key, value) VALUES ('user:1001:tweets', 'Hello, Redis!')",
        "INSERT INTO tweets__list (key, value) VALUES ('user:1001:tweets', 'Another tweet')",
        "INSERT INTO tweets__list (key, index, value) VALUES ('user:1001:tweets', 0, 'First tweet')",
        
        // Set operations
        "INSERT INTO followers__set (key, member) VALUES ('user:1001:followers', 'user:1002')",
        "INSERT INTO followers__set (key, member) VALUES ('user:1001:followers', 'user:1003')",
        
        // Sorted Set operations
        "INSERT INTO leaderboard__zset (key, member, score) VALUES ('games:leaderboard', 'user:1001', '1500')",
        "INSERT INTO leaderboard__zset (key, member, score) VALUES ('games:leaderboard', 'user:1002', '2000')",
        
        // ========== DELETE Operations ==========
        "DELETE FROM users WHERE key = 'user:1001'",
        "DELETE FROM users__hash WHERE key = 'user:1001'",
        "DELETE FROM tweets__list WHERE key = 'user:1001:tweets'",
        "DELETE FROM followers__set WHERE key = 'user:1001:followers' AND member = 'user:1002'",
        "DELETE FROM leaderboard__zset WHERE key = 'games:leaderboard' AND member = 'user:1001'",
    ];
    
    // Transform each SQL query to Redis command
    println!("SQL to Redis Command Transformer\n");
    println!("{:<70} | {}", "SQL Query", "Redis Command");
    println!("{:-<70}-+-{:-<30}", "", "");
    
    for sql in examples {
        let transformer = SqlToRedisTransformer::new()?;
        match transformer.transform(sql) {
            Ok(command) => {
                println!("{:<70} | {}", sql, command);
            },
            Err(e) => {
                println!("{:<70} | Error: {}", sql, e);
            },
        }
    }    
    Ok(())
}