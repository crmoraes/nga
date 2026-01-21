use once_cell::sync::Lazy;
use regex::Regex;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Default system instructions for the AI agent
pub const DEFAULT_SYSTEM_INSTRUCTIONS: &str = "You are an AI Agent.";

/// Default welcome message for users
pub const DEFAULT_WELCOME_MESSAGE: &str = "Hi, I'm an AI assistant. How can I help you?";

/// Default error message when something goes wrong
pub const DEFAULT_ERROR_MESSAGE: &str = "Sorry, it looks like something has gone wrong.";

/// Default locale for the agent
pub const DEFAULT_LOCALE: &str = "en_US";

/// Default value for all_additional_locales setting
pub const DEFAULT_ALL_ADDITIONAL_LOCALES: bool = false;

/// YAML boolean string for true
pub const YAML_TRUE: &str = "True";

/// YAML boolean string for false  
pub const YAML_FALSE: &str = "False";

// ============================================================================
// STATIC REGEX PATTERNS (compiled once at startup)
// ============================================================================

/// Regex pattern to match markdown tags like #TagName#
static MARKDOWN_TAG_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"#[A-Za-z]+#").expect("Invalid regex pattern for MARKDOWN_TAG_RE")
});

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
    
    // Remove markdown tags like #TagName# using pre-compiled regex
    let cleaned = MARKDOWN_TAG_RE.replace_all(desc, "");
    
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

/// Get default system values (instructions, welcome, error)
pub fn get_default_system_values() -> (String, String, String) {
    (
        DEFAULT_SYSTEM_INSTRUCTIONS.to_string(),
        DEFAULT_WELCOME_MESSAGE.to_string(),
        DEFAULT_ERROR_MESSAGE.to_string(),
    )
}

/// Get default language values (locale, all_additional_locales)
pub fn get_default_language_values() -> (String, bool) {
    (DEFAULT_LOCALE.to_string(), DEFAULT_ALL_ADDITIONAL_LOCALES)
}

/// Format boolean value for YAML output
pub fn format_boolean_value(value: bool) -> String {
    if value { YAML_TRUE } else { YAML_FALSE }.to_string()
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

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_topic_name_basic() {
        assert_eq!(sanitize_topic_name(Some("MyTopic")), "mytopic");
        assert_eq!(sanitize_topic_name(Some("my_topic")), "my_topic");
        assert_eq!(sanitize_topic_name(Some("My Topic")), "my_topic");
    }

    #[test]
    fn test_sanitize_topic_name_special_chars() {
        assert_eq!(sanitize_topic_name(Some("My-Topic")), "my_topic");
        assert_eq!(sanitize_topic_name(Some("My.Topic.Name")), "my_topic_name");
        assert_eq!(sanitize_topic_name(Some("Topic@123")), "topic_123");
    }

    #[test]
    fn test_sanitize_topic_name_edge_cases() {
        assert_eq!(sanitize_topic_name(None), "unnamed");
        assert_eq!(sanitize_topic_name(Some("")), "");
        assert_eq!(sanitize_topic_name(Some("___")), "");
    }

    #[test]
    fn test_sanitize_action_name() {
        assert_eq!(sanitize_action_name(Some("GetData")), "GetData");
        assert_eq!(sanitize_action_name(Some("get_data")), "get_data");
        assert_eq!(sanitize_action_name(None), "action");
    }

    #[test]
    fn test_generate_developer_name() {
        assert_eq!(generate_developer_name("My Agent"), "MY_AGENT");
        assert_eq!(generate_developer_name("test"), "TEST");
        assert_eq!(generate_developer_name("Hello World"), "HELLO_WORLD");
    }

    #[test]
    fn test_generate_developer_name_special_chars() {
        assert_eq!(generate_developer_name("My@Agent#1"), "MYAGENT1");
        assert_eq!(generate_developer_name("Test_Name"), "TEST_NAME");
    }

    #[test]
    fn test_format_label() {
        assert_eq!(format_label("my_topic"), "My Topic");
        assert_eq!(format_label("hello_world"), "Hello World");
        assert_eq!(format_label("test"), "Test");
    }

    #[test]
    fn test_clean_description() {
        assert_eq!(clean_description(Some("Hello World")), "Hello World");
        assert_eq!(clean_description(Some("Hello  World")), "Hello World");
        assert_eq!(clean_description(None), "");
    }

    #[test]
    fn test_clean_description_removes_tags() {
        assert_eq!(clean_description(Some("Hello #Tag# World")), "Hello World");
        assert_eq!(clean_description(Some("#Start# text #End#")), "text");
    }

    #[test]
    fn test_format_locales() {
        let locales = vec!["en_US".to_string(), "es_ES".to_string()];
        assert_eq!(format_locales(Some(&locales)), "en_US, es_ES");
        assert_eq!(format_locales(None), "");
    }

    #[test]
    fn test_escape_yaml_string() {
        assert_eq!(escape_yaml_string("hello"), "hello");
        assert_eq!(escape_yaml_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_yaml_string("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_yaml_string("tab\there"), "tab\\there");
    }

    #[test]
    fn test_get_default_system_values() {
        let (instructions, welcome, error) = get_default_system_values();
        assert_eq!(instructions, DEFAULT_SYSTEM_INSTRUCTIONS);
        assert_eq!(welcome, DEFAULT_WELCOME_MESSAGE);
        assert_eq!(error, DEFAULT_ERROR_MESSAGE);
    }

    #[test]
    fn test_get_default_language_values() {
        let (locale, all_locales) = get_default_language_values();
        assert_eq!(locale, DEFAULT_LOCALE);
        assert_eq!(all_locales, DEFAULT_ALL_ADDITIONAL_LOCALES);
    }

    #[test]
    fn test_format_boolean_value() {
        assert_eq!(format_boolean_value(true), YAML_TRUE);
        assert_eq!(format_boolean_value(false), YAML_FALSE);
    }

    #[test]
    fn test_merge_description_and_scope() {
        assert_eq!(
            merge_description_and_scope(Some("Desc"), Some("Scope"), "fallback"),
            "Desc Scope"
        );
        assert_eq!(
            merge_description_and_scope(Some("Desc"), None, "fallback"),
            "Desc"
        );
        assert_eq!(
            merge_description_and_scope(None, None, "test"),
            "Handles test requests"
        );
    }
}
