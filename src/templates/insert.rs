// templates/insert.rs - Templates for INSERT statements
// Defines Tera templates for Redis commands generated from INSERT statements

use tera::Tera;

/// Registers all INSERT statement templates with Tera
pub fn register_insert_templates(tera: &mut Tera) -> Result<(), tera::Error> {
    // --------------------------------
    // String Command Templates
    // --------------------------------
    tera.add_raw_template("string_set", "SET {{ key }} {{ value }}")?;
    
    // --------------------------------
    // Hash Command Templates
    // --------------------------------
    tera.add_raw_template("hash_set", "HSET {{ key }} {{ field_values }}")?;
    
    // --------------------------------
    // List Command Templates
    // --------------------------------
    tera.add_raw_template("list_push", "RPUSH {{ key }} {{ value }}")?;
    
    // --------------------------------
    // Set Command Templates
    // --------------------------------
    tera.add_raw_template("set_add", "SADD {{ key }} {{ members }}")?;
    
    // --------------------------------
    // Sorted Set Command Templates
    // --------------------------------
    tera.add_raw_template("zset_add", "ZADD {{ key }} {{ score }} {{ member }}")?;
    
    Ok(())
}