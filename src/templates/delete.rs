// templates/delete.rs - Templates for DELETE statements
// Defines Tera templates for Redis commands generated from DELETE statements

use tera::Tera;

/// Registers all DELETE statement templates with Tera
pub fn register_delete_templates(tera: &mut Tera) -> Result<(), tera::Error> {
    // --------------------------------
    // Common DEL template (used by all Redis data types for key deletion)
    // --------------------------------
    tera.add_raw_template("del", "DEL {{ key }}")?;
    
    // --------------------------------
    // Hash Command Templates
    // --------------------------------
    tera.add_raw_template("hash_delete", "DEL {{ key }}")?;
    tera.add_raw_template("hash_delete_field", "HDEL {{ key }} {{ field }}")?;
    
    // --------------------------------
    // List Command Templates
    // --------------------------------
    tera.add_raw_template("list_delete", "DEL {{ key }}")?;
    tera.add_raw_template("list_delete_value", "LREM {{ key }} 0 {{ value }}")?;
    
    // --------------------------------
    // Set Command Templates
    // --------------------------------
    tera.add_raw_template("set_delete", "DEL {{ key }}")?;
    tera.add_raw_template("set_delete_member", "SREM {{ key }} {{ member }}")?;
    
    // --------------------------------
    // Sorted Set Command Templates
    // --------------------------------
    tera.add_raw_template("zset_delete", "DEL {{ key }}")?;
    tera.add_raw_template("zset_delete_member", "ZREM {{ key }} {{ member }}")?;
    
    Ok(())
}