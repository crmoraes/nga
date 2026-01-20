use std::collections::HashMap;
use crate::models::*;
use crate::helpers::*;
use crate::variable_processor::*;

/// Detect input format and convert accordingly
pub fn detect_and_convert(input: &AgentforceInput, rules: &Option<ConversionRules>) -> Result<NGAOutput, String> {
    // Check if it's a Salesforce Agentforce export (has plugins array)
    if let Some(plugins) = &input.plugins {
        if !plugins.is_empty() {
            return convert_agentforce_format(input, rules);
        }
    }
    
    // Check if it's already in NGA-like format (has topics array)
    if let Some(topics) = &input.topics {
        if !topics.is_empty() {
            return convert_simple_format(input, rules);
        }
    }
    
    // Fallback: try to convert as generic input
    convert_generic_format(input, rules)
}

/// Convert Salesforce Agentforce JSON format to NGA
pub fn convert_agentforce_format(input: &AgentforceInput, rules: &Option<ConversionRules>) -> Result<NGAOutput, String> {
    let mut nga = NGAOutput {
        system: SystemSection {
            instructions: build_system_instructions(input, rules),
            messages: MessagesSection {
                welcome: extract_welcome_message(input),
                error: "Sorry, it looks like something has gone wrong.".to_string(),
            },
        },
        config: ConfigSection {
            default_agent_user: format!(
                "agentforce_service_agent@{}.ext",
                input.id.as_deref().unwrap_or("example")
            ),
            agent_label: input
                .label
                .as_ref()
                .or(input.name.as_ref())
                .map(|s| s.clone())
                .unwrap_or_else(|| "Agentforce Service Agent".to_string()),
            developer_name: generate_developer_name(
                input
                    .name
                    .as_deref()
                    .or(input.label.as_deref())
                    .unwrap_or("Agent")
            ),
            description: clean_description(input.description.as_deref()),
        },
        variables: extract_variables(input, rules),
        language: LanguageSection {
            default_locale: input
                .locale
                .as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| get_default_language_values().0),
            additional_locales: format_locales(input.secondary_locales.as_ref()),
            all_additional_locales: get_default_language_values().1,
        },
        topics: HashMap::new(),
        connections: HashMap::new(),
    };
    
    // Connection section
    let connection_type = if input.voice_config.is_some() {
        "voice"
    } else {
        "messaging"
    };
    let adaptive_response = rules
        .as_ref()
        .and_then(|r| {
            r.connection
                .as_ref()
                .and_then(|c| c.fields.as_ref())
                .and_then(|f| f.adaptive_response_allowed.as_ref())
                .and_then(|a| a.default_val)
        })
        .unwrap_or(true);
    
    nga.connections.insert(
        format!("connection {}", connection_type),
        ConnectionSection {
            adaptive_response_allowed: adaptive_response,
        },
    );
    
    // Topics section - Convert plugins to topics
    if let Some(plugins) = &input.plugins {
        // First, create the start_agent topic_selector
        let topic_selector = create_topic_selector_from_plugins(plugins, rules)?;
        nga.topics.insert("start_agent topic_selector".to_string(), topic_selector);
        
        // Then convert each plugin as a regular topic
        for plugin in plugins {
            if plugin.plugin_type.as_deref() != Some("TOPIC") {
                continue;
            }
            
            let topic_name = sanitize_topic_name(
                plugin.local_dev_name.as_deref().or(Some(plugin.name.as_str()))
            );
            let topic_key = format!("topic {}", topic_name);
            
            let topic = convert_plugin_to_topic(plugin, plugins, rules)?;
            nga.topics.insert(topic_key, topic);
        }
    }
    
    // Add default topics if missing
    ensure_default_topics(&mut nga, rules)?;
    
    Ok(nga)
}

/// Build comprehensive system instructions from input
fn build_system_instructions(input: &AgentforceInput, rules: &Option<ConversionRules>) -> String {
    let mut parts = Vec::new();
    
    if let Some(role) = &input.planner_role {
        parts.push(convert_variables_in_text(Some(role), rules));
    }
    
    if let Some(company) = &input.planner_company {
        parts.push(convert_variables_in_text(Some(company), rules));
    }
    
    if let Some(tone) = &input.planner_tone_type {
        let tone_map: HashMap<&str, &str> = [
            ("CASUAL", "Maintain a casual and friendly tone."),
            ("FORMAL", "Maintain a formal and professional tone."),
            ("NEUTRAL", "Maintain a neutral and balanced tone."),
        ]
        .iter()
        .cloned()
        .collect();
        
        if let Some(tone_msg) = tone_map.get(tone.as_str()) {
            parts.push(tone_msg.to_string());
        }
    }
    
    if let Some(location) = &input.user_location {
        parts.push(format!("User location: {}.", location));
    }
    
    if parts.is_empty() {
        get_default_system_values().0
    } else {
        parts.join(" ")
    }
}

/// Extract welcome message from input
fn extract_welcome_message(input: &AgentforceInput) -> String {
    if let Some(msg) = &input.welcome_message {
        return msg.clone();
    }
    if let Some(msg) = &input.welcome_message_alt {
        return msg.clone();
    }
    
    let label = input
        .label
        .as_ref()
        .or(input.name.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| "AI Assistant".to_string());
    
    format!("Hi, I'm {}. How can I help you today?", label)
}

/// Extract variables from all plugin functions
fn extract_variables(input: &AgentforceInput, rules: &Option<ConversionRules>) -> HashMap<String, Variable> {
    let mut variables = HashMap::new();
    
    let plugins = match &input.plugins {
        Some(p) => p,
        None => return variables,
    };
    
    for plugin in plugins {
        let functions = match &plugin.functions {
            Some(f) => f,
            None => continue,
        };
        
        for func in functions {
            // Extract input variables
            if let Some(input_type) = &func.input_type {
                if let Some(properties) = &input_type.properties {
                    for (prop_name, prop) in properties {
                        let clean_name = prop_name
                            .replace("Input:", "")
                            .chars()
                            .filter(|c| c.is_alphanumeric() || *c == '_')
                            .collect::<String>();
                        
                        if !clean_name.is_empty() && !variables.contains_key(&clean_name) {
                            let prop_type = map_property_type(
                                prop.prop_type.as_deref(),
                                prop,
                                rules,
                            );
                            variables.insert(
                                clean_name.clone(),
                                Variable {
                                    var_type: format!("mutable {}", prop_type),
                                    label: prop.title.clone(),
                                    source: None,
                                    description: prop
                                        .description
                                        .as_ref()
                                        .or(prop.title.as_ref())
                                        .map(|s| s.clone())
                                        .unwrap_or_else(|| format!("Variable for {}", clean_name)),
                                },
                            );
                        }
                    }
                }
            }
            
            // Extract output variables - object types are mutable, others are linked
            if let Some(output_type) = &func.output_type {
                if let Some(properties) = &output_type.properties {
                    for (prop_name, prop) in properties {
                        let clean_name = prop_name
                            .replace("Output:", "")
                            .chars()
                            .filter(|c| c.is_alphanumeric() || *c == '_')
                            .collect::<String>();
                        
                        if !clean_name.is_empty() && !variables.contains_key(&clean_name) {
                            let prop_type = map_property_type(
                                prop.prop_type.as_deref(),
                                prop,
                                rules,
                            );
                            let func_name = func
                                .local_dev_name
                                .as_ref()
                                .or(Some(&func.name))
                                .map(|s| s.clone())
                                .unwrap_or_else(|| "unknown".to_string());
                            
                            // Object types (including list[object]) should be mutable, not linked
                            let (category, source) = if prop_type == "object" || prop_type.contains("object") {
                                ("mutable".to_string(), None)
                            } else {
                                ("linked".to_string(), Some(format!("@action.{}.{}", func_name, prop_name)))
                            };
                            
                            variables.insert(
                                clean_name.clone(),
                                Variable {
                                    var_type: format!("{} {}", category, prop_type),
                                    label: prop.title.clone(),
                                    source,
                                    description: prop
                                        .description
                                        .as_ref()
                                        .or(prop.title.as_ref())
                                        .map(|s| s.clone())
                                        .unwrap_or_else(|| {
                                            format!(
                                                "Output from {}",
                                                func.label
                                                    .as_ref()
                                                    .or(Some(&func.name))
                                                    .map(|s| s.clone())
                                                    .unwrap_or_else(|| "unknown".to_string())
                                            )
                                        }),
                                },
                            );
                        }
                    }
                }
            }
        }
    }
    
    variables
}

/// Map JSON Schema types to NGA types
fn map_property_type(
    json_type: Option<&str>,
    prop: &Property,
    rules: &Option<ConversionRules>,
) -> String {
    let json_type = json_type.unwrap_or("object");
    
    // Get type mappings from rules
    let primitive_map: HashMap<&str, &str> = rules
        .as_ref()
        .and_then(|r| r.type_mappings.as_ref())
        .and_then(|tm| tm.primitive.as_ref())
        .map(|pm| {
            pm.iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect()
        })
        .unwrap_or_else(|| {
            [
                ("string", "string"),
                ("number", "number"),
                ("integer", "number"),
                ("boolean", "boolean"),
            ]
            .iter()
            .cloned()
            .collect()
        });
    
    let complex_map: HashMap<&str, &str> = rules
        .as_ref()
        .and_then(|r| r.type_mappings.as_ref())
        .and_then(|tm| tm.complex.as_ref())
        .map(|cm| {
            cm.iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect()
        })
        .unwrap_or_else(|| {
            [("object", "object"), ("array", "list[{itemType}]")]
                .iter()
                .cloned()
                .collect()
        });
    
    let default_type = rules
        .as_ref()
        .and_then(|r| r.type_mappings.as_ref())
        .and_then(|tm| tm.default_type.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| "object".to_string());
    
    // Handle array types with item definitions
    if json_type == "array" {
        if let Some(items) = &prop.items {
            let item_type = items.prop_type.as_deref().unwrap_or("object");
            let mapped_item_type = map_property_type(Some(item_type), items, rules);
            let list_format = complex_map
                .get("array")
                .copied()
                .unwrap_or("list[{itemType}]");
            return list_format.replace("{itemType}", &mapped_item_type);
        }
        return "list[object]".to_string();
    }
    
    primitive_map
        .get(json_type)
        .or_else(|| complex_map.get(json_type))
        .map(|s| s.to_string())
        .unwrap_or(default_type)
}

/// Convert a plugin to an NGA topic
pub fn convert_plugin_to_topic(
    plugin: &Plugin,
    _all_plugins: &[Plugin],
    rules: &Option<ConversionRules>,
) -> Result<Topic, String> {
    let instructions = build_topic_instructions(plugin, rules);
    let actions = build_detailed_actions(plugin, rules)?;
    
    // Build reasoning action references from detailed actions
    let reasoning_actions = build_reasoning_action_references(&actions);
    
    let fallback_name = plugin
        .label
        .as_ref()
        .or(Some(&plugin.name))
        .map(|s| s.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    
    let merged_description = merge_description_and_scope(
        plugin.description.as_deref(),
        plugin.scope.as_deref(),
        &fallback_name,
    );
    
    Ok(Topic {
        label: plugin
            .label
            .clone()
            .unwrap_or_else(|| format_label(&plugin.name)),
        description: merged_description,
        reasoning: ReasoningSection {
            instructions,
            actions: if reasoning_actions.is_empty() { None } else { Some(reasoning_actions) },
        },
        actions: Some(actions),
    })
}

/// Build reasoning action references from detailed actions
fn build_reasoning_action_references(
    actions: &HashMap<String, Action>
) -> HashMap<String, ReasoningAction> {
    let mut reasoning_actions = HashMap::new();
    for (action_name, action) in actions {
        let with_params = action.inputs.as_ref()
            .map(|inputs| {
                let mut params: Vec<String> = inputs.keys().cloned().collect();
                params.sort();
                params
            });
        
        reasoning_actions.insert(
            action_name.clone(),
            ReasoningAction {
                target: format!("@actions.{}", action_name),
                description: None,
                with_params,
            },
        );
    }
    reasoning_actions
}

/// Build topic instructions from instructionDefinitions
fn build_topic_instructions(plugin: &Plugin, rules: &Option<ConversionRules>) -> String {
    let mut parts = Vec::new();
    
    // Add scope as initial context
    if let Some(scope) = &plugin.scope {
        parts.push(convert_variables_in_text(Some(scope), rules));
    }
    
    // Add all instruction definitions
    if let Some(definitions) = &plugin.instruction_definitions {
        for instr in definitions {
            if let Some(desc) = &instr.description {
                parts.push(convert_variables_in_text(Some(desc), rules));
            }
        }
    }
    
    // Add security rules for certain topics
    let topic_name = plugin
        .local_dev_name
        .as_ref()
        .or(Some(&plugin.name))
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| String::new());
    
    if topic_name.contains("off_topic")
        || topic_name.contains("offtopic")
        || topic_name.contains("ambiguous")
        || topic_name.contains("general")
    {
        if let Some(rules) = rules {
            if let Some(security_rules) = &rules.security_rules {
                if let Some(default_rules) = &security_rules.default_rules {
                    if !default_rules.is_empty() {
                        parts.push("Rules:".to_string());
                        for rule in default_rules {
                            parts.push(format!("  {}", rule));
                        }
                    }
                }
            }
        }
    }
    
    if parts.is_empty() {
        "Handle user requests appropriately.".to_string()
    } else {
        parts.join("\n")
    }
}

/// Build detailed actions from functions
fn build_detailed_actions(
    plugin: &Plugin,
    rules: &Option<ConversionRules>,
) -> Result<HashMap<String, Action>, String> {
    let mut actions = HashMap::new();
    
    if let Some(functions) = &plugin.functions {
        for func in functions {
            let action_name = sanitize_action_name(
                func.local_dev_name.as_deref().or(Some(func.name.as_str()))
            );
            
            let fallback_desc = func
                .description
                .as_ref()
                .or(func.label.as_ref())
                .map(|s| s.clone())
                .unwrap_or_else(|| action_name.clone());
            
            let mut action = Action {
                description: clean_description(Some(&fallback_desc)),
                label: func.label.clone(),
                require_user_confirmation: func.require_user_confirmation.unwrap_or(false),
                include_in_progress_indicator: func.include_in_progress_indicator.unwrap_or(false),
                progress_indicator_message: func.progress_indicator_message.clone(),
                source: func.source.clone(),
                target: build_detailed_action_target(func, rules),
                inputs: None,
                outputs: None,
            };
            
            // Add inputs if present
            if let Some(input_type) = &func.input_type {
                action.inputs = Some(build_detailed_inputs(input_type, rules));
            }
            
            // Add outputs if present
            if let Some(output_type) = &func.output_type {
                action.outputs = Some(build_detailed_outputs(output_type, rules));
            }
            
            actions.insert(action_name, action);
        }
    }
    
    Ok(actions)
}

/// Build detailed action target
fn build_detailed_action_target(func: &Function, _rules: &Option<ConversionRules>) -> String {
    let target_type = func
        .invocation_target_type
        .as_deref()
        .unwrap_or("action");
            let target_name = func
                .invocation_target_name
                .as_deref()
                .or(func.invocation_target_id.as_deref())
                .or(Some(func.name.as_str()))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());
    
    // Map invocation types
    let type_map: HashMap<&str, &str> = [
        ("flow", "flow"),
        ("apex", "apex"),
        ("standardInvocableAction", "standardInvocableAction"),
        ("generatePromptResponse", "generatePromptResponse"),
    ]
    .iter()
    .cloned()
    .collect();
    
    let mapped_type = type_map.get(target_type).copied().unwrap_or(target_type);
    format!("{}://{}", mapped_type, target_name)
}

/// Build detailed inputs
fn build_detailed_inputs(
    input_type: &InputOutputType,
    rules: &Option<ConversionRules>,
) -> HashMap<String, ActionInputDef> {
    let mut inputs = HashMap::new();
    
    if let Some(properties) = &input_type.properties {
        for (name, prop) in properties {
            let clean_name = name.replace("Input:", "");
            let prop_type = map_property_type(prop.prop_type.as_deref(), prop, rules);
            
            // Determine is_required from the required array
            let is_required = input_type
                .required
                .as_ref()
                .map(|r| r.contains(name))
                .unwrap_or(false);
            
            // Use is_user_input from property if available, otherwise default based on is_required
            let is_user_input = prop.is_user_input.unwrap_or(is_required);
            
            inputs.insert(
                clean_name.clone(),
                ActionInputDef {
                    input_type: prop_type,
                    const_value: prop.const_value.clone().or(prop.default_value.clone()),
                    description: prop.description.clone().or(prop.title.clone()),
                    label: prop.title.clone().or(Some(clean_name)),
                    is_required,
                    is_user_input,
                    complex_data_type_name: prop.complex_data_type_name.clone(),
                },
            );
        }
    }
    
    inputs
}

/// Build detailed outputs
fn build_detailed_outputs(
    output_type: &InputOutputType,
    rules: &Option<ConversionRules>,
) -> HashMap<String, ActionOutputDef> {
    let mut outputs = HashMap::new();
    
    if let Some(properties) = &output_type.properties {
        for (name, prop) in properties {
            let clean_name = name.replace("Output:", "");
            let prop_type = map_property_type(prop.prop_type.as_deref(), prop, rules);
            
            // Use values from property if available, otherwise use defaults
            let is_displayable = prop.is_displayable.unwrap_or(false);
            let is_used_by_planner = prop.is_used_by_planner.unwrap_or(true);
            
            outputs.insert(
                clean_name.clone(),
                ActionOutputDef {
                    output_type: prop_type,
                    description: prop.description.clone().or(prop.title.clone()),
                    label: prop.title.clone().or(Some(clean_name)),
                    is_displayable,
                    is_used_by_planner,
                    complex_data_type_name: prop.complex_data_type_name.clone(),
                },
            );
        }
    }
    
    outputs
}

/// Convert simple format (with topics array) to NGA
pub fn convert_simple_format(
    input: &AgentforceInput,
    rules: &Option<ConversionRules>,
) -> Result<NGAOutput, String> {
    let defaults = get_default_system_values();
    let lang_defaults = get_default_language_values();
    
    let mut nga = NGAOutput {
        system: SystemSection {
            instructions: input
                .description
                .as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| defaults.0.clone()),
            messages: MessagesSection {
                welcome: input
                    .welcome_message
                    .as_ref()
                    .map(|s| s.clone())
                    .unwrap_or_else(|| defaults.1.clone()),
                error: defaults.2.clone(),
            },
        },
        config: ConfigSection {
            default_agent_user: "agentforce_service_agent@example.ext".to_string(),
            agent_label: input
                .label
                .as_ref()
                .or(input.name.as_ref())
                .map(|s| s.clone())
                .unwrap_or_else(|| "Custom Agent".to_string()),
            developer_name: generate_developer_name(
                input
                    .name
                    .as_deref()
                    .or(input.label.as_deref())
                    .unwrap_or("Agent")
            ),
            description: input
                .description
                .as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| "Service Agent".to_string()),
        },
        variables: HashMap::new(),
        language: LanguageSection {
            default_locale: input
                .locale
                .as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| lang_defaults.0.clone()),
            additional_locales: String::new(),
            all_additional_locales: lang_defaults.1,
        },
        topics: HashMap::new(),
        connections: HashMap::new(),
    };
    
    // Variables section
    if let Some(vars) = &input.variables {
        for v in vars {
            let var_name = v.name.as_ref().or(v.id.as_ref()).map(|s| s.clone());
            if let Some(name) = var_name {
                let var_type = v.var_type.as_deref().unwrap_or("string");
                // Object types (including list[object]) should always be mutable, not linked
                let (category, source) = if var_type == "object" || var_type.contains("object") {
                    ("mutable", None)
                } else if v.source.is_some() {
                    ("linked", v.source.clone())
                } else {
                    ("mutable", None)
                };
                nga.variables.insert(
                    name.clone(),
                    Variable {
                        var_type: format!("{} {}", category, var_type),
                        label: v.label.clone(),
                        source,
                        description: v
                            .description
                            .as_ref()
                            .map(|s| s.clone())
                            .unwrap_or_else(|| format!("Variable {}", name)),
                    },
                );
            }
        }
    }
    
    // Connection section
    let adaptive_response = rules
        .as_ref()
        .and_then(|r| {
            r.connection
                .as_ref()
                .and_then(|c| c.fields.as_ref())
                .and_then(|f| f.adaptive_response_allowed.as_ref())
                .and_then(|a| a.default_val)
        })
        .unwrap_or(true);
    
    nga.connections.insert(
        "connection messaging".to_string(),
        ConnectionSection {
            adaptive_response_allowed: adaptive_response,
        },
    );
    
    // Topics section
    if let Some(topics) = &input.topics {
        // Create topic selector
        let topic_selector = create_topic_selector_from_simple_topics(topics, rules)?;
        nga.topics.insert("start_agent topic_selector".to_string(), topic_selector);
        
        // Convert each topic
        for topic in topics {
            let topic_name = sanitize_topic_name(topic.name.as_deref().or(topic.id.as_deref()));
            let topic_key = format!("topic {}", topic_name);
            
            let nga_topic = Topic {
                label: topic
                    .label
                    .as_ref()
                    .map(|s| s.clone())
                    .unwrap_or_else(|| format_label(&topic_name)),
                description: merge_description_and_scope(
                    topic.description.as_deref(),
                    topic.scope.as_deref(),
                    &topic_name,
                ),
                reasoning: ReasoningSection {
                    instructions: topic
                        .instructions
                        .as_ref()
                        .or(topic.reasoning.as_ref())
                        .map(|s| s.clone())
                        .unwrap_or_else(|| "Handle user requests appropriately.".to_string()),
                    actions: None,
                },
                actions: convert_simple_actions_detailed(topic.actions.as_ref(), rules).ok(),
            };
            
            nga.topics.insert(topic_key, nga_topic);
        }
    }
    
    ensure_default_topics(&mut nga, rules)?;
    Ok(nga)
}

/// Convert simple actions to detailed format
fn convert_simple_actions_detailed(
    actions: Option<&Vec<ActionInput>>,
    _rules: &Option<ConversionRules>,
) -> Result<HashMap<String, Action>, String> {
    let mut result = HashMap::new();
    
    if let Some(actions) = actions {
        for action in actions {
            let action_name = action
                .name
                .as_ref()
                .or(action.id.as_ref())
                .map(|s| s.clone())
                .unwrap_or_else(|| "action".to_string());
            
            // Skip transition-type actions
            if action.target.is_some() || action.action_type.as_deref() == Some("transition") {
                continue;
            }
            
            // Skip escalation actions
            if action.action_type.as_deref() == Some("escalate") {
                continue;
            }
            
            let mut nga_action = Action {
                description: action
                    .description
                    .as_ref()
                    .map(|s| s.clone())
                    .unwrap_or_else(|| action_name.clone()),
                label: action.label.clone(),
                require_user_confirmation: action.require_user_confirmation.unwrap_or(false),
                include_in_progress_indicator: action.include_in_progress_indicator.unwrap_or(false),
                progress_indicator_message: action.progress_indicator_message.clone(),
                source: action.source.clone(),
                target: action
                    .invocation_target
                    .as_ref()
                    .or(action.target_name.as_ref())
                    .map(|s| s.clone())
                    .unwrap_or_else(|| format!("action://{}", action_name)),
                inputs: None,
                outputs: None,
            };
            
            // Add inputs if present
            if let Some(inputs) = &action.inputs {
                let mut nga_inputs = HashMap::new();
                for (input_name, input_def) in inputs {
                    nga_inputs.insert(
                        input_name.clone(),
                        ActionInputDef {
                            input_type: input_def.prop_type.as_deref().unwrap_or("string").to_string(),
                            const_value: input_def.default.clone(),
                            description: input_def.description.clone(),
                            label: input_def.label.clone().or(Some(input_name.clone())),
                            is_required: input_def.required.unwrap_or(false),
                            is_user_input: input_def.is_user_input.unwrap_or(true),
                            complex_data_type_name: input_def.complex_type.clone(),
                        },
                    );
                }
                nga_action.inputs = Some(nga_inputs);
            }
            
            // Add outputs if present
            if let Some(outputs) = &action.outputs {
                let mut nga_outputs = HashMap::new();
                for (output_name, output_def) in outputs {
                    nga_outputs.insert(
                        output_name.clone(),
                        ActionOutputDef {
                            output_type: output_def.prop_type.as_deref().unwrap_or("string").to_string(),
                            description: output_def.description.clone(),
                            label: output_def.label.clone().or(Some(output_name.clone())),
                            is_displayable: output_def.is_displayable.unwrap_or(false),
                            is_used_by_planner: output_def.is_used_by_planner.unwrap_or(true),
                            complex_data_type_name: output_def.complex_type.clone(),
                        },
                    );
                }
                nga_action.outputs = Some(nga_outputs);
            }
            
            result.insert(action_name, nga_action);
        }
    }
    
    Ok(result)
}

/// Convert generic/unknown format to NGA
pub fn convert_generic_format(
    input: &AgentforceInput,
    rules: &Option<ConversionRules>,
) -> Result<NGAOutput, String> {
    let defaults = get_default_system_values();
    let lang_defaults = get_default_language_values();
    
    let mut nga = NGAOutput {
        system: SystemSection {
            instructions: input
                .planner_role
                .as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| defaults.0.clone()),
            messages: MessagesSection {
                welcome: defaults.1.clone(),
                error: defaults.2.clone(),
            },
        },
        config: ConfigSection {
            default_agent_user: "agentforce_service_agent@example.ext".to_string(),
            agent_label: input
                .label
                .as_ref()
                .or(input.name.as_ref())
                .map(|s| s.clone())
                .unwrap_or_else(|| "Custom Agent".to_string()),
            developer_name: generate_developer_name(
                input.name.as_deref().unwrap_or("Agent")
            ),
            description: input
                .description
                .as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| "Service Agent".to_string()),
        },
        variables: HashMap::new(),
        language: LanguageSection {
            default_locale: input
                .locale
                .as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| lang_defaults.0.clone()),
            additional_locales: String::new(),
            all_additional_locales: lang_defaults.1,
        },
        topics: HashMap::new(),
        connections: HashMap::new(),
    };
    
    // Connection section
    let adaptive_response = rules
        .as_ref()
        .and_then(|r| {
            r.connection
                .as_ref()
                .and_then(|c| c.fields.as_ref())
                .and_then(|f| f.adaptive_response_allowed.as_ref())
                .and_then(|a| a.default_val)
        })
        .unwrap_or(true);
    
    nga.connections.insert(
        "connection messaging".to_string(),
        ConnectionSection {
            adaptive_response_allowed: adaptive_response,
        },
    );
    
    // Create default topics
    nga.topics.insert(
        "start_agent topic_selector".to_string(),
        create_default_topic_selector(rules)?,
    );
    nga.topics.insert(
        "topic escalation".to_string(),
        create_default_escalation_topic(rules)?,
    );
    nga.topics.insert(
        "topic off_topic".to_string(),
        create_default_off_topic(rules)?,
    );
    nga.topics.insert(
        "topic ambiguous_question".to_string(),
        create_default_ambiguous_topic(rules)?,
    );
    
    Ok(nga)
}

/// Create topic selector from plugins
fn create_topic_selector_from_plugins(
    plugins: &[Plugin],
    rules: &Option<ConversionRules>,
) -> Result<Topic, String> {
    let mut actions = HashMap::new();
    let template = get_topic_selector_template(rules);
    
    // Add transitions to all topics from plugins
    for plugin in plugins {
        if plugin.plugin_type.as_deref() != Some("TOPIC") {
            continue;
        }
        
        let topic_name = sanitize_topic_name(
            plugin.local_dev_name.as_deref().or(Some(plugin.name.as_str()))
        );
        let action_name = format!("go_to_{}", topic_name);
        
        actions.insert(
            action_name,
            ReasoningAction {
                target: format!("@utils.transition to @topic.{}", topic_name),
                description: None,
                with_params: None,
            },
        );
    }
    
    // Add default topic transitions
    let default_transitions = get_default_topic_transitions(rules);
    for (key, value) in default_transitions {
        if !actions.contains_key(&key) {
            actions.insert(key, value);
        }
    }
    
    Ok(Topic {
        label: template.0,
        description: template.1,
        reasoning: ReasoningSection {
            instructions: template.2,
            actions: Some(actions),
        },
        actions: None,
    })
}

/// Create topic selector from simple topics
fn create_topic_selector_from_simple_topics(
    topics: &[TopicInput],
    rules: &Option<ConversionRules>,
) -> Result<Topic, String> {
    let mut actions = HashMap::new();
    let template = get_topic_selector_template(rules);
    
    // Add transitions to all topics
    for topic in topics {
        let topic_name = sanitize_topic_name(topic.name.as_deref().or(topic.id.as_deref()));
        let action_name = format!("go_to_{}", topic_name);
        
        actions.insert(
            action_name,
            ReasoningAction {
                target: format!("@utils.transition to @topic.{}", topic_name),
                description: None,
                with_params: None,
            },
        );
    }
    
    // Add default topic transitions
    let default_transitions = get_default_topic_transitions(rules);
    for (key, value) in default_transitions {
        if !actions.contains_key(&key) {
            actions.insert(key, value);
        }
    }
    
    Ok(Topic {
        label: template.0,
        description: template.1,
        reasoning: ReasoningSection {
            instructions: template.2,
            actions: Some(actions),
        },
        actions: None,
    })
}

/// Get topic selector template from rules
fn get_topic_selector_template(rules: &Option<ConversionRules>) -> (String, String, String) {
    if let Some(rules) = rules {
        if let Some(templates) = &rules.templates {
            if let Some(topic_selector) = &templates.topic_selector {
                return (
                    topic_selector
                        .label
                        .as_ref()
                        .map(|s| s.clone())
                        .unwrap_or_else(|| "Topic Selector".to_string()),
                    topic_selector
                        .description
                        .as_ref()
                        .map(|s| s.clone())
                        .unwrap_or_else(|| {
                            "Welcome the user and determine the appropriate topic based on user input".to_string()
                        }),
                    topic_selector
                        .reasoning
                        .as_ref()
                        .and_then(|r| r.instructions.as_ref())
                        .map(|s| s.clone())
                        .unwrap_or_else(|| {
                            "Select the best tool to call based on conversation history and user's intent.".to_string()
                        }),
                );
            }
        }
    }
    
    (
        "Topic Selector".to_string(),
        "Welcome the user and determine the appropriate topic based on user input".to_string(),
        "Select the best tool to call based on conversation history and user's intent.".to_string(),
    )
}

/// Get default topic transitions
fn get_default_topic_transitions(
    rules: &Option<ConversionRules>,
) -> HashMap<String, ReasoningAction> {
    let mut defaults = HashMap::new();
    
    // Get transitions from template if available
    if let Some(rules) = rules {
        if let Some(templates) = &rules.templates {
            if let Some(topic_selector) = &templates.topic_selector {
                if let Some(reasoning) = &topic_selector.reasoning {
                    if let Some(actions) = &reasoning.actions {
                        for (key, value) in actions {
                            // Handle both string and object values
                            let target = if let Some(obj) = value.as_object() {
                                obj.get("target")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| value.to_string())
                            } else if let Some(str_val) = value.as_str() {
                                str_val.to_string()
                            } else {
                                value.to_string()
                            };
                            
                            defaults.insert(
                                key.clone(),
                                ReasoningAction {
                                    target,
                                    description: None,
                                    with_params: None,
                                },
                            );
                        }
                    }
                }
            }
        }
    }
    
    // Ensure essential defaults exist
    if !defaults.contains_key("go_to_escalation") {
        defaults.insert(
            "go_to_escalation".to_string(),
            ReasoningAction {
                target: "@utils.transition to @topic.escalation".to_string(),
                description: None,
                with_params: None,
            },
        );
    }
    if !defaults.contains_key("go_to_off_topic") {
        defaults.insert(
            "go_to_off_topic".to_string(),
            ReasoningAction {
                target: "@utils.transition to @topic.off_topic".to_string(),
                description: None,
                with_params: None,
            },
        );
    }
    if !defaults.contains_key("go_to_ambiguous_question") {
        defaults.insert(
            "go_to_ambiguous_question".to_string(),
            ReasoningAction {
                target: "@utils.transition to @topic.ambiguous_question".to_string(),
                description: None,
                with_params: None,
            },
        );
    }
    
    defaults
}

/// Create default topic selector
fn create_default_topic_selector(
    rules: &Option<ConversionRules>,
) -> Result<Topic, String> {
    let template = get_topic_selector_template(rules);
    let default_transitions = get_default_topic_transitions(rules);
    
    Ok(Topic {
        label: template.0,
        description: template.1,
        reasoning: ReasoningSection {
            instructions: template.2,
            actions: Some(default_transitions),
        },
        actions: None,
    })
}

/// Ensure default topics exist
fn ensure_default_topics(
    nga: &mut NGAOutput,
    rules: &Option<ConversionRules>,
) -> Result<(), String> {
    if !has_topic_by_name(nga, "escalation") {
        nga.topics.insert(
            "topic escalation".to_string(),
            create_default_escalation_topic(rules)?,
        );
    }
    if !has_topic_by_name(nga, "off_topic") && !has_topic_by_name(nga, "offtopic") {
        nga.topics.insert(
            "topic off_topic".to_string(),
            create_default_off_topic(rules)?,
        );
    }
    if !has_topic_by_name(nga, "ambiguous") {
        nga.topics.insert(
            "topic ambiguous_question".to_string(),
            create_default_ambiguous_topic(rules)?,
        );
    }
    Ok(())
}

/// Check if topic exists by name
fn has_topic_by_name(nga: &NGAOutput, name: &str) -> bool {
    let name_lower = name.to_lowercase();
    nga.topics.keys().any(|key| {
        (key.starts_with("topic ") || key.starts_with("start_agent "))
            && key.to_lowercase().contains(&name_lower)
    })
}

/// Create default escalation topic
fn create_default_escalation_topic(
    rules: &Option<ConversionRules>,
) -> Result<Topic, String> {
    let template = if let Some(rules) = rules {
        rules.templates.as_ref().and_then(|t| t.escalation.as_ref())
    } else {
        None
    };
    
    let default_label = template
        .and_then(|t| t.label.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| "Escalation".to_string());
    
    let default_desc = template
        .and_then(|t| t.description.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| {
            "Handles requests from users who want to transfer or escalate their conversation to a live human agent.".to_string()
        });
    
    let default_instructions = template
        .and_then(|t| t.reasoning.as_ref())
        .and_then(|r| r.instructions.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| {
            "If a user explicitly asks to transfer to a live agent, escalate the conversation.\nIf escalation to a live agent fails for any reason, acknowledge the issue and ask the user whether they would like to log a support case instead.".to_string()
        });
    
    let mut actions = HashMap::new();
    if let Some(template) = template {
        if let Some(reasoning) = &template.reasoning {
            if let Some(template_actions) = &reasoning.actions {
                for (key, value) in template_actions {
                    let target = if let Some(obj) = value.as_object() {
                        obj.get("target")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| value.to_string())
                    } else if let Some(str_val) = value.as_str() {
                        str_val.to_string()
                    } else {
                        value.to_string()
                    };
                    
                    let description = if let Some(obj) = value.as_object() {
                        obj.get("description")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    };
                    
                    actions.insert(
                        key.clone(),
                        ReasoningAction {
                            target,
                            description,
                            with_params: None,
                        },
                    );
                }
            }
        }
    }
    
    // Ensure escalate action exists
    if !actions.contains_key("escalate_to_human") {
        actions.insert(
            "escalate_to_human".to_string(),
            ReasoningAction {
                target: "@utils.escalate".to_string(),
                description: Some("Call this tool to escalate to a human agent.".to_string()),
                with_params: None,
            },
        );
    }
    
    Ok(Topic {
        label: default_label,
        description: default_desc,
        reasoning: ReasoningSection {
            instructions: default_instructions,
            actions: Some(actions),
        },
        actions: None,
    })
}

/// Create default off-topic topic
fn create_default_off_topic(rules: &Option<ConversionRules>) -> Result<Topic, String> {
    let template = if let Some(rules) = rules {
        rules.templates.as_ref().and_then(|t| t.off_topic.as_ref())
    } else {
        None
    };
    
    let default_label = template
        .and_then(|t| t.label.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| "Off Topic".to_string());
    
    let default_desc = template
        .and_then(|t| t.description.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| {
            "Redirect conversation to relevant topics when user request goes off-topic".to_string()
        });
    
    let base_instructions = template
        .and_then(|t| t.base_instructions.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| {
            "Your job is to redirect the conversation to relevant topics politely and succinctly.\nThe user request is off-topic. NEVER answer general knowledge questions. Only respond to general greetings and questions about your capabilities.\nDo not acknowledge the user's off-topic question. Redirect the conversation by asking how you can help with questions related to the pre-defined topics.".to_string()
        });
    
    // Add security rules if template includes them
    let mut instructions = base_instructions;
    let include_security = template
        .and_then(|t| t.include_security_rules)
        .unwrap_or(true);
    
    if include_security {
        if let Some(rules) = rules {
            if let Some(security_rules) = &rules.security_rules {
                if let Some(default_rules) = &security_rules.default_rules {
                    if !default_rules.is_empty() {
                        instructions.push_str("\nRules:\n  ");
                        instructions.push_str(&default_rules.join("\n  "));
                    }
                }
            }
        }
    }
    
    Ok(Topic {
        label: default_label,
        description: default_desc,
        reasoning: ReasoningSection {
            instructions,
            actions: Some(HashMap::new()),
        },
        actions: None,
    })
}

/// Create default ambiguous question topic
fn create_default_ambiguous_topic(
    rules: &Option<ConversionRules>,
) -> Result<Topic, String> {
    let template = if let Some(rules) = rules {
        rules
            .templates
            .as_ref()
            .and_then(|t| t.ambiguous_question.as_ref())
    } else {
        None
    };
    
    let default_label = template
        .and_then(|t| t.label.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| "Ambiguous Question".to_string());
    
    let default_desc = template
        .and_then(|t| t.description.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| {
            "Redirect conversation to relevant topics when user request is too ambiguous".to_string()
        });
    
    let base_instructions = template
        .and_then(|t| t.base_instructions.as_ref())
        .map(|s| s.clone())
        .unwrap_or_else(|| {
            "Your job is to help the user provide clearer, more focused requests for better assistance.\nDo not answer any of the user's ambiguous questions. Do not invoke any actions.\nPolitely guide the user to provide more specific details about their request.\nEncourage them to focus on their most important concern first to ensure you can provide the most helpful response.".to_string()
        });
    
    // Add security rules if template includes them
    let mut instructions = base_instructions;
    let include_security = template
        .and_then(|t| t.include_security_rules)
        .unwrap_or(true);
    
    if include_security {
        if let Some(rules) = rules {
            if let Some(security_rules) = &rules.security_rules {
                if let Some(default_rules) = &security_rules.default_rules {
                    if !default_rules.is_empty() {
                        instructions.push_str("\nRules:\n  ");
                        instructions.push_str(&default_rules.join("\n  "));
                    }
                }
            }
        }
    }
    
    Ok(Topic {
        label: default_label,
        description: default_desc,
        reasoning: ReasoningSection {
            instructions,
            actions: Some(HashMap::new()),
        },
        actions: None,
    })
}
