use crate::DesignConstraints;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_header_height_is_used_in_unity_editor() {
        // Read the Unity editor main.rs to verify it uses panel header height
        let editor_css = include_str!("../../../engine-editor/src/main.rs");
        
        // The Unity editor should be using the --panel-header-height CSS variable
        assert!(
            editor_css.contains("--panel-header-height") || 
            editor_css.contains("panel-header-height") ||
            editor_css.contains("panel_header_height"),
            "Unity editor CSS does not use panel header height setting"
        );
        
        println!("✅ Panel header height is referenced in Unity editor");
    }
    
    #[test]
    fn test_toolbar_height_is_used_in_unity_editor() {
        let editor_css = include_str!("../../../engine-editor/src/main.rs");
        
        assert!(
            editor_css.contains("--toolbar-height") || 
            editor_css.contains("toolbar-height") ||
            editor_css.contains("toolbar_height"),
            "Unity editor CSS does not use toolbar height setting"
        );
        
        println!("✅ Toolbar height is referenced in Unity editor");
    }
    
    #[test]
    fn test_sidebar_background_is_used_in_unity_editor() {
        let editor_css = include_str!("../../../engine-editor/src/main.rs");
        
        assert!(
            editor_css.contains("--sidebar-background") || 
            editor_css.contains("sidebar-background") ||
            editor_css.contains("sidebar_background"),
            "Unity editor CSS does not use sidebar background setting"
        );
        
        println!("✅ Sidebar background is referenced in Unity editor");
    }
    
    #[test]
    fn test_panel_padding_is_used_in_unity_editor() {
        let editor_css = include_str!("../../../engine-editor/src/main.rs");
        
        assert!(
            editor_css.contains("--panel-padding") || 
            editor_css.contains("panel-padding") ||
            editor_css.contains("panel_padding"),
            "Unity editor CSS does not use panel padding setting"
        );
        
        println!("✅ Panel padding is referenced in Unity editor");
    }
    
    #[test]
    fn test_button_height_variants_used_in_unity_editor() {
        let editor_css = include_str!("../../../engine-editor/src/main.rs");
        
        // Check that different button heights are used
        let has_sm = editor_css.contains("--button-height-sm") || editor_css.contains("button-height-sm");
        let has_md = editor_css.contains("--button-height-md") || editor_css.contains("button-height-md");
        let has_lg = editor_css.contains("--button-height-lg") || editor_css.contains("button-height-lg");
        
        assert!(
            has_sm || has_md || has_lg,
            "Unity editor CSS does not use any specific button height variants (sm, md, lg)"
        );
        
        println!("✅ Button height variants are referenced in Unity editor");
    }
    
    #[test]
    fn test_border_radius_variants_used_in_unity_editor() {
        let editor_css = include_str!("../../../engine-editor/src/main.rs");
        
        let has_sm = editor_css.contains("--border-radius-sm") || editor_css.contains("border-radius-sm");
        let has_md = editor_css.contains("--border-radius-md") || editor_css.contains("border-radius-md");
        let has_lg = editor_css.contains("--border-radius-lg") || editor_css.contains("border-radius-lg");
        
        assert!(
            has_sm || has_md || has_lg,
            "Unity editor CSS does not use any specific border radius variants"
        );
        
        println!("✅ Border radius variants are referenced in Unity editor");
    }
    
    #[test] 
    fn test_spacing_variants_used_in_unity_editor() {
        let editor_css = include_str!("../../../engine-editor/src/main.rs");
        
        let has_xs = editor_css.contains("--space-xs") || editor_css.contains("space-xs");
        let has_sm = editor_css.contains("--space-sm") || editor_css.contains("space-sm");
        let has_md = editor_css.contains("--space-md") || editor_css.contains("space-md");
        let has_lg = editor_css.contains("--space-lg") || editor_css.contains("space-lg");
        let has_xl = editor_css.contains("--space-xl") || editor_css.contains("space-xl");
        
        assert!(
            has_xs || has_sm || has_md || has_lg || has_xl,
            "Unity editor CSS does not use any specific spacing variants"
        );
        
        println!("✅ Spacing variants are referenced in Unity editor");
    }
    
    #[test]
    fn test_input_height_variants_used_in_unity_editor() {
        let editor_css = include_str!("../../../engine-editor/src/main.rs");
        
        let has_sm = editor_css.contains("--input-height-sm") || editor_css.contains("input-height-sm");
        let has_md = editor_css.contains("--input-height-md") || editor_css.contains("input-height-md");
        let has_lg = editor_css.contains("--input-height-lg") || editor_css.contains("input-height-lg");
        
        assert!(
            has_sm || has_md || has_lg,
            "Unity editor CSS does not use any specific input height variants"
        );
        
        println!("✅ Input height variants are referenced in Unity editor");
    }
    
    #[test]
    fn test_comprehensive_css_variable_usage_in_unity_editor() {
        let editor_css = include_str!("../../../engine-editor/src/main.rs");
        
        // Critical CSS variables that MUST be used for the editor to work properly
        let critical_variables = [
            // Panel styling that user complained about
            "panel-header-height",
            "toolbar-height", 
            "sidebar-background",
            "toolbar-background",
            "panel-background",
            "panel-padding",
            
            // Button variants
            "button-height",
            "button-primary",
            "button-secondary",
            
            // Input styling
            "input-height",
            "input-background",
            "input-border",
            
            // Text styling  
            "text-primary",
            "text-secondary",
            
            // Spacing that affects layout
            "space-sm",
            "space-md",
            "space-lg",
            
            // Border styling
            "border-radius",
            "border-primary",
            
            // Typography
            "font-primary",
            "font-size-base",
        ];
        
        let mut missing_variables = Vec::new();
        let mut found_count = 0;
        
        for var in critical_variables {
            let found = editor_css.contains(&format!("--{}", var)) || 
                       editor_css.contains(&format!("var(--{})", var)) ||
                       editor_css.contains(var);
            
            if found {
                found_count += 1;
                println!("✅ Found CSS variable: {}", var);
            } else {
                missing_variables.push(var);
                println!("❌ Missing CSS variable: {}", var);
            }
        }
        
        if !missing_variables.is_empty() {
            panic!(
                "Unity editor is missing {} critical CSS variables: {:?}\nOnly {}/{} variables are being used!",
                missing_variables.len(),
                missing_variables,
                found_count,
                critical_variables.len()
            );
        }
        
        println!("✅ All {} critical CSS variables are being used in Unity editor!", critical_variables.len());
    }
    
    #[test]
    fn test_css_var_function_usage() {
        let editor_css = include_str!("../../../engine-editor/src/main.rs");
        
        // Count how many var() functions are used
        let var_usage_count = editor_css.matches("var(--").count();
        
        assert!(
            var_usage_count >= 10,
            "Unity editor only uses {} var() CSS functions, expected at least 10",
            var_usage_count
        );
        
        println!("✅ Unity editor uses {} var() CSS functions", var_usage_count);
    }
}