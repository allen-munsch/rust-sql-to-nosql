// templates/select.rs - Templates for SELECT statements
// Defines Tera templates for Redis commands generated from SELECT statements

use tera::Tera;

/// Registers all SELECT statement templates with Tera
pub fn register_select_templates(tera: &mut Tera) -> Result<(), tera::Error> {
    // --------------------------------
    // String Command Templates
    // --------------------------------
    tera.add_raw_template("string_get", "GET {{ key }}")?;
    
    // --------------------------------
    // Hash Command Templates
    // --------------------------------
    tera.add_raw_template("hash_getall", "HGETALL {{ key }}")?;
    tera.add_raw_template("hash_get", "HGET {{ key }} {{ field }}")?;
    tera.add_raw_template("hash_hmget", "HMGET {{ key }} {{ fields }}")?;
    
    // --------------------------------
    // List Command Templates
    // --------------------------------
    tera.add_raw_template("list_getall", "LRANGE {{ key }} {{ start }} {{ stop }}")?;
    tera.add_raw_template("list_get_index", "LINDEX {{ key }} {{ index }}")?;
    
    // --------------------------------
    // Set Command Templates
    // --------------------------------
    tera.add_raw_template("set_getall", "SMEMBERS {{ key }}")?;
    tera.add_raw_template("set_ismember", "SISMEMBER {{ key }} {{ member }}")?;
    
    // --------------------------------
    // Sorted Set Command Templates
    // --------------------------------
    tera.add_raw_template("zset_getall", "ZRANGEBYSCORE {{ key }} {{ min }} {{ max }}")?;
    tera.add_raw_template("zset_get_score_range", "ZRANGEBYSCORE {{ key }} {{ min }} {{ max }}")?;
    tera.add_raw_template("zset_get_reversed", "ZREVRANGEBYSCORE {{ key }} {{ max }} {{ min }}")?;
    
    Ok(())
}