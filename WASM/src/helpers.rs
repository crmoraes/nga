use regex::Regex;

/// Sanitize topic name to valid format
pub fn sanitize_topic_name(name: Option<&str>) -> String {
    let name = name.unwrap_or("unnamed");
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    
    cleaned
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
        .trim_start_matches('_')
        .trim_end_matches('_')
        .to_lowercase()
}

/// Sanitize action name to valid format
pub fn sanitize_action_name(name: Option<&str>) -> String {
    let name = name.unwrap_or("action");
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    
    cleaned
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
        .trim_start_matches('_')
        .trim_end_matches('_')
        .to_string()
}

/// Generate developer name from label/name
pub fn generate_developer_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '_')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .to_uppercase()
        .chars()
        .take(80)
        .collect()
}

/// Format label from name (snake_case to Title Case)
pub fn format_label(name: &str) -> String {
    name.replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Clean description text
pub fn clean_description(desc: Option<&str>) -> String {
    let desc = match desc {
        Some(d) => d,
        None => return String::new(),
    };
    
    // Remove markdown tags like #TagName#
    let re = Regex::new(r"#[A-Za-z]+#").unwrap();
    let cleaned = re.replace_all(desc, "");
    
    // Normalize whitespace
    cleaned
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Format locales array to comma-separated string
pub fn format_locales(locales: Option<&Vec<String>>) -> String {
    match locales {
        Some(locs) => locs.join(", "),
        None => String::new(),
    }
}

/// Escape YAML string for output
pub fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Get default system values
pub fn get_default_system_values() -> (String, String, String) {
    (
        "You are an AI Agent.".to_string(),
        "Hi, I'm an AI assistant. How can I help you?".to_string(),
        "Sorry, it looks like something has gone wrong.".to_string(),
    )
}

/// Get default language values
pub fn get_default_language_values() -> (String, bool) {
    ("en_US".to_string(), false)
}

/// Format boolean value for YAML output
pub fn format_boolean_value(value: bool) -> String {
    if value { "True" } else { "False" }.to_string()
}

/// Merge description and scope into single description
pub fn merge_description_and_scope(
    description: Option<&str>,
    scope: Option<&str>,
    fallback_name: &str,
) -> String {
    let mut parts = Vec::new();
    
    if let Some(desc) = description {
        parts.push(clean_description(Some(desc)));
    }
    
    if let Some(scp) = scope {
        parts.push(clean_description(Some(scp)));
    }
    
    if parts.is_empty() {
        format!("Handles {} requests", fallback_name)
    } else {
        parts.join(" ")
    }
}
