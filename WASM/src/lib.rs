mod models;
mod helpers;
mod variable_processor;
mod converter;
mod yaml_generator;
mod report_generator;

use wasm_bindgen::prelude::*;
use serde_json;
use crate::models::*;
use crate::converter::*;
use crate::yaml_generator::*;
use crate::variable_processor::*;
use crate::report_generator::*;

/// Initialize WASM module
#[wasm_bindgen(start)]
pub fn init() {
    // Set up panic hook for better error messages (if available)
    // Note: console_error_panic_hook is optional
}

/// Main conversion function - converts input JSON to NGA YAML
/// 
/// # Arguments
/// * `input_json` - JSON string of the input agent configuration
/// * `rules_json` - Optional JSON string of conversion rules (can be empty string)
/// 
/// # Returns
/// JSON object with:
/// - `yaml`: The converted YAML string
/// - `has_variables_with_dollar`: Boolean indicating if variables were converted
/// - `topic_count`: Number of topics
/// - `action_count`: Number of actions
#[wasm_bindgen]
pub fn convert_agent(input_json: &str, rules_json: &str) -> Result<JsValue, JsValue> {
    // Parse input JSON
    let input: AgentforceInput = serde_json::from_str(input_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse input JSON: {}", e)))?;
    
    // Parse rules JSON (if provided)
    let rules: Option<ConversionRules> = if rules_json.is_empty() {
        None
    } else {
        match serde_json::from_str(rules_json) {
            Ok(r) => Some(r),
            Err(_) => None, // Use defaults if rules parsing fails
        }
    };
    
    // Check for variables with $ in the input
    let input_str = input_json;
    let has_variables_with_dollar = check_for_dollar_variables(input_str, &rules);
    
    // Detect format and convert
    let nga_output = detect_and_convert(&input, &rules)
        .map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))?;
    
    // Generate YAML
    let yaml_output = generate_nga_yaml(&nga_output, &rules);
    
    // Count topics and actions
    let topic_count = nga_output.topics.len();
    let action_count = nga_output
        .topics
        .values()
        .map(|topic| {
            topic.actions.as_ref().map(|a| a.len()).unwrap_or(0)
        })
        .sum::<usize>();
    
    // Create result object
    let result = serde_json::json!({
        "yaml": yaml_output,
        "has_variables_with_dollar": has_variables_with_dollar,
        "topic_count": topic_count,
        "action_count": action_count,
        "alert_message": if has_variables_with_dollar {
            get_variable_alert_message(&rules)
        } else {
            String::new()
        },
        "status_suffix": if has_variables_with_dollar {
            get_variable_status_suffix(&rules)
        } else {
            String::new()
        }
    });
    
    // Convert to JsValue
    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

/// Check if input contains variables with $ sign
#[wasm_bindgen]
pub fn check_dollar_variables(input: &str, rules_json: &str) -> bool {
    let rules: Option<ConversionRules> = if rules_json.is_empty() {
        None
    } else {
        serde_json::from_str(rules_json).ok()
    };
    
    check_for_dollar_variables(input, &rules)
}

/// Get variable alert message
#[wasm_bindgen]
pub fn get_alert_message(rules_json: &str) -> String {
    let rules: Option<ConversionRules> = if rules_json.is_empty() {
        None
    } else {
        serde_json::from_str(rules_json).ok()
    };
    
    get_variable_alert_message(&rules)
}

/// Get variable status suffix
#[wasm_bindgen]
pub fn get_status_suffix(rules_json: &str) -> String {
    let rules: Option<ConversionRules> = if rules_json.is_empty() {
        None
    } else {
        serde_json::from_str(rules_json).ok()
    };
    
    get_variable_status_suffix(&rules)
}

/// Count topics in NGA output (for testing/debugging)
#[wasm_bindgen]
pub fn count_topics(nga_json: &str) -> Result<usize, JsValue> {
    let nga: NGAOutput = serde_json::from_str(nga_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse NGA JSON: {}", e)))?;
    
    Ok(nga.topics.len())
}

/// Count actions in NGA output (for testing/debugging)
#[wasm_bindgen]
pub fn count_actions(nga_json: &str) -> Result<usize, JsValue> {
    let nga: NGAOutput = serde_json::from_str(nga_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse NGA JSON: {}", e)))?;
    
    let count: usize = nga
        .topics
        .values()
        .map(|topic| topic.actions.as_ref().map(|a| a.len()).unwrap_or(0))
        .sum();
    
    Ok(count)
}

/// Generate conversion report data (IP protected)
/// 
/// # Arguments
/// * `input_json` - JSON string of the input agent configuration
/// * `output_yaml` - The converted YAML string
/// * `metadata_json` - JSON string with conversion metadata
/// 
/// # Returns
/// JSON object with structured report data (not markdown)
#[wasm_bindgen]
pub fn generate_report_data(input_json: &str, output_yaml: &str, metadata_json: &str) -> Result<JsValue, JsValue> {
    // Parse input JSON
    let input: AgentforceInput = serde_json::from_str(input_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse input JSON: {}", e)))?;
    
    // Parse metadata JSON
    let metadata: ReportMetadata = serde_json::from_str(metadata_json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse metadata JSON: {}", e)))?;
    
    // Generate report data (IP protected logic)
    let report_data = report_generator::generate_report_data(&input, output_yaml, &metadata)
        .map_err(|e| JsValue::from_str(&format!("Failed to generate report data: {}", e)))?;
    
    // Convert to JsValue
    serde_wasm_bindgen::to_value(&report_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize report data: {}", e)))
}
