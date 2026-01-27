# NGA Converter - Input/Output Schema Reference

## Purpose

This skill document provides the complete JSON schema definitions for both the Salesforce Agentforce input format and the NGA YAML output format. Use this as a reference when performing conversions.

---

## Input Format: Salesforce Agentforce JSON

### Root Object Structure

```typescript
interface AgentforceInput {
  // Identifiers
  id?: string;                    // Salesforce record ID
  name?: string;                  // API name
  label?: string;                 // Display label
  description?: string;           // Agent description (may contain #TagName# markers)
  
  // Planner configuration
  plannerRole?: string;           // Agent persona/role instructions
  plannerCompany?: string;        // Company context
  plannerToneType?: string;       // "CASUAL" | "FORMAL" | "NEUTRAL"
  
  // Localization
  locale?: string;                // Primary locale (e.g., "en_US")
  secondaryLocales?: string[];    // Additional locales
  
  // Messages
  welcomeMessage?: string;        // Greeting message
  welcomeMessageAlt?: string;     // Alternative welcome message
  
  // Context
  userLocation?: string;          // User's geographic location
  
  // Channel
  voiceConfig?: object;           // Present if voice channel
  
  // Topics and Actions
  plugins?: Plugin[];             // Topics (when pluginType="TOPIC")
  
  // Alternative format (NGA-like input)
  topics?: TopicInput[];          // Simple topics array
  variables?: VariableInput[];    // Variables array
}
```

### Plugin Structure (Topics)

```typescript
interface Plugin {
  id?: string;
  name: string;                           // API name
  localDevName?: string;                  // Developer name override
  label?: string;                         // Display label
  description?: string;                   // Topic description
  scope?: string;                         // Topic scope/boundaries
  pluginType?: string;                    // "TOPIC" for topics
  source?: string;                        // Source template reference
  
  instructionDefinitions?: InstructionDefinition[];
  functions?: Function[];                 // Actions for this topic
  canEscalate?: boolean;                  // Can escalate to human
}

interface InstructionDefinition {
  name?: string;
  label?: string;
  description?: string;                   // Instruction text
}
```

### Function Structure (Actions)

```typescript
interface Function {
  name: string;
  localDevName?: string;
  label?: string;
  description?: string;
  
  // Invocation target
  invocationTargetType?: string;          // "flow" | "apex" | "standardInvocableAction" | "generatePromptResponse"
  invocationTargetName?: string;          // Target API name or ID
  invocationTargetId?: string;            // Alternative target ID
  
  // Configuration
  requireUserConfirmation?: boolean;
  includeInProgressIndicator?: boolean;
  progressIndicatorMessage?: string;
  source?: string;                        // Source template (may be Salesforce ID)
  
  // Input/Output schemas
  inputType?: InputOutputType;
  outputType?: InputOutputType;
}
```

### Input/Output Type Structure

```typescript
interface InputOutputType {
  properties?: Record<string, Property>;
  required?: string[];                    // List of required property names
}

interface Property {
  // Basic type info
  type?: string;                          // "string" | "number" | "integer" | "boolean" | "object" | "array"
  title?: string;                         // Display label
  description?: string;
  
  // For array types
  items?: Property;                       // Item type definition
  
  // Values
  const?: any;                            // Constant value
  default?: any;                          // Default value
  
  // Salesforce-specific fields (use these names exactly)
  "copilotAction:isUserInput"?: boolean;  // Whether input comes from user
  "copilotAction:isDisplayable"?: boolean; // Whether output is displayed
  "copilotAction:isUsedByPlanner"?: boolean; // Whether planner uses this
  
  // Lightning type info
  "lightning:type"?: string;              // e.g., "lightning__textType", "lightning__recordInfoType"
  "$ref"?: string;                        // Type reference, e.g., "#/$defs/lightning__recordInfoType"
}
```

### Lightning Types Reference

| Lightning Type | JSON Type | NGA Type |
|----------------|-----------|----------|
| `lightning__textType` | string | string |
| `lightning__numberType` | number | number |
| `lightning__booleanType` | boolean | boolean |
| `lightning__recordInfoType` | object | object |
| `lightning__listType` | array | list[object] |
| `lightning__richTextType` | string | object |
| `lightning__objectType` | object | object |

---

## Output Format: NGA YAML

### Complete Output Structure

```typescript
interface NGAOutput {
  system: SystemSection;
  config: ConfigSection;
  variables: Record<string, Variable>;
  language: LanguageSection;
  knowledge: KnowledgeSection;
  
  // Flattened sections (keys like "connection messaging", "topic name")
  [key: string]: ConnectionSection | Topic;
}
```

### System Section

```typescript
interface SystemSection {
  instructions: string;           // Global agent instructions
  messages: {
    welcome: string;              // Greeting message
    error: string;                // Error message
  };
}
```

**Construction Rules**:
- `instructions`: Combine `plannerRole` + `plannerCompany` + tone instruction
- `messages.welcome`: Use `welcomeMessage` or generate from label
- `messages.error`: Default to "Sorry, it looks like something has gone wrong."

### Config Section

```typescript
interface ConfigSection {
  default_agent_user: string;     // Format: "agentforce_service_agent@{id}.ext"
  agent_label: string;            // From label or name
  developer_name: string;         // Uppercase, underscores, max 80 chars
  description: string;            // Cleaned description (no markdown tags)
}
```

### Variable Section

```typescript
interface Variable {
  type: string;                   // "{category} {type}" e.g., "linked string", "mutable object"
  label?: string;
  source?: string;                // Only for linked types, format: "@Source.Field"
  description: string;
}
```

**Variable Type Format**: `{category} {dataType}`

Where:
- `category`: `linked` | `mutable`
- `dataType`: `string` | `number` | `boolean` | `object` | `list[string]` | `list[object]` | etc.

**Source Field Rules**:
- Only output `source` for `linked` variables
- Do NOT output `source` if it starts with `@action.` (action outputs should be mutable)

### Language Section

```typescript
interface LanguageSection {
  default_locale: string;         // e.g., "en_US"
  additional_locales: string;     // Comma-separated list
  all_additional_locales: boolean;
}
```

### Knowledge Section

```typescript
interface KnowledgeSection {
  rag_feature_config_id: string;  // Empty string if not configured
  citations_enabled: boolean;
}
```

### Connection Section

Key format: `connection {type}` where type is `messaging` or `voice`

```typescript
interface ConnectionSection {
  adaptive_response_allowed: boolean;
}
```

### Topic Section

Key formats:
- `start_agent topic_selector` - Entry point topic
- `topic {name}` - Regular topics

```typescript
interface Topic {
  label: string;
  description: string;
  reasoning: ReasoningSection;
  actions?: Record<string, Action>;  // Full action definitions
}

interface ReasoningSection {
  instructions: string;           // Multi-line with -> and | syntax
  actions?: Record<string, ReasoningAction>;  // Action references
}

interface ReasoningAction {
  target: string;                 // "@actions.Name" or "@utils.transition to @topic.name"
  description?: string;
  with_params?: string[];         // Parameter names for "with X = ..." clauses
}
```

### Action Definition

```typescript
interface Action {
  description: string;
  label?: string;
  require_user_confirmation: boolean;
  include_in_progress_indicator: boolean;
  progress_indicator_message?: string;
  source?: string;                // Only if readable API name
  target: string;                 // Format: "{type}://{name}"
  inputs?: Record<string, ActionInputDef>;
  outputs?: Record<string, ActionOutputDef>;
}

interface ActionInputDef {
  type: string;                   // NGA type
  const_value?: any;              // Constant value
  description?: string;
  label?: string;
  is_required: boolean;
  is_user_input: boolean;
  complex_data_type_name?: string;
}

interface ActionOutputDef {
  type: string;                   // NGA type
  description?: string;
  label?: string;
  is_displayable: boolean;
  is_used_by_planner: boolean;
  complex_data_type_name?: string;
}
```

---

## Field Mapping Tables

### Input → System Mapping

| Input Field | Output Field | Transformation |
|-------------|--------------|----------------|
| `plannerRole` | `system.instructions` | Convert variables |
| `plannerCompany` | `system.instructions` | Append to role |
| `plannerToneType` | `system.instructions` | Map to tone instruction |
| `welcomeMessage` or `welcomeMessageAlt` | `system.messages.welcome` | Use first available |
| (none) | `system.messages.error` | Default message |

### Input → Config Mapping

| Input Field | Output Field | Transformation |
|-------------|--------------|----------------|
| `label` or `name` | `config.agent_label` | First available |
| `name` or `label` | `config.developer_name` | Uppercase, underscores, max 80 |
| `description` | `config.description` | Remove `#TagName#` markers |
| `id` | `config.default_agent_user` | `agentforce_service_agent@{id}.ext` |

### Input → Language Mapping

| Input Field | Output Field | Transformation |
|-------------|--------------|----------------|
| `locale` | `language.default_locale` | Default: "en_US" |
| `secondaryLocales` | `language.additional_locales` | Join with ", " |
| (none) | `language.all_additional_locales` | Default: False |

### Plugin → Topic Mapping

| Plugin Field | Topic Field | Transformation |
|--------------|-------------|----------------|
| `localDevName` or `name` | Topic key | Sanitize: lowercase, underscores |
| `label` | `label` | Use directly or format from name |
| `description` + `scope` | `description` | Merge both |
| `scope` + `instructionDefinitions` | `reasoning.instructions` | Combine all |
| `functions` | `actions` | Convert each function |

### Function → Action Mapping

| Function Field | Action Field | Transformation |
|----------------|--------------|----------------|
| `description` or `label` | `description` | Clean markdown |
| `label` | `label` | Direct |
| `requireUserConfirmation` | `require_user_confirmation` | Default: False |
| `includeInProgressIndicator` | `include_in_progress_indicator` | Default: False |
| `progressIndicatorMessage` | `progress_indicator_message` | Direct |
| `source` | `source` | Only if readable name (has underscores) |
| `invocationTargetType` + `invocationTargetName` | `target` | Format: `{type}://{name}` |

### Property → Input/Output Mapping

| Property Field | Input/Output Field | Transformation |
|----------------|-------------------|----------------|
| `type` | `type` | Map via type rules |
| `title` | `label` | Direct |
| `description` | `description` | Direct |
| `const` or `default` | `const_value` | Direct |
| `copilotAction:isUserInput` | `is_user_input` | Default: true |
| `copilotAction:isDisplayable` | `is_displayable` | Default: false |
| `copilotAction:isUsedByPlanner` | `is_used_by_planner` | Default: true |
| `lightning:type` or `$ref` | `complex_data_type_name` | Extract type name |
| (required array) | `is_required` | Check if name in required[] |

---

## YAML Formatting Specifics

### Indentation Rules

```yaml
# Root sections (0 spaces)
system:
    # 4 spaces
    instructions: "..."
    messages:
        # 8 spaces
        welcome: "..."

# Config uses 2 spaces for some reason in the standard
config:
  default_agent_user: "..."  # 2 spaces

# Variables
variables:
    name: type              # 4 spaces
        source: ...         # 8 spaces
        description: "..."

# Topics
topic name:
    label: "..."            # 4 spaces
    
    reasoning:
        instructions: ->    # 8 spaces
            | line          # 12 spaces for content
        actions:
            name: target    # 12 spaces

    actions:
        ActionName:         # 8 spaces
            description: "..."  # 12 spaces
            inputs:
                "param": type   # 16 spaces
                    desc: "..." # 20 spaces
```

### Boolean Formatting

Always use capitalized `True` and `False`:
```yaml
require_user_confirmation: True
is_required: False
```

### String Quoting

- Always quote string values
- Escape special characters: `\n`, `\t`, `\"`, `\\`

### Multi-line Instructions

Use the arrow indicator with pipe prefix:
```yaml
instructions: ->
    | First line of instructions
    | Second line of instructions
    | Rules:
    |   Rule 1
    |   Rule 2
```

### Parameter Names

Always quote parameter names in inputs/outputs:
```yaml
inputs:
    "caseNumber": string
        description: "The case number"
```

---

## Validation Rules

### Name Validation

**Topic Names**:
- Pattern: `^[a-z][a-z0-9_]*$`
- Must start with letter
- Lowercase only
- No consecutive underscores

**Developer Names**:
- Pattern: `^[A-Z][A-Z0-9_]{0,78}[A-Z0-9]$`
- Max 80 characters
- Uppercase
- Must start with letter
- Cannot end with underscore

**Variable Names**:
- Pattern: `^[A-Za-z][A-Za-z0-9_]{0,78}$`
- Max 80 characters
- Must start with letter
- No consecutive underscores

### Required Sections

These sections must always be present:
1. `system` with `instructions` and `messages`
2. `config` with all required fields
3. `variables` (can be empty)
4. `language`
5. `knowledge`
6. At least one `connection` section
7. `start_agent topic_selector`

### Required Topics

These topics must exist (created if not in input):
1. `escalation`
2. `off_topic`
3. `ambiguous_question`

---

## Error Handling

### Missing Required Fields

If a required field is missing, use these defaults:

| Field | Default Value |
|-------|---------------|
| `system.instructions` | "You are an AI Agent." |
| `system.messages.welcome` | "Hi, I'm an AI assistant. How can I help you?" |
| `system.messages.error` | "Sorry, it looks like something has gone wrong." |
| `config.agent_label` | "Custom Agent" |
| `config.developer_name` | Generated from name |
| `config.description` | "Service Agent" |
| `language.default_locale` | "en_US" |

### Invalid Values

If a value is invalid:
- Empty strings → Use default
- Invalid types → Map to `object`
- Invalid names → Sanitize according to rules
- Salesforce IDs in source → Exclude source field
