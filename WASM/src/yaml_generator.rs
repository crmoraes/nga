use std::collections::HashMap;
use crate::models::*;
use crate::helpers::*;
use crate::variable_processor::*;

/// Generate NGA YAML output string
pub fn generate_nga_yaml(nga: &NGAOutput, rules: &Option<ConversionRules>) -> String {
    let mut output = String::new();
    
    // System section - apply variable conversion
    output.push_str("system:\n");
    let sys_instructions = convert_variables_in_text(Some(&nga.system.instructions), rules);
    output.push_str(&format!("    instructions: \"{}\"\n", escape_yaml_string(&sys_instructions)));
    output.push_str("    messages:\n");
    let welcome_msg = convert_variables_in_text(Some(&nga.system.messages.welcome), rules);
    let error_msg = convert_variables_in_text(Some(&nga.system.messages.error), rules);
    output.push_str(&format!("        welcome: \"{}\"\n", escape_yaml_string(&welcome_msg)));
    output.push_str(&format!("        error: \"{}\"\n", escape_yaml_string(&error_msg)));
    output.push_str("\n");
    
    // Config section
    output.push_str("config:\n");
    output.push_str(&format!("  default_agent_user: \"{}\"\n", nga.config.default_agent_user));
    output.push_str(&format!("  agent_label: \"{}\"\n", nga.config.agent_label));
    output.push_str(&format!("  developer_name: \"{}\"\n", nga.config.developer_name));
    let config_desc = convert_variables_in_text(Some(&nga.config.description), rules);
    output.push_str(&format!("  description: \"{}\"\n", escape_yaml_string(&config_desc)));
    output.push_str("\n");
    
    // Variables section
    if !nga.variables.is_empty() {
        output.push_str("variables:\n");
        let mut var_keys: Vec<_> = nga.variables.keys().collect();
        var_keys.sort();
        for name in var_keys {
            let variable = &nga.variables[name];
            output.push_str(&format!("    {}: {}\n", name, variable.var_type));
            // Only output source for linked type variables with non-action sources
            // (e.g., @MessagingSession.*, @User.* but NOT @action.*)
            if variable.var_type.starts_with("linked") {
                if let Some(source) = &variable.source {
                    if !source.starts_with("@action.") {
                        output.push_str(&format!("        source: {}\n", source));
                    }
                }
            }
            // Output label if present
            if let Some(label) = &variable.label {
                output.push_str(&format!("        label: \"{}\"\n", label));
            }
            let var_desc = convert_variables_in_text(Some(&variable.description), rules);
            output.push_str(&format!("        description: \"{}\"\n", escape_yaml_string(&var_desc)));
        }
    }
    output.push_str("\n");
    
    // Language section
    output.push_str("language:\n");
    output.push_str(&format!("    default_locale: \"{}\"\n", nga.language.default_locale));
    output.push_str(&format!("    additional_locales: \"{}\"\n", nga.language.additional_locales));
    output.push_str(&format!("    all_additional_locales: {}\n", format_boolean_value(nga.language.all_additional_locales)));
    output.push_str("\n");
    
    // Connection section
    let mut conn_keys: Vec<_> = nga.connections.keys().collect();
    conn_keys.sort();
    for key in conn_keys {
        if key.starts_with("connection ") {
            let connection = &nga.connections[key];
            output.push_str(&format!("{}:\n", key));
            output.push_str(&format!("    adaptive_response_allowed: {}\n", format_boolean_value(connection.adaptive_response_allowed)));
            output.push_str("\n");
            break;
        }
    }
    
    // Topics sections
    let mut topic_keys: Vec<_> = nga.topics.keys().collect();
    topic_keys.sort();
    for key in topic_keys {
        if key.starts_with("start_agent ") || key.starts_with("topic ") {
            let topic = &nga.topics[key];
            output.push_str(&format!("{}:\n", key));
            output.push_str(&format!("    label: \"{}\"\n", topic.label));
            output.push_str("\n");
            
            // Apply variable conversion to topic description
            let topic_desc = convert_variables_in_text(Some(&topic.description), rules);
            output.push_str(&format!("    description: \"{}\"\n", escape_yaml_string(&topic_desc)));
            output.push_str("\n");
            
            // Reasoning section
            output.push_str("    reasoning:\n");
            output.push_str(&format_instructions_block(&topic.reasoning.instructions, rules));
            
            // Reasoning actions (action references with 'with' clauses and descriptions)
            if let Some(reasoning_actions) = &topic.reasoning.actions {
                if !reasoning_actions.is_empty() {
                    output.push_str("        actions:\n");
                    let mut action_keys: Vec<_> = reasoning_actions.keys().collect();
                    action_keys.sort();
                    for action_name in action_keys {
                        let action = &reasoning_actions[action_name];
                        output.push_str(&format!("            {}: {}\n", action_name, action.target));
                        // Add 'with' clauses for action parameters
                        if let Some(params) = &action.with_params {
                            for param in params {
                                output.push_str(&format!("                with {} = ...\n", param));
                            }
                        }
                        // Add description if present
                        if let Some(desc) = &action.description {
                            let desc_converted = convert_variables_in_text(Some(desc), rules);
                            output.push_str(&format!("                description: \"{}\"\n", escape_yaml_string(&desc_converted)));
                        }
                    }
                    output.push_str("\n");
                }
            }
            
            // Full Actions section (detailed definitions)
            if let Some(actions) = &topic.actions {
                if !actions.is_empty() {
                    output.push_str("\n    actions:\n");
                    output.push_str(&format_detailed_actions(actions, rules));
                }
            }
            
            output.push_str("\n");
        }
    }
    
    // Trim excess whitespace but ensure trailing newline
    format!("{}\n", output.trim())
}

/// Format instructions block with proper syntax
fn format_instructions_block(instructions: &str, rules: &Option<ConversionRules>) -> String {
    let indicator = get_instruction_indicator(rules);
    let line_prefix = get_instruction_line_prefix(rules);
    
    if instructions.is_empty() {
        return format!("        instructions: {}\n            {} Handle user requests appropriately.\n", indicator, line_prefix);
    }
    
    let converted_instructions = convert_variables_in_text(Some(instructions), rules);
    let lines: Vec<&str> = converted_instructions.lines().collect();
    
    if lines.len() == 1 && lines[0].len() < 100 {
        return format!("        instructions: {}\n            {} {}\n", indicator, line_prefix, lines[0]);
    }
    
    let mut output = format!("        instructions: {}\n", indicator);
    for line in lines {
        output.push_str(&format!("            {} {}\n", line_prefix, line));
    }
    output
}

/// Get instruction indicator from rules
fn get_instruction_indicator(rules: &Option<ConversionRules>) -> String {
    if let Some(rules) = rules {
        if let Some(output_format) = &rules.output_format {
            if let Some(reasoning) = &output_format.reasoning {
                if let Some(instructions_format) = &reasoning.instructions_format {
                    if let Some(indicator) = &instructions_format.indicator {
                        return indicator.clone();
                    }
                }
            }
        }
    }
    "->".to_string()
}

/// Get instruction line prefix from rules
fn get_instruction_line_prefix(rules: &Option<ConversionRules>) -> String {
    if let Some(rules) = rules {
        if let Some(output_format) = &rules.output_format {
            if let Some(reasoning) = &output_format.reasoning {
                if let Some(instructions_format) = &reasoning.instructions_format {
                    if let Some(prefix) = &instructions_format.line_prefix {
                        return prefix.clone();
                    }
                }
            }
        }
    }
    "|".to_string()
}

/// Format detailed actions for output
fn format_detailed_actions(actions: &HashMap<String, Action>, rules: &Option<ConversionRules>) -> String {
    let mut output = String::new();
    let mut action_keys: Vec<_> = actions.keys().collect();
    action_keys.sort();
    
    for action_name in action_keys {
        let action = &actions[action_name];
        output.push_str(&format!("        {}:\n", action_name));
        
        // Description - apply variable conversion
        let desc = convert_variables_in_text(Some(&action.description), rules);
        output.push_str(&format!("            description: \"{}\"\n", escape_yaml_string(&desc)));
        
        // Label
        if let Some(label) = &action.label {
            output.push_str(&format!("            label: \"{}\"\n", label));
        }
        
        // User confirmation
        output.push_str(&format!("            require_user_confirmation: {}\n", format_boolean_value(action.require_user_confirmation)));
        
        // Progress indicator
        output.push_str(&format!("            include_in_progress_indicator: {}\n", format_boolean_value(action.include_in_progress_indicator)));
        
        // Source - only include if it's a readable name (contains underscores), not a Salesforce ID
        if let Some(source) = &action.source {
            if is_readable_source_name(source) {
                output.push_str(&format!("            source: \"{}\"\n", source));
            }
        }
        
        // Target
        output.push_str(&format!("            target: \"{}\"\n", action.target));
        
        // Progress indicator message (optional, after target)
        if let Some(progress_msg) = &action.progress_indicator_message {
            output.push_str(&format!("            progress_indicator_message: \"{}\"\n", escape_yaml_string(progress_msg)));
        }
        
        // Inputs
        if let Some(inputs) = &action.inputs {
            if !inputs.is_empty() {
                output.push_str("                \n");
                output.push_str("            inputs:\n");
                let mut input_keys: Vec<_> = inputs.keys().collect();
                input_keys.sort();
                for input_name in input_keys {
                    let input_def = &inputs[input_name];
                    // Quote input names
                    output.push_str(&format!("                \"{}\": {}\n", input_name, input_def.input_type));
                    
                    // Input properties - order: description, label, is_required, is_user_input, complex_data_type_name
                    if let Some(desc) = &input_def.description {
                        let input_desc = convert_variables_in_text(Some(desc), rules);
                        output.push_str(&format!("                    description: \"{}\"\n", escape_yaml_string(&input_desc)));
                    }
                    if let Some(label) = &input_def.label {
                        output.push_str(&format!("                    label: \"{}\"\n", label));
                    }
                    output.push_str(&format!("                    is_required: {}\n", format_boolean_value(input_def.is_required)));
                    output.push_str(&format!("                    is_user_input: {}\n", format_boolean_value(input_def.is_user_input)));
                    if let Some(complex_type) = &input_def.complex_data_type_name {
                        output.push_str(&format!("                    complex_data_type_name: \"{}\"\n", complex_type));
                    }
                }
            }
        }
        
        // Outputs
        if let Some(outputs) = &action.outputs {
            if !outputs.is_empty() {
                output.push_str("                \n");
                output.push_str("            outputs:\n");
                let mut output_keys: Vec<_> = outputs.keys().collect();
                output_keys.sort();
                for output_name in output_keys {
                    let output_def = &outputs[output_name];
                    // Quote output names
                    output.push_str(&format!("                \"{}\": {}\n", output_name, output_def.output_type));
                    
                    // Output properties - order: description, label, is_displayable, is_used_by_planner, complex_data_type_name
                    if let Some(desc) = &output_def.description {
                        let output_desc = convert_variables_in_text(Some(desc), rules);
                        output.push_str(&format!("                    description: \"{}\"\n", escape_yaml_string(&output_desc)));
                    }
                    if let Some(label) = &output_def.label {
                        output.push_str(&format!("                    label: \"{}\"\n", label));
                    }
                    output.push_str(&format!("                    is_displayable: {}\n", format_boolean_value(output_def.is_displayable)));
                    output.push_str(&format!("                    is_used_by_planner: {}\n", format_boolean_value(output_def.is_used_by_planner)));
                    if let Some(complex_type) = &output_def.complex_data_type_name {
                        output.push_str(&format!("                    complex_data_type_name: \"{}\"\n", complex_type));
                    }
                }
            }
        }
    }
    
    output
}

/// Check if source is a readable name (API name with underscores) vs a Salesforce ID
/// Salesforce IDs are typically 15 or 18 alphanumeric characters without underscores
fn is_readable_source_name(source: &str) -> bool {
    // If it contains underscores, it's likely an API name
    if source.contains('_') {
        return true;
    }
    
    // If it contains spaces, it's a readable name
    if source.contains(' ') {
        return true;
    }
    
    // Check if it looks like a Salesforce ID (15 or 18 alphanumeric chars)
    let len = source.len();
    if (len == 15 || len == 18) && source.chars().all(|c| c.is_alphanumeric()) {
        // Additional check: Salesforce IDs typically have a mix of letters and numbers
        let has_letters = source.chars().any(|c| c.is_alphabetic());
        let has_numbers = source.chars().any(|c| c.is_numeric());
        if has_letters && has_numbers {
            return false; // This looks like a Salesforce ID
        }
    }
    
    // Default: if it's purely alphanumeric and reasonably short, might be an ID
    // Check for pattern: starts with numbers or has consecutive digits (common in SF IDs)
    if source.chars().all(|c| c.is_alphanumeric()) {
        // Check if it has 3+ consecutive digits (common in Salesforce IDs like "172Wt00000HG6ShIAL")
        let mut consecutive_digits = 0;
        for c in source.chars() {
            if c.is_numeric() {
                consecutive_digits += 1;
                if consecutive_digits >= 3 {
                    return false; // Likely a Salesforce ID
                }
            } else {
                consecutive_digits = 0;
            }
        }
    }
    
    true
}
