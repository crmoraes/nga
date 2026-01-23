use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// INPUT MODELS (from JavaScript/JSON)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentforceInput {
    pub id: Option<String>,
    pub name: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub planner_role: Option<String>,
    pub planner_company: Option<String>,
    pub planner_tone_type: Option<String>,
    pub locale: Option<String>,
    pub secondary_locales: Option<Vec<String>>,
    pub welcome_message: Option<String>,
    pub welcome_message_alt: Option<String>,
    pub user_location: Option<String>,
    pub voice_config: Option<serde_json::Value>,
    pub plugins: Option<Vec<Plugin>>,
    pub topics: Option<Vec<TopicInput>>,
    pub variables: Option<Vec<VariableInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    pub name: String,
    pub local_dev_name: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub scope: Option<String>,
    pub plugin_type: Option<String>,
    pub instruction_definitions: Option<Vec<InstructionDefinition>>,
    pub functions: Option<Vec<Function>>,
    pub can_escalate: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionDefinition {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    pub name: String,
    pub local_dev_name: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub invocation_target_type: Option<String>,
    pub invocation_target_name: Option<String>,
    pub invocation_target_id: Option<String>,
    pub input_type: Option<InputOutputType>,
    pub output_type: Option<InputOutputType>,
    pub require_user_confirmation: Option<bool>,
    pub include_in_progress_indicator: Option<bool>,
    pub progress_indicator_message: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputOutputType {
    pub properties: Option<HashMap<String, Property>>,
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    #[serde(rename = "type")]
    pub prop_type: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub items: Option<Box<Property>>,
    pub const_value: Option<serde_json::Value>,
    // Additional fields from Salesforce exports - use copilotAction: prefixed names
    #[serde(rename = "copilotAction:isUserInput")]
    pub is_user_input: Option<bool>,
    #[serde(rename = "copilotAction:isDisplayable")]
    pub is_displayable: Option<bool>,
    #[serde(rename = "copilotAction:isUsedByPlanner")]
    pub is_used_by_planner: Option<bool>,
    // lightning:type is the source field in Salesforce input format
    // Its value is used to populate complex_data_type_name in the NGA output format
    #[serde(rename = "lightning:type")]
    pub lightning_type: Option<String>,
    #[serde(rename = "$ref")]
    pub ref_type: Option<String>,
    #[serde(rename = "default")]
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInput {
    pub name: Option<String>,
    pub id: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub scope: Option<String>,
    pub instructions: Option<String>,
    pub reasoning: Option<String>,
    pub actions: Option<Vec<ActionInput>>,
    pub is_start: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInput {
    pub name: Option<String>,
    pub id: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub target: Option<String>,
    pub invocation_target: Option<String>,
    pub target_name: Option<String>,
    #[serde(rename = "type")]
    pub action_type: Option<String>,
    pub inputs: Option<HashMap<String, ActionProperty>>,
    pub outputs: Option<HashMap<String, ActionProperty>>,
    pub require_user_confirmation: Option<bool>,
    pub include_in_progress_indicator: Option<bool>,
    pub progress_indicator_message: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionProperty {
    #[serde(rename = "type")]
    pub prop_type: Option<String>,
    pub description: Option<String>,
    pub label: Option<String>,
    pub required: Option<bool>,
    pub default: Option<serde_json::Value>,
    pub is_user_input: Option<bool>,
    pub is_displayable: Option<bool>,
    pub is_used_by_planner: Option<bool>,
    pub complex_type: Option<String>,
    pub complex_data_type_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInput {
    pub name: Option<String>,
    pub id: Option<String>,
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub var_type: Option<String>,
    pub source: Option<String>,
    pub description: Option<String>,
}

// ============================================================================
// OUTPUT MODELS (NGA Format)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NGAOutput {
    pub system: SystemSection,
    pub config: ConfigSection,
    #[serde(flatten)]
    pub topics: HashMap<String, Topic>,
    pub variables: HashMap<String, Variable>,
    pub language: LanguageSection,
    pub knowledge: KnowledgeSection,
    #[serde(flatten)]
    pub connections: HashMap<String, ConnectionSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSection {
    pub instructions: String,
    pub messages: MessagesSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesSection {
    pub welcome: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSection {
    pub default_agent_user: String,
    pub agent_label: String,
    pub developer_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    #[serde(rename = "type")]
    pub var_type: String,
    pub label: Option<String>,
    pub source: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSection {
    pub default_locale: String,
    pub additional_locales: String,
    pub all_additional_locales: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSection {
    pub rag_feature_config_id: String,
    pub citations_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSection {
    pub adaptive_response_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub label: String,
    pub description: String,
    pub reasoning: ReasoningSection,
    pub actions: Option<HashMap<String, Action>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningSection {
    pub instructions: String,
    pub actions: Option<HashMap<String, ReasoningAction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningAction {
    pub target: String,
    pub description: Option<String>,
    pub with_params: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub description: String,
    pub label: Option<String>,
    pub require_user_confirmation: bool,
    pub include_in_progress_indicator: bool,
    pub progress_indicator_message: Option<String>,
    pub source: Option<String>,
    pub target: String,
    pub inputs: Option<HashMap<String, ActionInputDef>>,
    pub outputs: Option<HashMap<String, ActionOutputDef>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInputDef {
    #[serde(rename = "type")]
    pub input_type: String,
    pub const_value: Option<serde_json::Value>,
    pub description: Option<String>,
    pub label: Option<String>,
    pub is_required: bool,
    pub is_user_input: bool,
    pub complex_data_type_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOutputDef {
    #[serde(rename = "type")]
    pub output_type: String,
    pub description: Option<String>,
    pub label: Option<String>,
    pub is_displayable: bool,
    pub is_used_by_planner: bool,
    pub complex_data_type_name: Option<String>,
}

// ============================================================================
// RULES MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRules {
    pub version: Option<String>,
    pub variable_conversion: Option<VariableConversionRules>,
    pub output_format: Option<OutputFormatRules>,
    pub target_format: Option<TargetFormatRules>,
    pub type_mappings: Option<TypeMappings>,
    pub templates: Option<Templates>,
    pub security_rules: Option<SecurityRules>,
    pub connection: Option<ConnectionRules>,
    pub system: Option<SystemRules>,
    pub language: Option<LanguageRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableConversionRules {
    pub enabled: Option<bool>,
    pub patterns: Option<Vec<VariablePattern>>,
    pub alert_message: Option<String>,
    pub status_suffix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariablePattern {
    pub pattern: String,
    pub replacement: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormatRules {
    pub indentation: Option<IndentationRules>,
    pub action_definition: Option<ActionDefinitionRules>,
    pub reasoning: Option<ReasoningFormatRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndentationRules {
    pub base: Option<u32>,
    pub nested: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinitionRules {
    pub boolean_format: Option<BooleanFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanFormat {
    #[serde(rename = "true")]
    pub true_val: Option<String>,
    #[serde(rename = "false")]
    pub false_val: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningFormatRules {
    pub instructions_format: Option<InstructionsFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionsFormat {
    pub indicator: Option<String>,
    pub line_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetFormatRules {
    pub syntax: Option<String>,
    pub mappings: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMappings {
    pub primitive: Option<HashMap<String, String>>,
    pub complex: Option<HashMap<String, String>>,
    #[serde(rename = "default")]
    pub default_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Templates {
    pub topic_selector: Option<TopicSelectorTemplate>,
    pub escalation: Option<EscalationTemplate>,
    pub off_topic: Option<OffTopicTemplate>,
    pub ambiguous_question: Option<AmbiguousQuestionTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSelectorTemplate {
    pub label: Option<String>,
    pub description: Option<String>,
    pub reasoning: Option<TemplateReasoning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateReasoning {
    pub instructions: Option<String>,
    pub actions: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationTemplate {
    pub label: Option<String>,
    pub description: Option<String>,
    pub reasoning: Option<TemplateReasoning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffTopicTemplate {
    pub label: Option<String>,
    pub description: Option<String>,
    pub include_security_rules: Option<bool>,
    pub base_instructions: Option<String>,
    pub reasoning: Option<TemplateReasoning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguousQuestionTemplate {
    pub label: Option<String>,
    pub description: Option<String>,
    pub include_security_rules: Option<bool>,
    pub base_instructions: Option<String>,
    pub reasoning: Option<TemplateReasoning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRules {
    pub default_rules: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRules {
    pub fields: Option<ConnectionFields>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionFields {
    pub adaptive_response_allowed: Option<AdaptiveResponseAllowed>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveResponseAllowed {
    #[serde(rename = "default")]
    pub default_val: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRules {
    pub fields: Option<SystemFields>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemFields {
    pub instructions: Option<SystemInstructionsField>,
    pub messages: Option<SystemMessagesField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInstructionsField {
    #[serde(rename = "default")]
    pub default_val: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessagesField {
    pub fields: Option<SystemMessagesFields>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessagesFields {
    pub welcome: Option<SystemWelcomeField>,
    pub error: Option<SystemErrorField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemWelcomeField {
    #[serde(rename = "default")]
    pub default_val: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemErrorField {
    #[serde(rename = "default")]
    pub default_val: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageRules {
    pub fields: Option<LanguageFields>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageFields {
    pub default_locale: Option<LanguageDefaultLocale>,
    pub all_additional_locales: Option<LanguageAllAdditionalLocales>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDefaultLocale {
    #[serde(rename = "default")]
    pub default_val: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAllAdditionalLocales {
    #[serde(rename = "default")]
    pub default_val: Option<bool>,
}
