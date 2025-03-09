use clap::{arg, command, Parser, Subcommand};
use sql_redis::SqlToRedisTransformer;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sqlnosql")]
#[command(about = "Transform SQL queries to Redis commands", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// SQL query to transform
    #[arg(short, long)]
    query: Option<String>,

    /// Input file containing SQL queries (one per line)
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// List all supported patterns
    #[arg(long)]
    list_patterns: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Transform a SQL query to Redis command
    Transform {
        /// SQL query to transform
        query: String,
    },
    /// List all supported patterns
    ListPatterns,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let transformer = SqlToRedisTransformer::new()?;

    // Handle --list-patterns flag
    if cli.list_patterns {
        print_patterns(&transformer);
        return Ok(());
    }

    // Handle subcommands
    if let Some(command) = cli.command {
        match command {
            Commands::Transform { query } => {
                transform_query(&transformer, &query)?;
            }
            Commands::ListPatterns => {
                print_patterns(&transformer);
            }
        }
        return Ok(());
    }

    // Handle --query argument
    if let Some(query) = cli.query {
        transform_query(&transformer, &query)?;
        return Ok(());
    }

    // Handle --file argument
    if let Some(file_path) = cli.file {
        let content = fs::read_to_string(file_path)?;
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with("--") {
                println!("SQL: {}", line);
                match transform_query(&transformer, line) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Error: {}", e),
                }
                println!();
            }
        }
        return Ok(());
    }

    // Check if stdin is available for reading
    let mut buffer = String::new();
    
    // Read from stdin - this will handle both direct piping and interactive input
    if let Ok(bytes_read) = io::stdin().read_to_string(&mut buffer) {
        if bytes_read > 0 {
            // Split the input by lines and process each line as a separate query
            for line in buffer.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with("--") {
                    println!("SQL: {}", line);
                    match transform_query(&transformer, line) {
                        Ok(_) => (),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                    println!();
                }
            }
            return Ok(());
        }
    }

    // No input was provided
    println!("No input provided. Use --help for usage information.");
    Ok(())
}

fn print_patterns(transformer: &SqlToRedisTransformer) {
    println!("Supported SQL to Redis patterns:");
    
    // Get detailed pattern information
    let details = transformer.get_pattern_details();
    
    // Group by pattern type/category
    let mut select_patterns = Vec::new();
    let mut insert_patterns = Vec::new();
    let mut update_patterns = Vec::new();
    let mut delete_patterns = Vec::new();
    
    for (i, pattern) in details.iter().enumerate() {
        let entry = format!("  {}. {} (matcher: {})\n     SQL: {}\n     Redis: {}", 
            i + 1, 
            pattern.name, 
            pattern.matcher,
            pattern.sql_pattern,
            pattern.redis_pattern);
        
        // Categorize based on name or matcher
        if pattern.name.contains("get") || pattern.matcher.contains("get") {
            select_patterns.push(entry);
        } else if pattern.name.contains("set") || pattern.matcher.contains("set") ||
                  pattern.name.contains("push") || pattern.matcher.contains("push") ||
                  pattern.name.contains("add") || pattern.matcher.contains("add") {
            insert_patterns.push(entry);
        } else if pattern.name.contains("update") || pattern.matcher.contains("update") {
            update_patterns.push(entry);
        } else if pattern.name.contains("delete") || pattern.matcher.contains("delete") ||
                  pattern.name.contains("del") || pattern.matcher.contains("del") {
            delete_patterns.push(entry);
        } else {
            // Default to select if we can't categorize
            select_patterns.push(entry);
        }
    }
    
    // Print each category
    if !select_patterns.is_empty() {
        println!("\nSELECT Operations:");
        for pattern in select_patterns {
            println!("{}", pattern);
        }
    }
    
    if !insert_patterns.is_empty() {
        println!("\nINSERT Operations:");
        for pattern in insert_patterns {
            println!("{}", pattern);
        }
    }
    
    if !update_patterns.is_empty() {
        println!("\nUPDATE Operations:");
        for pattern in update_patterns {
            println!("{}", pattern);
        }
    }
    
    if !delete_patterns.is_empty() {
        println!("\nDELETE Operations:");
        for pattern in delete_patterns {
            println!("{}", pattern);
        }
    }
}

fn transform_query(transformer: &SqlToRedisTransformer, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    match transformer.transform(query) {
        Ok(command) => {
            println!("Redis: {}", command);
            Ok(())
        }
        Err(e) => {
            Err(format!("Transformation failed: {}", e).into())
        }
    }
}