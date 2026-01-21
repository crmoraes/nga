use once_cell::sync::Lazy;
use regex::Regex;
use crate::models::ConversionRules;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Default alert message when variables are converted
pub const DEFAULT_VARIABLE_ALERT_MESSAGE: &str = "Variables within instructions will be converted to @variables format";

/// Default status suffix when variables are converted
pub const DEFAULT_VARIABLE_STATUS_SUFFIX: &str = "(variables converted to @variables format)";

// ============================================================================
// STATIC REGEX PATTERNS (compiled once at startup)
// ============================================================================

/// Fallback pattern - matches {!$...}, {$!...}, {$...}, or {!...} (not already @variables)
static DOLLAR_VAR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{[!]?\$[!]?[^}]+\}|\{![^@}][^}]*\}").expect("Invalid regex pattern for DOLLAR_VAR_PATTERN")
});

/// Pattern: {!$VarName} → {!@variables.VarName}
static VAR_PATTERN_1: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{!\$([^}]+)\}").expect("Invalid regex pattern for VAR_PATTERN_1")
});

/// Pattern: {$!VarName} → {!@variables.VarName}
static VAR_PATTERN_2: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\$!([^}]+)\}").expect("Invalid regex pattern for VAR_PATTERN_2")
});

/// Pattern: {$VarName} → {!@variables.VarName}
static VAR_PATTERN_3: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\$([^!}][^}]*)\}").expect("Invalid regex pattern for VAR_PATTERN_3")
});

/// Pattern: {!VarName} → {!@variables.VarName} (only if not already @variables)
static VAR_PATTERN_4: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{!([^@}][^}]*)\}").expect("Invalid regex pattern for VAR_PATTERN_4")
});

/// Check if input contains variables that need conversion to @variables format
pub fn check_for_dollar_variables(input: &str, rules: &Option<ConversionRules>) -> bool {
    if let Some(rules) = rules {
        if let Some(var_conv) = &rules.variable_conversion {
            if var_conv.enabled == Some(false) {
                return false;
            }
            
            if let Some(patterns) = &var_conv.patterns {
                for pattern_def in patterns {
                    if let Ok(re) = Regex::new(&pattern_def.pattern) {
                        if re.is_match(input) {
                            return true;
                        }
                    }
                }
                return false;
            }
        }
    }
    
    // Use pre-compiled fallback pattern
    DOLLAR_VAR_PATTERN.is_match(input)
}

/// Convert variables to @variables format
pub fn convert_variables_in_text(text: Option<&str>, rules: &Option<ConversionRules>) -> String {
    let text = match text {
        Some(t) => t,
        None => return String::new(),
    };
    
    // Check if variable conversion is enabled in rules
    if let Some(rules) = rules {
        if let Some(var_conv) = &rules.variable_conversion {
            if var_conv.enabled == Some(false) {
                return text.to_string();
            }
            
            if let Some(patterns) = &var_conv.patterns {
                let mut result = text.to_string();
                for pattern_def in patterns {
                    if let Ok(re) = Regex::new(&pattern_def.pattern) {
                        result = re.replace_all(&result, &pattern_def.replacement).to_string();
                    }
                }
                return result;
            }
        }
    }
    
    // Fallback: Handle all variable patterns using pre-compiled regex, converting to @variables format
    let mut result = text.to_string();
    
    // {!$Glossary} → {!@variables.Glossary}
    result = VAR_PATTERN_1.replace_all(&result, "{!@variables.$1}").to_string();
    
    // {$!Glossary} → {!@variables.Glossary}
    result = VAR_PATTERN_2.replace_all(&result, "{!@variables.$1}").to_string();
    
    // {$Glossary} → {!@variables.Glossary}
    result = VAR_PATTERN_3.replace_all(&result, "{!@variables.$1}").to_string();
    
    // {!Glossary} → {!@variables.Glossary} (only if not already @variables)
    result = VAR_PATTERN_4.replace_all(&result, "{!@variables.$1}").to_string();
    
    result
}

/// Get variable alert message from rules
pub fn get_variable_alert_message(rules: &Option<ConversionRules>) -> String {
    if let Some(rules) = rules {
        if let Some(var_conv) = &rules.variable_conversion {
            if let Some(msg) = &var_conv.alert_message {
                return msg.clone();
            }
        }
    }
    DEFAULT_VARIABLE_ALERT_MESSAGE.to_string()
}

/// Get variable status suffix from rules
pub fn get_variable_status_suffix(rules: &Option<ConversionRules>) -> String {
    if let Some(rules) = rules {
        if let Some(var_conv) = &rules.variable_conversion {
            if let Some(suffix) = &var_conv.status_suffix {
                return suffix.clone();
            }
        }
    }
    DEFAULT_VARIABLE_STATUS_SUFFIX.to_string()
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_for_dollar_variables_with_exclaim_dollar() {
        assert!(check_for_dollar_variables("{!$MyVar}", &None));
        assert!(check_for_dollar_variables("text {!$MyVar} more text", &None));
    }

    #[test]
    fn test_check_for_dollar_variables_with_dollar_exclaim() {
        assert!(check_for_dollar_variables("{$!MyVar}", &None));
    }

    #[test]
    fn test_check_for_dollar_variables_with_dollar_only() {
        assert!(check_for_dollar_variables("{$MyVar}", &None));
    }

    #[test]
    fn test_check_for_dollar_variables_without_variables() {
        assert!(!check_for_dollar_variables("plain text", &None));
        assert!(!check_for_dollar_variables("no variables here", &None));
    }

    #[test]
    fn test_convert_variables_exclaim_dollar() {
        let result = convert_variables_in_text(Some("{!$MyVar}"), &None);
        assert_eq!(result, "{!@variables.MyVar}");
    }

    #[test]
    fn test_convert_variables_dollar_exclaim() {
        let result = convert_variables_in_text(Some("{$!MyVar}"), &None);
        assert_eq!(result, "{!@variables.MyVar}");
    }

    #[test]
    fn test_convert_variables_dollar_only() {
        let result = convert_variables_in_text(Some("{$MyVar}"), &None);
        assert_eq!(result, "{!@variables.MyVar}");
    }

    #[test]
    fn test_convert_variables_exclaim_only() {
        let result = convert_variables_in_text(Some("{!MyVar}"), &None);
        assert_eq!(result, "{!@variables.MyVar}");
    }

    #[test]
    fn test_convert_variables_none() {
        let result = convert_variables_in_text(None, &None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_convert_variables_no_variables() {
        let result = convert_variables_in_text(Some("plain text"), &None);
        assert_eq!(result, "plain text");
    }

    #[test]
    fn test_convert_variables_mixed_text() {
        let result = convert_variables_in_text(Some("Hello {!$Name}, welcome!"), &None);
        assert_eq!(result, "Hello {!@variables.Name}, welcome!");
    }

    #[test]
    fn test_get_variable_alert_message_default() {
        let msg = get_variable_alert_message(&None);
        assert_eq!(msg, DEFAULT_VARIABLE_ALERT_MESSAGE);
    }

    #[test]
    fn test_get_variable_status_suffix_default() {
        let suffix = get_variable_status_suffix(&None);
        assert_eq!(suffix, DEFAULT_VARIABLE_STATUS_SUFFIX);
    }
}
