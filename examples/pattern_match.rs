// examples/pattern_match.rs
use sql_redis::pattern::cte::extract_ctes;
use sql_redis::pattern::matchers::common::{has_limit, key_equals, order_by_score_desc, score_range, string_table, table_with_suffix};
use sql_redis::pattern::subquery::extract_all_subqueries;
use sql_redis::pattern::combinators::Pattern;
use sql_redis::pattern::join::JoinInfo;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use sqlparser::ast::{Statement, SetExpr, Query};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dialect = GenericDialect {};
    
    // Define example SQL queries
    let examples = vec![
        // String operations
        "select * from users;",
        "SELECT * FROM users WHERE key = 'user:*'",
        "SELECT value FROM users WHERE key = 'user:1001'",
        "SELECT * FROM users WHERE key IN ('user:1001', 'user:1002')",
        
        // Hash field operations
        "SELECT name, email FROM users__hash WHERE key = 'user:1001'",
        "SELECT * FROM users__hash WHERE key = 'user:1001'",
        "SELECT * FROM users__hash WHERE key IN ('user:1001', 'user:1002')",
        "SELECT name FROM users__hash WHERE key = 'user:1001' AND name LIKE 'J%'",
        
        // List operations
        "SELECT * FROM messages__list WHERE key = 'user:1001:messages' LIMIT 10",
        "SELECT * FROM messages__list WHERE key = 'user:1001:messages'",
        "SELECT * FROM messages__list WHERE key = 'user:1001:messages' AND index = 0",
        "SELECT * FROM messages__list WHERE key = 'user:1001:messages' AND index BETWEEN 0 AND 5",
        "SELECT * FROM messages__list WHERE key = 'user:1001:messages' ORDER BY index DESC LIMIT 3",
        
        // Set operations 
        "SELECT * FROM followers__set WHERE key = 'user:1001:followers'",
        "SELECT * FROM followers__set WHERE key = 'user:1001:followers' AND member = 'user:1002'",
        "SELECT COUNT(*) FROM followers__set WHERE key = 'user:1001:followers'",
        "SELECT * FROM followers__set WHERE key IN ('user:1001:followers', 'user:1002:followers')",
        
        // Sorted set operations
        "SELECT * FROM leaderboard__zset WHERE key = 'games:global'",
        "SELECT * FROM leaderboard__zset WHERE key = 'games:global' AND score > 1000",
        "SELECT * FROM leaderboard__zset WHERE key = 'games:global' AND score < 1000",
        "SELECT * FROM leaderboard__zset WHERE key = 'games:global' AND score BETWEEN 1000 AND 2000",
        "SELECT * FROM leaderboard__zset WHERE key = 'games:global' ORDER BY score DESC",
        "SELECT * FROM leaderboard__zset WHERE key = 'games:global' ORDER BY score DESC LIMIT 10",
        "SELECT * FROM leaderboard__zset WHERE key = 'games:global' AND score > 1000 ORDER BY score DESC",
        "SELECT member, score FROM leaderboard__zset WHERE key = 'games:global' AND member = 'user:1001'",
        
        // Complex queries
        "SELECT * FROM users__hash WHERE key = 'user:1001' AND (age > 30 OR name LIKE 'J%')",
        "SELECT * FROM leaderboard__zset WHERE key = 'games:global' AND score > 1000 AND member LIKE 'user:10%'",
        
        // CTE examples
        "WITH active_users AS (SELECT * FROM users WHERE last_login > '2023-01-01')
            SELECT * FROM active_users WHERE score > 100",
            
        "WITH active_users AS (SELECT * FROM users WHERE last_login > '2023-01-01'),
            high_score_users AS (SELECT * FROM leaderboard__zset WHERE score > 1000)
            SELECT * FROM active_users JOIN high_score_users ON active_users.id = high_score_users.user_id",
            
        // JOIN examples
        "SELECT u.name, p.title 
            FROM users u
            JOIN posts__hash p ON u.id = p.user_id
            WHERE u.key = 'user:1001'",
            
        "SELECT u.name, f.member
            FROM users__hash u
            JOIN followers__set f ON u.id = f.user_id
            WHERE f.key = 'followers:global'",
            
        // INSERT examples
        "INSERT INTO users (key, value) VALUES ('user:1003', 'Alice Smith')",
        "INSERT INTO users__hash (key, name, email, age) VALUES ('user:1003', 'Alice Smith', 'alice@example.com', '28')",
        "INSERT INTO messages__list (key, value) VALUES ('user:1003:messages', 'Hello world')",
        "INSERT INTO followers__set (key, member) VALUES ('user:1003:followers', 'user:1001')",
        "INSERT INTO leaderboard__zset (key, member, score) VALUES ('games:global', 'user:1003', '1200')",
        
        // DELETE examples
        "DELETE FROM users WHERE key = 'user:1003'",
        "DELETE FROM users__hash WHERE key = 'user:1003'",
        "DELETE FROM messages__list WHERE key = 'user:1003:messages' AND index = 0",
        "DELETE FROM followers__set WHERE key = 'user:1003:followers' AND member = 'user:1001'",
        "DELETE FROM leaderboard__zset WHERE key = 'games:global' AND member = 'user:1003'"
    ];
    
    println!("SQL Pattern Matching Examples\n");
    
    for sql in examples {
        println!("SQL: {}", sql);
        
        // Parse the SQL
        let statements = Parser::parse_sql(&dialect, sql)?;
        if let Some(stmt) = statements.first() {
            // Match different patterns
            println!("  Patterns matched:");
            
            if let Statement::Query(query) = stmt {
                // Patterns that operate on Query
                match_query_patterns(query);
                
                // Handle CTE pattern
                if let Some(with) = &query.with {
                    if let Ok(ctes) = extract_ctes().match_pattern(with) {
                        println!("    - Contains CTEs: {}", ctes.iter()
                            .map(|cte| cte.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", "));
                    }
                }
                
                // Extract expressions from WHERE clause for pattern matching
                if let SetExpr::Select(select) = &*query.body {
                    if let Some(expr) = &select.selection {
                        // Try to match key pattern
                        if let Ok(key) = key_equals().match_pattern(expr) {
                            println!("    - Key equals: {}", key);
                        }
                        
                        // Try to match score range pattern
                        if let Ok((min, max)) = score_range().match_pattern(expr) {
                            println!("    - Score range: {} to {}", min, max);
                        }
                    }
                    
                    // Extract table types
                    let tables = extract_table_types_from_select(select);
                    if !tables.is_empty() {
                        for (table_type, table_name) in tables {
                            println!("    - Table type: {} ({})", table_type, table_name);
                        }
                    }
                }
            }
            
            // Try to match subqueries
            let subqueries = extract_all_subqueries(stmt);
            if !subqueries.is_empty() {
                println!("    - Contains {} subqueries", subqueries.len());
            }
            
            // Try to match JOIN patterns
            if let Some(joins) = extract_joins(stmt) {
                println!("    - Contains {} JOIN operations", joins.len());
                for join in joins {
                    println!("      {:?} JOIN between {} and {}", 
                        join.join_type,
                        join.left.name,
                        join.right.name);
                }
            }
        }
        
        println!();
    }
    
    Ok(())
}

// Helper function to match patterns on Query object
fn match_query_patterns(query: &Query) {
    // Try to match limit pattern
    if let Ok(limit) = has_limit().match_pattern(query) {
        println!("    - Has LIMIT: {}", limit);
    }
    
    // Try to match order by pattern
    if let Ok(_) = order_by_score_desc().match_pattern(query) {
        println!("    - Order by score DESC");
    }
}

// Helper function to extract table types from select statement
fn extract_table_types_from_select(select: &sqlparser::ast::Select) -> Vec<(String, String)> {
    let mut results = Vec::new();
    
    for table_with_joins in &select.from {
        // Apply the patterns on the table factor
        let table_factor = &table_with_joins.relation;
        
        if let Ok(table) = string_table().match_pattern(table_factor) {
            results.push(("String".to_string(), table));
        }
        
        if let Ok(table) = table_with_suffix("__hash").match_pattern(table_factor) {
            results.push(("Hash".to_string(), table));
        }
        
        if let Ok(table) = table_with_suffix("__list").match_pattern(table_factor) {
            results.push(("List".to_string(), table));
        }
        
        if let Ok(table) = table_with_suffix("__set").match_pattern(table_factor) {
            results.push(("Set".to_string(), table));
        }
        
        if let Ok(table) = table_with_suffix("__zset").match_pattern(table_factor) {
            results.push(("Sorted Set".to_string(), table));
        }
    }
    
    results
}

// Helper function to extract joins
fn extract_joins(_stmt: &sqlparser::ast::Statement) -> Option<Vec<JoinInfo>> {
    // Implementation would depend on available join extraction functions
    // This is a placeholder
    None
}