//! Tests for Lua script debugging system

#[cfg(test)]
mod tests {
    use crate::lua::debugging::{
        LuaDebugger, DebugInfo, BreakpointId, CallStackFrame, VariableInfo
    };
    use crate::ScriptError;
    use mlua::{Lua, Function as LuaFunction};
    use std::collections::HashMap;
    
    #[test]
    fn test_debugger_creation() {
        let lua = Lua::new();
        let debugger = LuaDebugger::new(&lua).unwrap();
        
        assert_eq!(debugger.breakpoint_count(), 0);
        assert!(!debugger.is_paused());
        assert!(debugger.get_call_stack().is_empty());
    }
    
    #[test]
    fn test_breakpoint_management() {
        let lua = Lua::new();
        let mut debugger = LuaDebugger::new(&lua).unwrap();
        
        // Add breakpoint
        let bp_id = debugger.add_breakpoint("test_script.lua", 10).unwrap();
        assert_eq!(debugger.breakpoint_count(), 1);
        assert!(debugger.has_breakpoint("test_script.lua", 10));
        
        // Add another breakpoint
        let bp_id2 = debugger.add_breakpoint("test_script.lua", 15).unwrap();
        assert_eq!(debugger.breakpoint_count(), 2);
        
        // Remove breakpoint
        debugger.remove_breakpoint(bp_id).unwrap();
        assert_eq!(debugger.breakpoint_count(), 1);
        assert!(!debugger.has_breakpoint("test_script.lua", 10));
        assert!(debugger.has_breakpoint("test_script.lua", 15));
        
        // Clear all breakpoints
        debugger.clear_breakpoints();
        assert_eq!(debugger.breakpoint_count(), 0);
    }
    
    #[test]
    fn test_call_stack_inspection() {
        let lua = Lua::new();
        let mut debugger = LuaDebugger::new(&lua).unwrap();
        
        // Simulate execution with call stack
        debugger.push_call_frame("main.lua", "main", 1).unwrap();
        debugger.push_call_frame("utils.lua", "helper_function", 25).unwrap();
        debugger.push_call_frame("math.lua", "calculate", 42).unwrap();
        
        let call_stack = debugger.get_call_stack();
        assert_eq!(call_stack.len(), 3);
        
        // Check top frame (most recent call)
        let top_frame = &call_stack[0];
        assert_eq!(top_frame.file_name, "math.lua");
        assert_eq!(top_frame.function_name, "calculate");
        assert_eq!(top_frame.line_number, 42);
        
        // Check bottom frame (original call)
        let bottom_frame = &call_stack[2];
        assert_eq!(bottom_frame.file_name, "main.lua");
        assert_eq!(bottom_frame.function_name, "main");
        assert_eq!(bottom_frame.line_number, 1);
        
        // Pop frame
        debugger.pop_call_frame().unwrap();
        assert_eq!(debugger.get_call_stack().len(), 2);
    }
    
    #[test]
    fn test_variable_inspection() {
        let lua = Lua::new();
        let mut debugger = LuaDebugger::new(&lua).unwrap();
        
        // Add some variables to inspect
        debugger.add_local_variable("x", "number", "42".to_string()).unwrap();
        debugger.add_local_variable("name", "string", "\"hello\"".to_string()).unwrap();
        debugger.add_local_variable("valid", "boolean", "true".to_string()).unwrap();
        
        let variables = debugger.get_local_variables();
        assert_eq!(variables.len(), 3);
        
        // Check variable details
        let x_var = variables.iter().find(|v| v.name == "x").unwrap();
        assert_eq!(x_var.var_type, "number");
        assert_eq!(x_var.value, "42");
        
        let name_var = variables.iter().find(|v| v.name == "name").unwrap();
        assert_eq!(name_var.var_type, "string");
        assert_eq!(name_var.value, "\"hello\"");
        
        // Clear variables
        debugger.clear_local_variables();
        assert!(debugger.get_local_variables().is_empty());
    }
    
    #[test]
    fn test_execution_control() {
        let lua = Lua::new();
        let mut debugger = LuaDebugger::new(&lua).unwrap();
        
        // Initially not paused
        assert!(!debugger.is_paused());
        
        // Pause execution
        debugger.pause().unwrap();
        assert!(debugger.is_paused());
        
        // Step into
        debugger.step_into().unwrap();
        assert!(debugger.is_paused()); // Should still be paused after step
        
        // Step over
        debugger.step_over().unwrap();
        assert!(debugger.is_paused());
        
        // Continue execution
        debugger.continue_execution().unwrap();
        assert!(!debugger.is_paused());
    }
    
    #[test]
    fn test_error_handling_with_debug_info() {
        let lua = Lua::new();
        let mut debugger = LuaDebugger::new(&lua).unwrap();
        
        // Simulate an error with debug information
        let error_info = DebugInfo {
            file_name: "test.lua".to_string(),
            line_number: 15,
            function_name: Some("buggy_function".to_string()),
            error_message: "attempt to index a nil value".to_string(),
            call_stack: vec![
                CallStackFrame {
                    file_name: "test.lua".to_string(),
                    function_name: "buggy_function".to_string(),
                    line_number: 15,
                    local_variables: HashMap::new(),
                },
                CallStackFrame {
                    file_name: "main.lua".to_string(),
                    function_name: "main".to_string(),
                    line_number: 5,
                    local_variables: HashMap::new(),
                }
            ],
        };
        
        debugger.handle_runtime_error(error_info).unwrap();
        
        // Should be paused on error
        assert!(debugger.is_paused());
        
        // Should have captured call stack
        let call_stack = debugger.get_call_stack();
        assert_eq!(call_stack.len(), 2);
        assert_eq!(call_stack[0].function_name, "buggy_function");
    }
    
    #[test]
    fn test_conditional_breakpoints() {
        let lua = Lua::new();
        let mut debugger = LuaDebugger::new(&lua).unwrap();
        
        // Add conditional breakpoint
        let bp_id = debugger.add_conditional_breakpoint(
            "loop.lua", 
            10, 
            "i > 5".to_string()
        ).unwrap();
        
        assert_eq!(debugger.breakpoint_count(), 1);
        
        // Test condition evaluation
        assert!(!debugger.should_break_at("loop.lua", 10, &[("i", "3")]).unwrap());
        assert!(debugger.should_break_at("loop.lua", 10, &[("i", "7")]).unwrap());
    }
    
    #[test]
    fn test_watch_expressions() {
        let lua = Lua::new();
        let mut debugger = LuaDebugger::new(&lua).unwrap();
        
        // Add watch expressions
        let watch_id1 = debugger.add_watch_expression("player.health".to_string()).unwrap();
        let watch_id2 = debugger.add_watch_expression("x + y".to_string()).unwrap();
        
        assert_eq!(debugger.watch_expression_count(), 2);
        
        // Evaluate watch expressions
        debugger.set_context_variable("player", "{ health = 100 }");
        debugger.set_context_variable("x", "10");
        debugger.set_context_variable("y", "20");
        
        let watch_results = debugger.evaluate_watch_expressions().unwrap();
        assert_eq!(watch_results.len(), 2);
        
        // Remove watch expression
        debugger.remove_watch_expression(watch_id1).unwrap();
        assert_eq!(debugger.watch_expression_count(), 1);
    }
    
    #[test]
    fn test_debug_api_registration() {
        let lua = Lua::new();
        let debugger = LuaDebugger::new(&lua).unwrap();
        
        // Register debug API
        debugger.register_debug_api(&lua).unwrap();
        
        // Check that debug functions are available
        let globals = lua.globals();
        assert!(globals.get::<_, LuaFunction>("debug_print").is_ok());
        assert!(globals.get::<_, LuaFunction>("debug_inspect").is_ok());
        assert!(globals.get::<_, LuaFunction>("debug_break").is_ok());
        
        // Test debug_print function from Lua
        lua.load(r#"
            debug_print("Hello from script!")
        "#).exec().unwrap();
        
        // Test debug_inspect function
        lua.load(r#"
            local x = 42
            debug_inspect("x", x)
        "#).exec().unwrap();
    }
    
    #[test]
    fn test_step_execution_modes() {
        let lua = Lua::new();
        let mut debugger = LuaDebugger::new(&lua).unwrap();
        
        // Set up execution context
        debugger.push_call_frame("test.lua", "main", 1).unwrap();
        debugger.pause().unwrap();
        
        // Test step into
        debugger.step_into().unwrap();
        assert_eq!(debugger.get_current_line(), Some(2));
        
        // Test step over (should skip function calls)
        debugger.step_over().unwrap();
        assert_eq!(debugger.get_current_line(), Some(3));
        
        // Test step out (should exit current function)
        debugger.push_call_frame("test.lua", "helper", 10).unwrap();
        debugger.step_out().unwrap();
        assert_eq!(debugger.get_call_stack().len(), 1);
    }
}