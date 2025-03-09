// templates/update.rs - Templates for UPDATE statements
// Defines Tera templates for Redis commands generated from UPDATE statements

use tera::Tera;

/// Registers all UPDATE statement templates with Tera
pub fn register_update_templates(tera: &mut Tera) -> Result<(), tera::Error> {
    // --------------------------------
    // String Command Templates
    // --------------------------------
    // String update uses SET command (same as string insert)
    tera.add_raw_template("string_update", "SET {{ key }} {{ value }}")?;
    
    // --------------------------------
    // Hash Command Templates
    // --------------------------------
    // Hash update uses HSET command (same as hash insert)
    tera.add_raw_template("hash_update", "HSET {{ key }} {{ field_values }}")?;
    
    // --------------------------------
    // List Command Templates
    // --------------------------------
    tera.add_raw_template("list_update", "LSET {{ key }} {{ index }} {{ value }}")?;
    
    // --------------------------------
    // Sorted Set Command Templates
    // --------------------------------
    // Sorted set update score uses ZADD command (same as zset insert)
    tera.add_raw_template("zset_update", "ZADD {{ key }} {{ score }} {{ member }}")?;
    
    Ok(())
}