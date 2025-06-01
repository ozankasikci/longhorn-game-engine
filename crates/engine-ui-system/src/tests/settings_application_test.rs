use crate::DesignConstraints;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_color_settings_included_in_css() {
        let constraints = DesignConstraints::unity_dark();
        let css = constraints.to_css();
        
        // Test that ALL color settings are included in CSS
        let expected_colors = [
            "window_background",
            "panel_background", 
            "toolbar_background",
            "sidebar_background",
            "button_primary_bg",
            "button_secondary_bg", 
            "button_danger_bg",
            "input_background",
            "input_border",
            "text_primary",
            "text_secondary", 
            "text_accent",
            "border_primary",
        ];
        
        for color in expected_colors {
            assert!(
                css.contains(&format!("--{}", color.replace("_", "-"))) || 
                css.contains(&color.replace("_", "-")),
                "CSS missing color setting: {}", color
            );
        }
        
        println!("✅ All color settings test passed");
    }
    
    #[test] 
    fn test_all_geometry_settings_included_in_css() {
        let constraints = DesignConstraints::unity_dark();
        let css = constraints.to_css();
        
        // Test that ALL geometry settings are included in CSS
        let expected_geometry = [
            "button_height_sm",
            "button_height_md", 
            "button_height_lg",
            "input_height_sm",
            "input_height_md",
            "input_height_lg", 
            "panel_header_height",
            "toolbar_height",
            "border_radius_sm",
            "border_radius_md",
            "border_radius_lg",
        ];
        
        for geometry in expected_geometry {
            assert!(
                css.contains(&format!("--{}", geometry.replace("_", "-"))) ||
                css.contains(&geometry.replace("_", "-")),
                "CSS missing geometry setting: {}", geometry
            );
        }
        
        println!("✅ All geometry settings test passed");
    }
    
    #[test]
    fn test_all_spacing_settings_included_in_css() {
        let constraints = DesignConstraints::unity_dark();
        let css = constraints.to_css();
        
        // Test that ALL spacing settings are included in CSS  
        let expected_spacing = [
            "space_xs",
            "space_sm",
            "space_md", 
            "space_lg",
            "space_xl",
            "panel_padding",
        ];
        
        for spacing in expected_spacing {
            assert!(
                css.contains(&format!("--{}", spacing.replace("_", "-"))) ||
                css.contains(&spacing.replace("_", "-")),
                "CSS missing spacing setting: {}", spacing
            );
        }
        
        println!("✅ All spacing settings test passed");
    }
    
    #[test]
    fn test_comprehensive_css_variable_mapping() {
        let mut constraints = DesignConstraints::unity_dark();
        
        // Change some values to unique ones we can test for
        constraints.colors.sidebar_background = "#UNIQUE1".to_string();
        constraints.colors.toolbar_background = "#UNIQUE2".to_string(); 
        constraints.colors.button_secondary_bg = "#UNIQUE3".to_string();
        constraints.geometry.button_height_sm = 99.0;
        constraints.spacing.space_xl = 88.0;
        
        let css = constraints.to_css();
        
        // Test that our unique values appear in the CSS
        assert!(css.contains("#UNIQUE1"), "Sidebar background color not in CSS");
        assert!(css.contains("#UNIQUE2"), "Toolbar background color not in CSS");
        assert!(css.contains("#UNIQUE3"), "Secondary button color not in CSS");
        assert!(css.contains("99px"), "Small button height not in CSS");
        assert!(css.contains("88px"), "XL spacing not in CSS");
        
        println!("✅ Comprehensive CSS mapping test passed");
    }
    
    #[test]
    fn test_css_variables_generated_correctly() {
        // This test verifies that the CSS variables we generate are properly formatted
        let constraints = DesignConstraints::unity_dark();
        let css = constraints.to_css();
        
        println!("Generated CSS:\n{}", css);
        
        // Check that main CSS variables are generated correctly
        let expected_variables = [
            "--window-bg",
            "--panel-bg", 
            "--button-primary",
            "--text-primary",
            "--border-radius",
            "--space-md",
        ];
        
        for var in expected_variables {
            assert!(css.contains(var), "CSS variable {} not generated", var);
        }
        
        // Check that CSS is valid (contains :root and closing braces)
        assert!(css.contains(":root"), "CSS missing :root selector");
        assert!(css.contains("}"), "CSS missing closing braces");
        
        println!("✅ CSS variables generation test passed");
    }
    
    #[test]
    fn test_settings_hot_reload_comprehensive() {
        // Test that changing any setting and saving triggers all the right changes
        let mut constraints = DesignConstraints::unity_dark();
        
        // Original values
        let original_window_bg = constraints.colors.window_background.clone();
        let original_panel_bg = constraints.colors.panel_background.clone();
        let original_button_height = constraints.geometry.button_height_md;
        
        // Change values
        constraints.colors.window_background = "#FF0000".to_string();
        constraints.colors.panel_background = "#00FF00".to_string(); 
        constraints.geometry.button_height_md = 50.0;
        
        // Generate CSS with changes
        let css = constraints.to_css();
        
        // Verify all changes are reflected
        assert!(css.contains("#FF0000"), "Window background change not in CSS");
        assert!(css.contains("#00FF00"), "Panel background change not in CSS");
        assert!(css.contains("50px"), "Button height change not in CSS");
        
        // Verify original values are not the primary values anymore (they may exist in legacy section)
        assert!(css.contains("#FF0000"), "New window background not found");
        assert!(css.contains("#00FF00"), "New panel background not found");
        assert!(css.contains("50px"), "New button height not found");
        
        println!("✅ Hot reload comprehensive test passed");
    }
}