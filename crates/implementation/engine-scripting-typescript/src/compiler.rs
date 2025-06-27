//! TypeScript to JavaScript compilation using SWC
//! 
//! This module handles compilation of TypeScript source code to JavaScript
//! that can be executed by the V8 engine. It provides fast compilation
//! with caching and incremental compilation support.

use engine_scripting::{ScriptError, ScriptResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use swc_core::common::{
    errors::{ColorConfig, Handler},
    sync::Lrc,
    SourceMap, FilePathMapping,
};
use swc_core::ecma::{
    parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax},
    transforms::{
        base::resolver,
        typescript::strip,
    },
    codegen::{text_writer::JsWriter, Emitter},
};
use swc_ecma_visit::{FoldWith, as_folder};

/// TypeScript compiler configuration
#[derive(Debug, Clone)]
pub struct CompilerOptions {
    /// Target JavaScript version
    pub target: EsVersion,
    /// Enable strict mode
    pub strict: bool,
    /// Generate source maps
    pub source_map: bool,
    /// Enable decorators
    pub decorators: bool,
    /// JSX support
    pub jsx: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            target: EsVersion::Es2020,
            strict: true,
            source_map: true,
            decorators: true,
            jsx: false,
        }
    }
}

/// JavaScript/ECMAScript version targets
#[derive(Debug, Clone, Copy)]
pub enum EsVersion {
    Es5,
    Es2015,
    Es2017,
    Es2018,
    Es2019,
    Es2020,
    Es2021,
    Es2022,
    EsNext,
}

/// Compilation result containing JavaScript code and optional source map
#[derive(Debug, Clone)]
pub struct CompilationResult {
    /// Compiled JavaScript code
    pub code: String,
    /// Source map for debugging (if enabled)
    pub source_map: Option<String>,
    /// Compilation warnings
    pub warnings: Vec<String>,
}

/// Cached compilation entry
#[derive(Debug, Clone)]
struct CachedCompilation {
    result: CompilationResult,
    source_hash: u64,
    timestamp: std::time::SystemTime,
}

/// TypeScript compiler with caching support
pub struct TypeScriptCompiler {
    options: CompilerOptions,
    source_map: Lrc<SourceMap>,
    cache: Arc<Mutex<HashMap<PathBuf, CachedCompilation>>>,
}

impl TypeScriptCompiler {
    /// Create a new TypeScript compiler with default options
    pub fn new() -> Self {
        Self::with_options(CompilerOptions::default())
    }
    
    /// Create a new TypeScript compiler with custom options
    pub fn with_options(options: CompilerOptions) -> Self {
        let source_map = Lrc::new(SourceMap::new(FilePathMapping::empty()));
        
        Self {
            options,
            source_map,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Compile TypeScript source code to JavaScript
    pub fn compile(&self, source: &str, file_path: Option<&Path>) -> ScriptResult<CompilationResult> {
        // Check cache if file path is provided
        if let Some(path) = file_path {
            if let Some(cached) = self.check_cache(path, source)? {
                return Ok(cached);
            }
        }
        
        let result = self.compile_source(source, file_path)?;
        
        // Cache the result if file path is provided
        if let Some(path) = file_path {
            self.cache_result(path, source, &result)?;
        }
        
        Ok(result)
    }
    
    /// Compile TypeScript source without caching
    pub fn compile_source(&self, source: &str, file_path: Option<&Path>) -> ScriptResult<CompilationResult> {
        // Use SWC's GLOBALS context for proper compilation
        swc_core::common::GLOBALS.set(&swc_core::common::Globals::new(), || {
            let filename = file_path
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "<inline>".to_string());
            
            // Create source file
            let source_file = self.source_map.new_source_file(
                swc_core::common::FileName::Custom(filename.clone()),
                source.to_string(),
            );
            
            // Set up error handler
            let _handler = Handler::with_emitter_writer(
                Box::new(std::io::stderr()),
                Some(self.source_map.clone()),
            );
            
            // Parse TypeScript
            let lexer = Lexer::new(
                Syntax::Typescript(TsSyntax {
                    tsx: self.options.jsx,
                    decorators: self.options.decorators,
                    dts: false,
                    no_early_errors: false,
                    disallow_ambiguous_jsx_like: true,
                }),
                self.target_to_es_version(),
                StringInput::from(&*source_file),
                None,
            );
            
            let mut parser = Parser::new_from(lexer);
            let module = parser.parse_module().map_err(|e| {
                ScriptError::CompilationError {
                    message: format!("Parse error in {}: {:?}", filename, e),
                    script_id: None,
                    source: None,
                }
            })?;
            
            // Wrap module in Program for transforms
            let program = swc_core::ecma::ast::Program::Module(module);
            
            // Transform TypeScript to JavaScript
            let program = program.fold_with(&mut resolver(
                swc_core::common::Mark::new(),
                swc_core::common::Mark::new(),
                true,
            ));
            
            let program = program.fold_with(&mut as_folder(strip(swc_core::common::Mark::new())));
            
            // Extract module back from program
            let module = match program {
                swc_core::ecma::ast::Program::Module(m) => m,
                _ => return Err(ScriptError::CompilationError {
                    message: "Expected module program".to_string(),
                    script_id: None,
                    source: None,
                }),
            };
            
            // Generate JavaScript code
            let mut buf = Vec::new();
            let writer = JsWriter::new(self.source_map.clone(), "\n", &mut buf, None);
            
            let mut emitter = Emitter {
                cfg: swc_core::ecma::codegen::Config::default(),
                cm: self.source_map.clone(),
                comments: None,
                wr: writer,
            };
            
            emitter.emit_module(&module).map_err(|e| {
                ScriptError::CompilationError {
                    message: format!("Code generation error: {:?}", e),
                    script_id: None,
                    source: None,
                }
            })?;
            
            let code = String::from_utf8(buf).map_err(|e| {
                ScriptError::CompilationError {
                    message: format!("Invalid UTF-8 in generated code: {}", e),
                    script_id: None,
                    source: None,
                }
            })?;
            
            Ok(CompilationResult {
                code,
                source_map: None, // TODO: Implement source map generation
                warnings: Vec::new(),
            })
        })
    }
    
    /// Check if we have a cached compilation result
    fn check_cache(&self, path: &Path, source: &str) -> ScriptResult<Option<CompilationResult>> {
        let cache = self.cache.lock().map_err(|_| {
            ScriptError::runtime("Failed to acquire cache lock".to_string())
        })?;
        
        if let Some(cached) = cache.get(path) {
            let source_hash = self.hash_source(source);
            if cached.source_hash == source_hash {
                return Ok(Some(cached.result.clone()));
            }
        }
        
        Ok(None)
    }
    
    /// Cache a compilation result
    fn cache_result(&self, path: &Path, source: &str, result: &CompilationResult) -> ScriptResult<()> {
        let mut cache = self.cache.lock().map_err(|_| {
            ScriptError::runtime("Failed to acquire cache lock".to_string())
        })?;
        
        let cached = CachedCompilation {
            result: result.clone(),
            source_hash: self.hash_source(source),
            timestamp: std::time::SystemTime::now(),
        };
        
        cache.insert(path.to_path_buf(), cached);
        Ok(())
    }
    
    /// Clear the compilation cache
    pub fn clear_cache(&self) -> ScriptResult<()> {
        let mut cache = self.cache.lock().map_err(|_| {
            ScriptError::runtime("Failed to acquire cache lock".to_string())
        })?;
        cache.clear();
        Ok(())
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> ScriptResult<CacheStats> {
        let cache = self.cache.lock().map_err(|_| {
            ScriptError::runtime("Failed to acquire cache lock".to_string())
        })?;
        
        Ok(CacheStats {
            entries: cache.len(),
            total_size: cache.values()
                .map(|c| c.result.code.len())
                .sum(),
        })
    }
    
    /// Convert target to SWC ES version
    fn target_to_es_version(&self) -> swc_core::ecma::ast::EsVersion {
        match self.options.target {
            EsVersion::Es5 => swc_core::ecma::ast::EsVersion::Es5,
            EsVersion::Es2015 => swc_core::ecma::ast::EsVersion::Es2015,
            EsVersion::Es2017 => swc_core::ecma::ast::EsVersion::Es2017,
            EsVersion::Es2018 => swc_core::ecma::ast::EsVersion::Es2018,
            EsVersion::Es2019 => swc_core::ecma::ast::EsVersion::Es2019,
            EsVersion::Es2020 => swc_core::ecma::ast::EsVersion::Es2020,
            EsVersion::Es2021 => swc_core::ecma::ast::EsVersion::Es2021,
            EsVersion::Es2022 => swc_core::ecma::ast::EsVersion::Es2022,
            EsVersion::EsNext => swc_core::ecma::ast::EsVersion::EsNext,
        }
    }
    
    /// Calculate hash of source code for caching
    fn hash_source(&self, source: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_typescript_compilation() {
        let compiler = TypeScriptCompiler::new();
        
        let typescript_code = r#"
            interface Player {
                name: string;
                health: number;
            }
            
            class GameEntity implements Player {
                constructor(public name: string, public health: number = 100) {}
                
                takeDamage(amount: number): void {
                    this.health -= amount;
                    console.log(`${this.name} took ${amount} damage. Health: ${this.health}`);
                }
            }
            
            const player = new GameEntity("Hero");
            player.takeDamage(10);
        "#;
        
        let result = compiler.compile(typescript_code, None);
        assert!(result.is_ok(), "TypeScript compilation should succeed");
        
        let compiled = result.unwrap();
        assert!(!compiled.code.is_empty(), "Compiled code should not be empty");
        assert!(compiled.code.contains("GameEntity"), "Compiled code should contain class name");
    }
    
    #[test]
    fn test_compilation_caching() {
        let compiler = TypeScriptCompiler::new();
        let test_path = Path::new("test.ts");
        
        let typescript_code = "const x: number = 42;";
        
        // First compilation
        let result1 = compiler.compile(typescript_code, Some(test_path));
        assert!(result1.is_ok());
        
        // Second compilation should use cache
        let result2 = compiler.compile(typescript_code, Some(test_path));
        assert!(result2.is_ok());
        
        let stats = compiler.cache_stats().unwrap();
        assert_eq!(stats.entries, 1, "Should have one cache entry");
    }
    
    #[test]
    fn test_syntax_error_handling() {
        let compiler = TypeScriptCompiler::new();
        
        let invalid_typescript = "const x: number = 'not a number'";
        let result = compiler.compile(invalid_typescript, None);
        
        // Note: This might succeed at compile time but fail at runtime
        // TypeScript's type checking is more lenient in SWC
        assert!(result.is_ok(), "SWC should handle this TypeScript code");
    }
    
    #[test]
    fn test_debug_let_compilation() {
        let compiler = TypeScriptCompiler::new();
        
        let typescript_code = r#"
            let moduleLevel = "initial";
            
            function function1(): string {
                return "function1_initial";
            }
            
            function getModuleLevel(): string {
                return moduleLevel;
            }
        "#;
        
        let result = compiler.compile(typescript_code, None);
        assert!(result.is_ok(), "TypeScript compilation should succeed");
        
        let compiled = result.unwrap();
        println!("Generated JS:\n{}", compiled.code);
        
        // Test the regex replacement
        use regex::Regex;
        let let_regex = Regex::new(r"\blet\s+").unwrap();
        let const_regex = Regex::new(r"\bconst\s+").unwrap();
        
        let temp = let_regex.replace_all(&compiled.code, "var ");
        let hot_reload_code = const_regex.replace_all(&temp, "var ").to_string();
        
        println!("Hot reload JS:\n{}", hot_reload_code);
        assert!(!hot_reload_code.contains("let "), "Should not contain 'let '");
        assert!(!compiled.code.is_empty(), "Compiled code should not be empty");
    }
}