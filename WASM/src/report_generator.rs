use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::models::*;

// ============================================================================
// REPORT DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    pub agent_info: AgentInfo,
    pub topics: Vec<TopicReport>,
    pub variables: Vec<VariableReport>,
    pub variables_in_instructions: VariablesInInstructions,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub label: String,
    pub description: String,
    pub planner_role: Option<String>,
    pub planner_company: Option<String>,
    pub planner_tone_type: Option<String>,
    pub locale: Option<String>,
    pub secondary_locales: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicReport {
    pub name: String,
    pub label: String,
    pub description: String,
    pub is_start: bool,
    pub actions: Vec<ActionReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionReport {
    pub name: String,
    pub label: String,
    pub description: String,
    pub target: String,
    pub action_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableReport {
    pub name: String,
    pub var_type: String,
    pub source: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariablesInInstructions {
    pub has_variables: bool,
    pub alert_message: String,
    pub variables: Vec<String>,
}

// ============================================================================
// VARIABLE PATTERN DETECTION
// ============================================================================

/// Get variable detection patterns (IP protected)
fn get_variable_patterns() -> Vec<Regex> {
    vec![
        Regex::new(r"\{!@variables\.([^}]+)\}").unwrap(),
        Regex::new(r"\{!\$([^}]+)\}").unwrap(),
        Regex::new(r"\{\$!([^}]+)\}").unwrap(),
        Regex::new(r"\{\$([^!}][^}]*)\}").unwrap(),
        Regex::new(r"\{!([^@}][^}]*)\}").unwrap(),
    ]
}

/// Extract variables from text using patterns
pub fn extract_variables_from_text(text: &str) -> HashSet<String> {
    let patterns = get_variable_patterns();
    let mut found_variables = HashSet::new();
    
    for pattern in patterns {
        for cap in pattern.captures_iter(text) {
            if let Some(var_name) = cap.get(1) {
                found_variables.insert(var_name.as_str().to_string());
            } else if let Some(full_match) = cap.get(0) {
                found_variables.insert(full_match.as_str().to_string());
            }
        }
    }
    
    found_variables
}

// ============================================================================
// ANALYSIS ALGORITHMS
// ============================================================================

/// Check if description is missing or empty
pub fn is_description_missing(description: &str) -> bool {
    description.is_empty() || 
    description == "No description" || 
    description.trim().is_empty()
}

/// Analyze topics for missing descriptions
pub fn analyze_topics_missing_descriptions(topics: &[TopicReport]) -> Vec<String> {
    topics
        .iter()
        .filter(|t| is_description_missing(&t.description))
        .map(|t| t.name.clone())
        .collect()
}

/// Analyze topics without actions
pub fn analyze_topics_without_actions(topics: &[TopicReport]) -> Vec<String> {
    topics
        .iter()
        .filter(|t| t.actions.is_empty())
        .map(|t| t.name.clone())
        .collect()
}

/// Analyze actions missing descriptions
pub fn analyze_actions_missing_descriptions(topics: &[TopicReport]) -> usize {
    topics
        .iter()
        .flat_map(|t| &t.actions)
        .filter(|a| is_description_missing(&a.description))
        .count()
}

/// Analyze variables missing descriptions
pub fn analyze_variables_missing_descriptions(variables: &[VariableReport]) -> Vec<String> {
    variables
        .iter()
        .filter(|v| is_description_missing(&v.description))
        .map(|v| v.name.clone())
        .collect()
}

/// Check if a target name appears to be an alphanumeric ID (like a Salesforce record ID)
/// rather than a human-readable flow API name
fn is_alphanumeric_id(target_name: &str) -> bool {
    // Salesforce record IDs are typically 15 or 18 characters with mixed letters and numbers
    // Pattern: combination of letters and numbers that doesn't look like a standard name
    // e.g., "3A7x00000004CqWEAU" or "001xx000003DGbYAAW"
    
    // Must have both letters and numbers
    let has_letters = target_name.chars().any(|c| c.is_ascii_alphabetic());
    let has_numbers = target_name.chars().any(|c| c.is_ascii_digit());
    
    if !has_letters || !has_numbers {
        return false;
    }
    
    // Standard flow names typically use underscores, spaces, or are PascalCase/camelCase
    // Salesforce IDs don't have underscores or spaces
    let has_underscore = target_name.contains('_');
    let has_space = target_name.contains(' ');
    
    if has_underscore || has_space {
        return false;
    }
    
    // Check if it looks like a Salesforce ID pattern (alphanumeric, often 15-18 chars)
    // but also catch shorter IDs that are clearly not flow names
    let alphanumeric_only = target_name.chars().all(|c| c.is_ascii_alphanumeric());
    
    if !alphanumeric_only {
        return false;
    }
    
    // Heuristic: if it has consecutive numbers (like "00000") or starts with numbers,
    // it's likely an ID rather than a flow name
    let starts_with_number = target_name.chars().next().map_or(false, |c| c.is_ascii_digit());
    let has_consecutive_numbers = target_name.chars()
        .collect::<Vec<_>>()
        .windows(3)
        .any(|w| w.iter().all(|c| c.is_ascii_digit()));
    
    starts_with_number || has_consecutive_numbers
}

/// Represents a flow action with alphanumeric target name that needs review
#[derive(Debug, Clone)]
pub struct FlowActionReview {
    pub topic_name: String,
    pub action_name: String,
    pub target_name: String,
}

/// Analyze flow actions with alphanumeric target names that may need review
pub fn analyze_flow_actions_with_alphanumeric_targets(topics: &[TopicReport]) -> Vec<FlowActionReview> {
    let mut results = Vec::new();
    
    for topic in topics {
        for action in &topic.actions {
            // Check if action type is "flow" (case insensitive)
            let is_flow = action.action_type.to_lowercase() == "flow";
            
            if is_flow && is_alphanumeric_id(&action.target) {
                results.push(FlowActionReview {
                    topic_name: topic.name.clone(),
                    action_name: action.name.clone(),
                    target_name: action.target.clone(),
                });
            }
        }
    }
    
    results
}

// ============================================================================
// REPORT DATA GENERATION
// ============================================================================

/// Generate report data from input and output
pub fn generate_report_data(
    input: &AgentforceInput,
    output_yaml: &str,
    metadata: &ReportMetadata,
) -> Result<ReportData, String> {
    // 1. Extract agent information
    let agent_info = extract_agent_info(input);
    
    // 2. Extract topics and actions
    let topics = extract_topics_from_input(input);
    
    // 3. Extract variables
    let variables = extract_variables_from_output(output_yaml, input);
    
    // 4. Detect variables in instructions
    let variables_in_instructions = detect_variables_in_instructions(
        input,
        output_yaml,
        metadata,
    );
    
    // 5. Generate analysis notes
    let notes = generate_analysis_notes(&topics, &variables, metadata);
    
    Ok(ReportData {
        agent_info,
        topics,
        variables,
        variables_in_instructions,
        notes,
    })
}

/// Extract agent information from input
fn extract_agent_info(input: &AgentforceInput) -> AgentInfo {
    AgentInfo {
        name: input.name.clone().or_else(|| input.label.clone()).unwrap_or_else(|| "Unnamed Agent".to_string()),
        label: input.label.clone().or_else(|| input.name.clone()).unwrap_or_else(|| "Unnamed Agent".to_string()),
        description: input.description.clone().unwrap_or_else(|| "No description provided".to_string()),
        planner_role: input.planner_role.clone(),
        planner_company: input.planner_company.clone(),
        planner_tone_type: input.planner_tone_type.clone(),
        locale: input.locale.clone(),
        secondary_locales: input.secondary_locales.as_ref().map(|locales| locales.join(", ")),
    }
}

/// Extract topics and actions from input
fn extract_topics_from_input(input: &AgentforceInput) -> Vec<TopicReport> {
    let mut topics = Vec::new();
    
    // Extract from plugins (Agentforce format)
    if let Some(plugins) = &input.plugins {
        for (index, plugin) in plugins.iter().enumerate() {
            if plugin.plugin_type.as_deref() == Some("TOPIC") || plugin.plugin_type.is_none() {
                let topic_name = plugin.local_dev_name.clone()
                    .or_else(|| Some(plugin.name.clone()))
                    .unwrap_or_else(|| format!("topic_{}", index + 1));
                
                let topic_label = plugin.label.clone().unwrap_or_else(|| topic_name.clone());
                
                let topic_description = if let (Some(desc), Some(scope)) = (&plugin.description, &plugin.scope) {
                    format!("{}\n\n{}", desc, scope)
                } else {
                    plugin.description.clone()
                        .or_else(|| plugin.scope.clone())
                        .unwrap_or_else(|| "No description".to_string())
                };
                
                let mut actions = Vec::new();
                
                // Extract actions from functions
                if let Some(functions) = &plugin.functions {
                    for func in functions {
                        let action_name = func.local_dev_name.clone()
                            .or_else(|| Some(func.name.clone()))
                            .unwrap_or_else(|| "unnamed_action".to_string());
                        
                        let action_label = func.label.clone().unwrap_or_else(|| action_name.clone());
                        let action_description = func.description.clone().unwrap_or_else(|| "No description".to_string());
                        let action_target = func.invocation_target_name.clone()
                            .or_else(|| func.invocation_target_type.clone())
                            .unwrap_or_else(|| "N/A".to_string());
                        let action_type = func.invocation_target_type.clone().unwrap_or_else(|| "unknown".to_string());
                        
                        actions.push(ActionReport {
                            name: action_name,
                            label: action_label,
                            description: action_description,
                            target: action_target,
                            action_type,
                        });
                    }
                }
                
                // Add escalation action if needed
                if plugin.can_escalate == Some(true) {
                    actions.push(ActionReport {
                        name: "escalate_to_human".to_string(),
                        label: "Escalate to Human".to_string(),
                        description: "Transfer to a live human agent".to_string(),
                        target: "@utils.escalate".to_string(),
                        action_type: "escalation".to_string(),
                    });
                }
                
                topics.push(TopicReport {
                    name: topic_name,
                    label: topic_label,
                    description: topic_description,
                    is_start: index == 0,
                    actions,
                });
            }
        }
    }
    // Extract from topics array (Simple format)
    else if let Some(input_topics) = &input.topics {
        for (index, topic) in input_topics.iter().enumerate() {
            let topic_name = topic.name.clone()
                .or_else(|| topic.id.clone())
                .unwrap_or_else(|| format!("topic_{}", index + 1));
            
            let topic_label = topic.label.clone().unwrap_or_else(|| topic_name.clone());
            let topic_description = topic.description.clone()
                .or_else(|| topic.scope.clone())
                .unwrap_or_else(|| "No description".to_string());
            
            let mut actions = Vec::new();
            
            if let Some(topic_actions) = &topic.actions {
                for action in topic_actions {
                    let action_name = action.name.clone()
                        .or_else(|| action.id.clone())
                        .unwrap_or_else(|| "unnamed_action".to_string());
                    
                    let action_label = action.label.clone()
                        .or_else(|| action.name.clone())
                        .unwrap_or_else(|| "Unnamed Action".to_string());
                    
                    let action_description = action.description.clone().unwrap_or_else(|| "No description".to_string());
                    let action_target = action.target.clone()
                        .or_else(|| action.invocation_target.clone())
                        .unwrap_or_else(|| "N/A".to_string());
                    let action_type = action.action_type.clone().unwrap_or_else(|| "unknown".to_string());
                    
                    actions.push(ActionReport {
                        name: action_name,
                        label: action_label,
                        description: action_description,
                        target: action_target,
                        action_type,
                    });
                }
            }
            
            topics.push(TopicReport {
                name: topic_name,
                label: topic_label,
                description: topic_description,
                is_start: topic.is_start == Some(true) || index == 0,
                actions,
            });
        }
    }
    
    topics
}

/// Extract variables from output YAML and input
fn extract_variables_from_output(output_yaml: &str, input: &AgentforceInput) -> Vec<VariableReport> {
    let mut variables = Vec::new();
    
    // Parse output YAML to extract variables
    if let Ok(output_parsed) = serde_yaml::from_str::<serde_json::Value>(output_yaml) {
        if let Some(vars_obj) = output_parsed.get("variables").and_then(|v| v.as_object()) {
            for (var_name, var_data) in vars_obj {
                let var_type = var_data.get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let source = var_data.get("source")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                let description = var_data.get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No description")
                    .to_string();
                
                variables.push(VariableReport {
                    name: var_name.clone(),
                    var_type,
                    source,
                    description,
                });
            }
        }
    }
    
    // Also extract from input if available
    if let Some(input_vars) = &input.variables {
        for var_input in input_vars {
            let var_name = var_input.name.clone()
                .or_else(|| var_input.id.clone())
                .unwrap_or_else(|| "unnamed_variable".to_string());
            
            // Check if variable already exists
            if !variables.iter().any(|v| v.name == var_name) {
                let var_type = var_input.var_type.clone().unwrap_or_else(|| "unknown".to_string());
                let description = var_input.description.clone().unwrap_or_else(|| "No description".to_string());
                
                variables.push(VariableReport {
                    name: var_name,
                    var_type,
                    source: var_input.source.clone(),
                    description,
                });
            }
        }
    }
    
    variables
}

/// Detect variables in instructions
fn detect_variables_in_instructions(
    input: &AgentforceInput,
    output_yaml: &str,
    metadata: &ReportMetadata,
) -> VariablesInInstructions {
    let has_variables = metadata.has_variables_with_dollar;
    let alert_message = metadata.alert_message.clone().unwrap_or_else(|| {
        "Variables within instructions were converted to @variables format.".to_string()
    });
    
    let mut found_variables = HashSet::new();
    
    if has_variables {
        // Search in input JSON string
        if let Ok(input_str) = serde_json::to_string(input) {
            let vars = extract_variables_from_text(&input_str);
            found_variables.extend(vars);
        }
        
        // Search in output YAML
        let vars = extract_variables_from_text(output_yaml);
        found_variables.extend(vars);
    }
    
    let mut variables_list: Vec<String> = found_variables.into_iter().collect();
    variables_list.sort();
    
    VariablesInInstructions {
        has_variables,
        alert_message,
        variables: variables_list,
    }
}

/// Generate analysis notes
fn generate_analysis_notes(
    topics: &[TopicReport],
    variables: &[VariableReport],
    metadata: &ReportMetadata,
) -> Vec<String> {
    let mut notes = Vec::new();
    
    // Check for missing descriptions in topics
    let topics_without_desc = analyze_topics_missing_descriptions(topics);
    if !topics_without_desc.is_empty() {
        notes.push(format!(
            "- {} topic(s) are missing descriptions: {}",
            topics_without_desc.len(),
            topics_without_desc.join(", ")
        ));
    }
    
    // Check for topics without actions
    let topics_without_actions = analyze_topics_without_actions(topics);
    if !topics_without_actions.is_empty() {
        notes.push(format!(
            "- {} topic(s) have no actions: {}",
            topics_without_actions.len(),
            topics_without_actions.join(", ")
        ));
    }
    
    // Check for actions without descriptions
    let actions_without_desc_count = analyze_actions_missing_descriptions(topics);
    if actions_without_desc_count > 0 {
        notes.push(format!(
            "- {} action(s) are missing descriptions",
            actions_without_desc_count
        ));
    }
    
    // Check for variables without descriptions
    let vars_without_desc = analyze_variables_missing_descriptions(variables);
    if !vars_without_desc.is_empty() {
        notes.push(format!(
            "- {} variable(s) are missing descriptions: {}",
            vars_without_desc.len(),
            vars_without_desc.join(", ")
        ));
    }
    
    // Check for flow actions with alphanumeric target names (likely Salesforce record IDs)
    let flow_actions_to_review = analyze_flow_actions_with_alphanumeric_targets(topics);
    if !flow_actions_to_review.is_empty() {
        notes.push(format!(
            "- ⚠️ **REVIEW REQUIRED:** {} flow action(s) have alphanumeric target names that appear to be Salesforce record IDs rather than flow API names:",
            flow_actions_to_review.len()
        ));
        for action_review in &flow_actions_to_review {
            notes.push(format!(
                "  - **Topic:** `{}` → **Action:** `{}` → **Target:** `{}`",
                action_review.topic_name,
                action_review.action_name,
                action_review.target_name
            ));
        }
        notes.push("  - Please verify these flow references and replace with the correct flow API names if needed.".to_string());
    }
    
    // Conversion metadata notes
    if let Some(status_suffix) = &metadata.status_suffix {
        notes.push(format!("- {}", status_suffix));
    }
    
    notes
}

// ============================================================================
// REPORT METADATA STRUCTURE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub input_format: String,
    pub topic_count: usize,
    pub action_count: usize,
    pub has_variables_with_dollar: bool,
    pub alert_message: Option<String>,
    pub status_suffix: Option<String>,
}
