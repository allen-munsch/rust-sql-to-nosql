// templates/mod.rs - Template engine for Redis command generation
// Loads and renders Redis command templates based on SQL patterns

use tera::{Context, Tera};
use crate::context::TemplateContext;
use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum TemplateError {
    IoError(io::Error),
    TeraError(tera::Error),
    InvalidPath(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::IoError(e) => write!(f, "IO error: {}", e),
            TemplateError::TeraError(e) => write!(f, "Template error: {}", e),
            TemplateError::InvalidPath(p) => write!(f, "Invalid path: {}", p),
        }
    }
}

impl Error for TemplateError {}

impl From<io::Error> for TemplateError {
    fn from(error: io::Error) -> Self {
        TemplateError::IoError(error)
    }
}

impl From<tera::Error> for TemplateError {
    fn from(error: tera::Error) -> Self {
        TemplateError::TeraError(error)
    }
}

pub struct TemplateEngine {
    tera: Tera,
    template_dir: PathBuf,
}

impl TemplateEngine {
    pub fn new() -> Result<Self, TemplateError> {
        let mut tera = Tera::default();
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let template_dir = Path::new(crate_dir).join("src").join("templates");
        
        // Register basic templates
        Self::register_command_templates(&mut tera)?;
        
        let mut engine = Self { tera, template_dir };
        
        // Register Lua templates
        engine.register_lua_templates()?;
        
        Ok(engine)
    }
    
    /// Register basic command templates as raw strings for simple Redis commands
    fn register_command_templates(tera: &mut Tera) -> Result<(), TemplateError> {
        // Common templates
        tera.add_raw_template("del", "DEL {{ key }}")?;
        
        // String operations
        tera.add_raw_template("string_get", "GET {{ key }}")?;
        tera.add_raw_template("string_set", "SET {{ key }} {{ value }}")?;
        
        // Hash operations
        tera.add_raw_template("hash_getall", "HGETALL {{ key }}")?;
        tera.add_raw_template("hash_get", "HGET {{ key }} {{ field }}")?;
        tera.add_raw_template("hash_hmget", "HMGET {{ key }} {{ fields }}")?;
        tera.add_raw_template("hash_set", "HSET {{ key }} {{ field_values }}")?;
        tera.add_raw_template("hash_delete_field", "HDEL {{ key }} {{ field }}")?;
        
        // List operations
        tera.add_raw_template("list_getall", "LRANGE {{ key }} 0 -1")?;
        tera.add_raw_template("list_get_index", "LINDEX {{ key }} {{ index }}")?;
        tera.add_raw_template("list_get_range", "LRANGE {{ key }} {{ start }} {{ stop }}")?;
        tera.add_raw_template("list_push", "RPUSH {{ key }} {{ value }}")?;
        tera.add_raw_template("list_update", "LSET {{ key }} {{ index }} {{ value }}")?;
        tera.add_raw_template("list_delete_value", "LREM {{ key }} 0 {{ value }}")?;
        
        // Set operations
        tera.add_raw_template("set_getall", "SMEMBERS {{ key }}")?;
        tera.add_raw_template("set_ismember", "SISMEMBER {{ key }} {{ member }}")?;
        tera.add_raw_template("set_add", "SADD {{ key }} {{ members }}")?;
        tera.add_raw_template("set_delete_member", "SREM {{ key }} {{ member }}")?;
        
        // Sorted Set operations
        tera.add_raw_template("zset_getall", "ZRANGEBYSCORE {{ key }} -inf +inf")?;
        tera.add_raw_template("zset_get_score_range", "ZRANGEBYSCORE {{ key }} {{ min }} {{ max }}")?;
        tera.add_raw_template("zset_get_reversed", "ZREVRANGEBYSCORE {{ key }} {{ max }} {{ min }}")?;
        tera.add_raw_template("zset_add", "ZADD {{ key }} {{ score }} {{ member }}")?;
        tera.add_raw_template("zset_delete_member", "ZREM {{ key }} {{ member }}")?;
        
        Ok(())
    }
    
    /// Dynamically registers all Lua templates from the 'templates/lua' directory
    fn register_lua_templates(&mut self) -> Result<(), TemplateError> {
        let lua_dir = self.template_dir.join("lua");
        self.visit_dirs(&lua_dir, "")?;
        Ok(())
    }
    
    /// Recursively visit directories to find and register Lua templates
    fn visit_dirs(&mut self, dir: &Path, prefix: &str) -> Result<(), TemplateError> {
        if !dir.is_dir() {
            return Err(TemplateError::InvalidPath(format!("{:?} is not a directory", dir)));
        }
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Create a new prefix by adding the directory name
                let dir_name = path.file_name()
                    .ok_or_else(|| TemplateError::InvalidPath(format!("Invalid directory name: {:?}", path)))?
                    .to_string_lossy();
                
                let new_prefix = if prefix.is_empty() {
                    dir_name.to_string()
                } else {
                    format!("{}_{}", prefix, dir_name)
                };
                
                self.visit_dirs(&path, &new_prefix)?;
            } else if let Some(ext) = path.extension() {
                if ext == "lua" {
                    // Get the file name without extension
                    let file_stem = path.file_stem()
                        .ok_or_else(|| TemplateError::InvalidPath(format!("Invalid file name: {:?}", path)))?
                        .to_string_lossy();
                    
                    // Create template name in the format: prefix_filename_lua
                    let template_name = if prefix.is_empty() {
                        format!("{}_lua", file_stem)
                    } else {
                        format!("{}_{}_lua", prefix, file_stem)
                    };
                    
                    // Read file content
                    let content = fs::read_to_string(&path)?;
                    
                    // Register template
                    self.tera.add_raw_template(&template_name, &content)
                        .map_err(TemplateError::from)?;
                    
                    // For debug
                    // println!("Registered Lua template: {} from {:?}", template_name, path);
                }
            }
        }
        
        Ok(())
    }
    
    /// Render a template with the given context
    pub fn render(&self, template_name: &str, context: &TemplateContext) -> Result<String, tera::Error> {
        let mut tera_context = Context::new();
        for (key, value) in context {
            tera_context.insert(key, value);
        }
        self.tera.render(template_name, &tera_context)
    }
    
    /// Render a Lua template with the given context
    pub fn render_lua(&self, category: &str, operation: &str, context: &TemplateContext) -> Result<String, tera::Error> {
        let template_name = format!("{}_{}_{}", category, operation, "lua");
        self.render(&template_name, context)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use tera::{Context, Tera};

    #[test]
    fn test_load_templates() {
        // Create a new Tera instance
        let mut tera = Tera::default();
        
        // Add basic templates
        let result = register_basic_templates(&mut tera);
        assert!(result.is_ok(), "Failed to register basic templates: {:?}", result.err());
        
        // Get template dir path
        let template_dir = get_template_directory();
        assert!(template_dir.exists(), "Template directory does not exist: {:?}", template_dir);
        
        // Load Lua templates
        let lua_dir = template_dir.join("lua");
        assert!(lua_dir.exists(), "Lua directory does not exist: {:?}", lua_dir);
        
        let result = load_lua_templates(&mut tera, &lua_dir, "");
        assert!(result.is_ok(), "Failed to load Lua templates: {:?}", result.err());
        
        // Check that some expected templates are loaded
        let expected_templates = vec![
            "string_get", 
            "hash_getall", 
            "list_getall",
            "string_get_lua",
            "hash_hgetall_lua",
            "complex_join_inner_join_lua"
        ];
        
        for template_name in expected_templates {
            assert!(tera.get_template_names().any(|n| n == template_name), 
                    "Expected template '{}' was not loaded", template_name);
        }
        
        // Test rendering a simple template
        let mut context = Context::new();
        context.insert("key", "user:1001");
        
        let result = tera.render("string_get", &context);
        assert!(result.is_ok(), "Failed to render template: {:?}", result.err());
        assert_eq!(result.unwrap(), "GET user:1001");
        
        // Test rendering a complex template
        let result = tera.render("string_get_lua", &context);
        assert!(result.is_ok(), "Failed to render Lua template: {:?}", result.err());
        assert!(result.unwrap().contains("redis.call('GET', key)"));
    }
    
    fn register_basic_templates(tera: &mut Tera) -> Result<(), tera::Error> {
        // Register a few basic templates for testing
        tera.add_raw_template("string_get", "GET {{ key }}")?;
        tera.add_raw_template("hash_getall", "HGETALL {{ key }}")?;
        tera.add_raw_template("list_getall", "LRANGE {{ key }} 0 -1")?;
        
        Ok(())
    }
    
    fn get_template_directory() -> PathBuf {
        // In a test, the current directory might be the project root
        // We need to find the template directory relative to that
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        manifest_dir.join("src").join("templates")
    }
    
    fn load_lua_templates(tera: &mut Tera, dir: &Path, prefix: &str) -> Result<(), tera::Error> {
        // Skip if directory doesn't exist
        if !dir.is_dir() {
            return Ok(());
        }
        
        for entry in fs::read_dir(dir).map_err(|e| tera::Error::msg(format!("Failed to read directory: {}", e)))? {
            let entry = entry.map_err(|e| tera::Error::msg(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Create a new prefix by adding the directory name
                let dir_name = path.file_name()
                    .ok_or_else(|| tera::Error::msg(format!("Invalid directory name: {:?}", path)))?
                    .to_string_lossy();
                
                let new_prefix = if prefix.is_empty() {
                    dir_name.to_string()
                } else {
                    format!("{}_{}", prefix, dir_name)
                };
                
                load_lua_templates(tera, &path, &new_prefix)?;
            } else if let Some(ext) = path.extension() {
                if ext == "lua" {
                    // Get the file name without extension
                    let file_stem = path.file_stem()
                        .ok_or_else(|| tera::Error::msg(format!("Invalid file name: {:?}", path)))?
                        .to_string_lossy();
                    
                    // Create template name in the format: prefix_filename_lua
                    let template_name = if prefix.is_empty() {
                        format!("{}_lua", file_stem)
                    } else {
                        format!("{}_{}_lua", prefix, file_stem)
                    };
                    
                    // Read file content
                    let content = fs::read_to_string(&path)
                        .map_err(|e| tera::Error::msg(format!("Failed to read file: {}", e)))?;
                    
                    // Register template
                    tera.add_raw_template(&template_name, &content)?;
                    
                    // For debugging
                    // println!("Registered Lua template: {} from {:?}", template_name, path);
                }
            }
        }
        
        Ok(())
    }
}