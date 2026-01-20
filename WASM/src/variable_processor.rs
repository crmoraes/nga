use regex::Regex;
use crate::models::ConversionRules;

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
    
    // Fallback pattern - matches {!$...}, {$!...}, {$...}, or {!...} (not already @variables)
    let var_pattern = Regex::new(r"\{[!]?\$[!]?[^}]+\}|\{![^@}][^}]*\}").unwrap();
    var_pattern.is_match(input)
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
    
    // Fallback: Handle all variable patterns, converting to @variables format
    let mut result = text.to_string();
    
    // {!$Glossary} → {!@variables.Glossary}
    let re1 = Regex::new(r"\{!\$([^}]+)\}").unwrap();
    result = re1.replace_all(&result, "{!@variables.$1}").to_string();
    
    // {$!Glossary} → {!@variables.Glossary}
    let re2 = Regex::new(r"\{\$!([^}]+)\}").unwrap();
    result = re2.replace_all(&result, "{!@variables.$1}").to_string();
    
    // {$Glossary} → {!@variables.Glossary}
    let re3 = Regex::new(r"\{\$([^!}][^}]*)\}").unwrap();
    result = re3.replace_all(&result, "{!@variables.$1}").to_string();
    
    // {!Glossary} → {!@variables.Glossary} (only if not already @variables)
    let re4 = Regex::new(r"\{!([^@}][^}]*)\}").unwrap();
    result = re4.replace_all(&result, "{!@variables.$1}").to_string();
    
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
    "Variables within instructions will be converted to @variables format".to_string()
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
    "(variables converted to @variables format)".to_string()
}
