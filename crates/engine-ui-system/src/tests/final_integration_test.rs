use crate::design_loader;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comprehensive_css_generation_from_current_constraints() {
        // Load the actual current design constraints (if they exist)
        let constraints = design_loader::load_current_design();
        let css = constraints.to_css();
        
        println!("Current CSS generated from constraints:");
        println!("{}", css);
        
        // Test that all major categories are included
        assert!(css.contains("/* Background Colors */"), "Background colors section missing");
        assert!(css.contains("/* Button Colors */"), "Button colors section missing");
        assert!(css.contains("/* Input Colors */"), "Input colors section missing");
        assert!(css.contains("/* Text Colors */"), "Text colors section missing");
        assert!(css.contains("/* Border Colors */"), "Border colors section missing");
        assert!(css.contains("/* Status Colors */"), "Status colors section missing");
        assert!(css.contains("/* Typography */"), "Typography section missing");
        assert!(css.contains("/* Button Geometry */"), "Button geometry section missing");
        assert!(css.contains("/* Input Geometry */"), "Input geometry section missing");
        assert!(css.contains("/* Panel Geometry */"), "Panel geometry section missing");
        assert!(css.contains("/* Border Radii */"), "Border radii section missing");
        assert!(css.contains("/* Border Widths */"), "Border widths section missing");
        assert!(css.contains("/* Icon Sizes */"), "Icon sizes section missing");
        assert!(css.contains("/* Spacing */"), "Spacing section missing");
        assert!(css.contains("/* Effects */"), "Effects section missing");
        assert!(css.contains("/* Legacy CSS Variables for Backward Compatibility */"), "Legacy variables section missing");
        
        // Test that specific panel settings that were problematic are now included
        assert!(css.contains("--sidebar-background"), "Sidebar background variable missing");
        assert!(css.contains("--toolbar-background"), "Toolbar background variable missing");
        assert!(css.contains("--button-secondary-bg"), "Secondary button variable missing");
        assert!(css.contains("--button-danger-bg"), "Danger button variable missing");
        assert!(css.contains("--input-border"), "Input border variable missing");
        assert!(css.contains("--text-accent"), "Text accent variable missing");
        assert!(css.contains("--panel-padding"), "Panel padding variable missing");
        assert!(css.contains("--toolbar-height"), "Toolbar height variable missing");
        
        // Test that different button and input sizes are included
        assert!(css.contains("--button-height-sm"), "Small button height missing");
        assert!(css.contains("--button-height-md"), "Medium button height missing");
        assert!(css.contains("--button-height-lg"), "Large button height missing");
        assert!(css.contains("--input-height-sm"), "Small input height missing");
        assert!(css.contains("--input-height-md"), "Medium input height missing");
        assert!(css.contains("--input-height-lg"), "Large input height missing");
        
        // Test that different border radii are included
        assert!(css.contains("--border-radius-sm"), "Small border radius missing");
        assert!(css.contains("--border-radius-md"), "Medium border radius missing");
        assert!(css.contains("--border-radius-lg"), "Large border radius missing");
        
        // Test that all spacing sizes are included
        assert!(css.contains("--space-xs"), "Extra small spacing missing");
        assert!(css.contains("--space-sm"), "Small spacing missing");
        assert!(css.contains("--space-md"), "Medium spacing missing");
        assert!(css.contains("--space-lg"), "Large spacing missing");
        assert!(css.contains("--space-xl"), "Extra large spacing missing");
        
        println!("✅ Comprehensive CSS generation test passed - ALL settings are now included!");
    }
    
    #[test]
    fn test_settings_count_verification() {
        let constraints = design_loader::load_current_design();
        let css = constraints.to_css();
        
        // Count the number of CSS variables generated
        let variable_count = css.matches("--").count();
        
        // We should have at least 50+ variables for a comprehensive system
        assert!(variable_count >= 50, "Only {} CSS variables generated, expected at least 50", variable_count);
        
        println!("✅ Generated {} CSS variables - comprehensive coverage achieved!", variable_count);
    }
}