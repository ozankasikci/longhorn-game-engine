// crates/longhorn-scripting/src/compiler.rs
use crate::js_runtime::JsRuntimeError;
use std::collections::HashMap;
use std::path::Path;

/// Compiled script with metadata
#[derive(Debug, Clone)]
pub struct CompiledScript {
    /// Original TypeScript source path
    pub source_path: String,
    /// Compiled JavaScript code
    pub js_code: String,
    /// Class name extracted from "export default class Foo"
    pub class_name: String,
    /// Execution order (parsed from static executionOrder)
    pub execution_order: i32,
    /// Property definitions (name -> default value as JSON)
    pub properties: HashMap<String, String>,
}

/// TypeScript compiler using deno_core
pub struct TypeScriptCompiler {
    // No fields needed - we do simple string-based type stripping for MVP
}

impl TypeScriptCompiler {
    pub fn new() -> Self {
        Self {}
    }

    /// Compile TypeScript source to JavaScript
    pub fn compile(&mut self, source: &str, _filename: &str) -> Result<String, JsRuntimeError> {
        // For now, just pass through (deno_core handles TS natively in modules)
        // In a full implementation, we'd use swc or deno's TS compiler
        // For MVP, we'll require pre-compiled JS or use simple TS that's valid JS

        // Strip type annotations (very basic - production would use swc)
        let js = self.strip_types(source);
        Ok(js)
    }

    /// Very basic type stripping (MVP only - use swc in production)
    fn strip_types(&self, source: &str) -> String {
        let mut result = String::new();
        let mut chars = source.chars().peekable();

        while let Some(c) = chars.next() {
            // Skip type annotations after :
            if c == ':' {
                // Check if this looks like a type annotation (not object key)
                let mut type_annotation = String::new();
                let mut depth = 0;

                while let Some(&next) = chars.peek() {
                    if next == '{' || next == '<' || next == '(' {
                        depth += 1;
                        type_annotation.push(chars.next().unwrap());
                    } else if next == '}' || next == '>' || next == ')' {
                        if depth > 0 {
                            depth -= 1;
                            type_annotation.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    } else if (next == '=' || next == ',' || next == ';' || next == '\n') && depth == 0 {
                        break;
                    } else {
                        type_annotation.push(chars.next().unwrap());
                    }
                }

                // Keep the colon only if it's an object literal
                if type_annotation.trim().starts_with('{') ||
                   type_annotation.contains('\n') ||
                   type_annotation.trim().is_empty() {
                    result.push(':');
                    result.push_str(&type_annotation);
                }
                // Otherwise strip the type annotation
            } else {
                result.push(c);
            }
        }

        // Remove export/default keywords (for non-module script execution)
        result = result.replace("export default ", "");
        result = result.replace("export ", "");

        result
    }

    /// Load and compile a TypeScript file
    pub fn compile_file(&mut self, path: &Path) -> Result<CompiledScript, CompilerError> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| CompilerError::Io(e.to_string()))?;

        let js_code = self.compile(&source, path.to_str().unwrap_or("unknown"))
            .map_err(|e| CompilerError::Compilation(e.to_string()))?;

        // Parse execution order from source (look for static executionOrder = N)
        let execution_order = self.parse_execution_order(&source);

        // Parse property definitions
        let properties = self.parse_properties(&source);

        // Parse class name
        let class_name = self.parse_class_name(&source);

        Ok(CompiledScript {
            source_path: path.display().to_string(),
            js_code,
            class_name,
            execution_order,
            properties,
        })
    }

    fn parse_execution_order(&self, source: &str) -> i32 {
        // Look for: static executionOrder = N
        for line in source.lines() {
            if line.contains("static") && line.contains("executionOrder") {
                if let Some(eq_pos) = line.find('=') {
                    let after_eq = &line[eq_pos + 1..];
                    let num_str: String = after_eq
                        .chars()
                        .filter(|c| c.is_ascii_digit() || *c == '-')
                        .collect();
                    if let Ok(n) = num_str.parse() {
                        return n;
                    }
                }
            }
        }
        0 // default
    }

    fn parse_properties(&self, source: &str) -> HashMap<String, String> {
        let mut props = HashMap::new();

        // Look for class properties with defaults: name = value;
        // This is a simplified parser - production would use proper AST
        for line in source.lines() {
            let trimmed = line.trim();

            // Skip if it's a method or static
            if trimmed.starts_with("static") ||
               trimmed.contains("(") ||
               trimmed.starts_with("//") ||
               trimmed.starts_with("on") {
                continue;
            }

            // Look for: propName = value
            if let Some(eq_pos) = trimmed.find('=') {
                let name = trimmed[..eq_pos].trim();
                let value = trimmed[eq_pos + 1..].trim().trim_end_matches(';');

                // Only include simple names (no types)
                let name = name.split(':').next().unwrap_or(name).trim();

                if !name.is_empty() && !name.contains(' ') {
                    props.insert(name.to_string(), value.to_string());
                }
            }
        }

        props
    }

    fn parse_class_name(&self, source: &str) -> String {
        // Look for: export default class ClassName
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.contains("export") && trimmed.contains("default") && trimmed.contains("class") {
                // Extract class name after "class"
                if let Some(class_pos) = trimmed.find("class") {
                    let after_class = &trimmed[class_pos + 5..].trim_start();
                    let class_name: String = after_class
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !class_name.is_empty() {
                        return class_name;
                    }
                }
            }
        }
        "UnnamedScript".to_string()
    }
}

impl Default for TypeScriptCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Compilation error: {0}")]
    Compilation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_execution_order() {
        let compiler = TypeScriptCompiler::new();

        let source = r#"
export default class Test {
    static executionOrder = -10;
}
"#;
        assert_eq!(compiler.parse_execution_order(source), -10);
    }

    #[test]
    fn test_parse_execution_order_default() {
        let compiler = TypeScriptCompiler::new();
        let source = "export default class Test {}";
        assert_eq!(compiler.parse_execution_order(source), 0);
    }

    #[test]
    fn test_parse_properties() {
        let compiler = TypeScriptCompiler::new();

        let source = r#"
export default class Test {
    speed = 5.0;
    name = "player";
    active = true;

    onUpdate() {}
}
"#;
        let props = compiler.parse_properties(source);
        assert_eq!(props.get("speed"), Some(&"5.0".to_string()));
        assert_eq!(props.get("name"), Some(&"\"player\"".to_string()));
        assert_eq!(props.get("active"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_class_name() {
        let compiler = TypeScriptCompiler::new();

        let source = r#"
export default class PlayerController {
    speed = 5.0;
}
"#;
        assert_eq!(compiler.parse_class_name(source), "PlayerController");
    }

    #[test]
    fn test_parse_class_name_default() {
        let compiler = TypeScriptCompiler::new();
        let source = "const x = 1;";
        assert_eq!(compiler.parse_class_name(source), "UnnamedScript");
    }
}
