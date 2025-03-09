// tests/bench_tests.rs
//! Benchmark tests for SQL to Redis transformation
//!
//! Run with: cargo test --test bench_tests -- --ignored

use std::time::Instant;
mod test_utils;

const ITERATIONS: usize = 1000;

#[test]
#[ignore] // Only run these benchmarks when explicitly requested
fn bench_string_operations() {
    let transformer = test_utils::create_transformer();
    let sql = "SELECT * FROM users WHERE key = 'user:1001'";
    
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _result = transformer.transform(sql).unwrap();
    }
    let duration = start.elapsed();
    
    println!("String GET - {} iterations: {:?} ({:?} per operation)", 
        ITERATIONS, duration, duration / ITERATIONS as u32);
}

#[test]
#[ignore] // Only run these benchmarks when explicitly requested
fn bench_hash_operations() {
    let transformer = test_utils::create_transformer();
    let sql = "SELECT * FROM users__hash WHERE key = 'user:1001'";
    
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _result = transformer.transform(sql).unwrap();
    }
    let duration = start.elapsed();
    
    println!("Hash HGETALL - {} iterations: {:?} ({:?} per operation)", 
        ITERATIONS, duration, duration / ITERATIONS as u32);
}

#[test]
#[ignore] // Only run these benchmarks when explicitly requested
fn bench_complex_operations() {
    let transformer = test_utils::create_transformer();
    let sql = "SELECT * FROM leaderboard__zset WHERE key = 'game:global' AND score > 1000 ORDER BY score DESC LIMIT 10";
    
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _result = transformer.transform(sql).unwrap();
    }
    let duration = start.elapsed();
    
    println!("Complex ZSet operation - {} iterations: {:?} ({:?} per operation)", 
        ITERATIONS, duration, duration / ITERATIONS as u32);
}

#[test]
#[ignore] // Only run these benchmarks when explicitly requested
fn bench_all_operations() {
    let transformer = test_utils::create_transformer();
    
    let test_cases = [
        "SELECT * FROM users WHERE key = 'user:1001'",
        "SELECT * FROM users__hash WHERE key = 'user:1001'",
        "SELECT name FROM users__hash WHERE key = 'user:1001'",
        "SELECT * FROM posts__list WHERE key = 'user:1001:posts'",
        "SELECT * FROM followers__set WHERE key = 'user:1001:followers'",
        "SELECT * FROM leaderboard__zset WHERE key = 'game:global'",
        "INSERT INTO users (key, value) VALUES ('user:1001', 'John Doe')",
        "UPDATE users SET value = 'Jane Doe' WHERE key = 'user:1001'",
        "DELETE FROM users WHERE key = 'user:1001'",
    ];
    
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        for sql in &test_cases {
            let _result = transformer.transform(sql).unwrap();
        }
    }
    let duration = start.elapsed();
    
    println!("All operations ({} queries) - {} iterations: {:?} ({:?} per full set)", 
        test_cases.len(), ITERATIONS, duration, duration / ITERATIONS as u32);
}