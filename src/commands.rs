// commands.rs - Redis command generation
use sqlparser::ast::Statement;
use crate::pattern::extractors;

/// A Redis command with its arguments
#[derive(Debug, Clone)]
pub struct RedisCommand {
   pub command: String,
   pub args: Vec<String>,
}

impl RedisCommand {
   /// Create a new Redis command
   pub fn new<S: Into<String>, A: Into<String>>(command: S, args: Vec<A>) -> Self {
       Self {
           command: command.into(),
           args: args.into_iter().map(|a| a.into()).collect(),
       }
   }

   /// Format the command as a string
   pub fn to_string(&self) -> String {
       let mut parts = Vec::with_capacity(self.args.len() + 1);
       parts.push(self.command.clone());
       parts.extend(self.args.clone());
       parts.join(" ")
   }
}

/// Generate a Redis command from a SQL statement
pub fn generate_command(stmt: &Statement) -> Option<RedisCommand> {
   // Try to match against all patterns, in order
   
   // String operations
   if let Some(info) = extractors::extract_string_get(stmt) {
       return Some(RedisCommand::new("GET", vec![info.key]));
   }
   
   // Hash operations
   if let Some(info) = extractors::extract_hash_getall(stmt) {
       return Some(RedisCommand::new("HGETALL", vec![info.key]));
   }
   
   if let Some(info) = extractors::extract_hash_get(stmt) {
       return Some(RedisCommand::new("HGET", vec![info.key, info.field]));
   }
   
   if let Some(info) = extractors::extract_hash_multi_get(stmt) {
       let mut args = Vec::with_capacity(info.fields.len() + 1);
       args.push(info.key);
       args.extend(info.fields);
       return Some(RedisCommand::new("HMGET", args));
   }
   
   // List operations
   if let Some(info) = extractors::extract_list_get_index(stmt) {
       return Some(RedisCommand::new("LINDEX", vec![info.key, info.index]));
   }
   
   if let Some(info) = extractors::extract_list_get_range(stmt) {
       return Some(RedisCommand::new("LRANGE", vec![
           info.key,
           "0".to_string(),
           (info.limit - 1).to_string()
       ]));
   }
   
   if let Some(info) = extractors::extract_list_getall(stmt) {
       // TODO
       return Some(RedisCommand::new("LRANGE", vec![info.key, "0".to_string(), "-1".to_string()]));
   }
   
   // Set operations
   if let Some(info) = extractors::extract_set_ismember(stmt) {
       return Some(RedisCommand::new("SISMEMBER", vec![info.key, info.member]));
   }
   
   if let Some(info) = extractors::extract_set_getall(stmt) {
       return Some(RedisCommand::new("SMEMBERS", vec![info.key]));
   }
   
   // Sorted Set operations
   if let Some(info) = extractors::extract_zset_get_reversed(stmt) {
       return Some(RedisCommand::new("ZREVRANGEBYSCORE", vec![info.key, "+inf".to_string(), "-inf".to_string()]));
   }
   
   if let Some(info) = extractors::extract_zset_get_score_range(stmt) {
       return Some(RedisCommand::new("ZRANGEBYSCORE", vec![
           info.key,
           info.min,
           info.max
       ]));
   }
   
   if let Some(info) = extractors::extract_zset_getall(stmt) {
       return Some(RedisCommand::new("ZRANGEBYSCORE", vec![info.key, "-inf".to_string(), "+inf".to_string()]));
   }
   
   // Insert operations
   if let Some(info) = extractors::extract_insert_command(stmt) {
       match determine_table_type(&info.table) {
           "string" => {
               if let Some(value) = info.fields.get("value") {
                   return Some(RedisCommand::new("SET", vec![info.key, value.clone()]));
               }
           },
           "hash" => {
               // HMSET
               let mut args = vec![info.key];
               for (field, value) in info.fields.iter() {
                   if field != "key" {
                       args.push(field.clone());
                       args.push(value.clone());
                   }
               }
               return Some(RedisCommand::new("HMSET", args));
           },
           "list" => {
               if let Some(value) = info.fields.get("value") {
                   if let Some(index) = info.fields.get("index") {
                       // LSET
                       return Some(RedisCommand::new("LSET", vec![info.key, index.clone(), value.clone()]));
                   } else {
                       // RPUSH
                       return Some(RedisCommand::new("RPUSH", vec![info.key, value.clone()]));
                   }
               }
           },
           "set" => {
               if let Some(member) = info.fields.get("member") {
                   // SADD
                   return Some(RedisCommand::new("SADD", vec![info.key, member.clone()]));
               }
           },
           "zset" => {
               if let (Some(member), Some(score)) = (info.fields.get("member"), info.fields.get("score")) {
                   // ZADD
                   return Some(RedisCommand::new("ZADD", vec![info.key, score.clone(), member.clone()]));
               }
           },
           _ => {}
       }
   }
   
   // Delete operations
   if let Some(info) = extractors::extract_delete_command(stmt) {
       match determine_table_type(&info.table) {
           "string" | "hash" | "list" | "set" | "zset" => {
               if info.member.is_none() && info.index.is_none() {
                   // DEL
                   return Some(RedisCommand::new("DEL", vec![info.key]));
               }
           },
           _ => {}
       }

       // Type-specific member deletion
       match determine_table_type(&info.table) {
           "hash" => {
               if let Some(field) = info.member {
                   return Some(RedisCommand::new("HDEL", vec![info.key, field]));
               }
           },
           "list" => {
               if let Some(value) = info.member {
                   return Some(RedisCommand::new("LREM", vec![info.key, "0".to_string(), value]));
               }
           },
           "set" => {
               if let Some(member) = info.member {
                   return Some(RedisCommand::new("SREM", vec![info.key, member]));
               }
           },
           "zset" => {
               if let Some(member) = info.member {
                   return Some(RedisCommand::new("ZREM", vec![info.key, member]));
               }
           },
           _ => {}
       }
   }
   
   // No match found
   None
}

// Helper to determine Redis data type from table name
fn determine_table_type(table: &str) -> &'static str {
   if table.ends_with("__hash") {
       "hash"
   } else if table.ends_with("__list") {
       "list"
   } else if table.ends_with("__set") {
       "set"
   } else if table.ends_with("__zset") {
       "zset"
   } else {
       "string"
   }
}