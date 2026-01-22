# Agentforce AgentScript Interpreter - User Guide

Welcome! This tool helps you convert your Salesforce Agentforce agent configurations into the **AgentScript format** - a readable, YAML-based format that makes it easy to understand and modify your AI agents.

> **Reference Documentation**: [Agent Script | Agentforce Developer Guide](https://developer.salesforce.com/docs/ai/agentforce/guide/agent-script.html)

---

## What Is This Tool?

The Agentforce AgentScript Interpreter transforms your agent definitions from JSON format into the AgentScript format used by Agentforce Builder. Think of it as a translator that interpret your agent configuration into a more readable and manageable format.

### Why Use It?

- **Interpret existing agents** into the new script-based format
- **Understand agent structure** and how topics flow together

---

## How to Use the Interpreter

### Getting Your Agent Configuration

To export an existing agent's planner configuration from Salesforce:

1. Navigate to your Salesforce org's base URL
2. Append `/support/qa/planner.jsp` to the URL (e.g., `https://your-org.salesforce.com/support/qa/planner.jsp`)
3. Select your agent from the list
4. Copy or download the JSON configuration

> **Tip:** This planner config contains all the information needed for conversion, including topics, instructions, and functions.

### Step 1: Load Your Agent File

You have three ways to provide your agent configuration:

1. **Upload a File** - Click the upload button (↑) in the input panel
2. **Drag & Drop** - Simply drag a `.json`, `.yaml`, or `.yml` file onto the input area
3. **Paste Content** - Copy and paste your agent configuration directly into the text area

**Supported file types:**
- `.json` files (JSON format)
- `.yaml` or `.yml` files (YAML format)

**File size limit:** Maximum 5MB per file

### Step 2: Convert

Once your input is loaded:

- Click the **Convert** button (or press `Ctrl+Enter` on your keyboard)
- **First-time conversion:** A disclaimer will appear that you must accept before proceeding
- Once accepted, the interpreter will automatically detect your input format
- The generated output will appear in the right panel

> **Note:** The disclaimer only needs to be accepted once per session. Subsequent conversions will proceed immediately.

### Step 3: Export Your Result

After conversion, you can:

- **Copy** - Click the copy button to copy the output to your clipboard
- **Download** - Click the download button to save as `converted_agent.yaml`
- **View Conversion Report** - Click the Conversion Report button (📄) to see a detailed report of what was converted

> **Note:** The Conversion Report button only appears after a successful conversion.

---

## ⚠️ What NEEDS Attention!

After using the interpreter output, the following items require **manual action** in the new Agentforce Builder:

| Item | What You Need to Do |
|------|---------------------|
| **Flow Actions** | Custom actions that call Flows will show the **Flow record ID** (not the API name) in the output. You must **manually re-select the Flow** for each action in Agentforce Builder. |
| **Context Filters** | Context Filters are **NOT converted**. Review your original Context Filter logic and implement it using Agentforce Builder's **deterministic features** if needed. |
| **Data Library** | A `knowledge` section is generated with empty defaults. You must **manually configure the `rag_feature_config_id`** in Agentforce Builder if your agent requires a Data Library. |

> **Tip:** Use the Conversion Report to identify all Flow actions that may need attention.

---

## What Input Formats Are Supported?

The interpreter recognizes three types of input formats:

### Salesforce Agentforce Export (Most Common)

This is the standard format exported from Salesforce Agentforce. It contains a `plugins` array with your agent topics and functions.

**What it looks like:**
```json
{
  "name": "My_Service_Agent",
  "label": "My Service Agent",
  "description": "A helpful service agent",
  "plannerRole": "You are a helpful service agent.",
  "plannerCompany": "Your company provides excellent customer service.",
  "plannerToneType": "CASUAL",
  "locale": "en_US",
  "plugins": [
    {
      "name": "General_FAQ",
      "label": "General FAQ",
      "description": "Handles general FAQ questions",
      "scope": "Your job is to answer general questions.",
      "pluginType": "TOPIC",
      "instructionDefinitions": [...],
      "functions": [...]
    }
  ]
}
```

### Generic Format

Basic key-value pairs. The interpreter will create a default agent structure from this.

---

## What Gets Converted?

The interpreter transforms your agent configuration into several sections:

### System Instructions

Your agent's personality and behavior guidelines:
- **plannerRole** → System instructions (what the agent is)
- **plannerCompany** → Company context (company information)
- **plannerToneType** → Tone guidance (CASUAL, FORMAL, or NEUTRAL)
- **Welcome/Error messages** → User-facing messages

### Agent Configuration

Basic agent information:
- **name** → Developer name (sanitized format)
- **label** → Display name (human-readable)
- **description** → Agent description

### Topics

Each topic in your agent becomes a separate section:
- **Topic name** → Cleaned and formatted
- **Description** → Combined from description and scope fields
- **Instructions** → All instruction definitions combined
- **Actions** → All functions converted to actions with detailed inputs/outputs

### Action Inputs & Outputs

The interpreter intelligently handles action inputs and outputs:
- **Input filtering** → Only inputs where `isUserInput` is `true` are included (internal system parameters are filtered out)
- **Source field** → Only included when it's a readable API name (e.g., `SvcCopilotTmpl__GetCaseByCaseNumber`), Salesforce IDs are excluded
- **Complex data types** → Automatically extracted from `lightning:type` or `$ref` fields
- **Object types** → When an output type is `object` or `list[object]`, `complex_data_type_name` defaults to `lightning__recordInfoType`

### Variables

Variables are only included in the output when they are **actually referenced** in your agent definition:
- Variables must be referenced in instructions, descriptions, scopes, or other text fields
- Reference patterns detected: `{!$VarName}`, `{$VarName}`, `{!VarName}`, `@variables.VarName`
- Variables that exist in function definitions but are never used are **not** included in the output

> **Note:** This keeps your converted agent clean by only including variables that are actually needed.

### Language Settings

- **Default locale** → Primary language (e.g., "en_US")
- **Additional locales** → Supported languages

---

## Understanding the Output

The generated output follows the AgentScript format, which is organized into clear sections:

### System Section
Defines the agent's core behavior and instructions.

### Config Section
Contains agent metadata like name, label, and description.

### Variables Section
Lists all variables the agent can use, with their types and sources.

### Language Section
Specifies supported languages and locales.

### Topics Section
Each topic includes:
- **Label** - Display name
- **Description** - What the topic handles
- **Reasoning** - Instructions for the agent with action references
- **Actions** - Detailed action definitions

### Actions Section (within Topics)
Each action includes:
- **description** - What the action does
- **label** - Human-readable name
- **require_user_confirmation** - Whether user confirmation is needed
- **include_in_progress_indicator** - Whether to show progress
- **source** - API name of the source (only for readable names, not Salesforce IDs)
- **target** - Where the action points (e.g., `flow://FlowName`)
- **inputs** - Input parameters (only those marked as user inputs)
  - Quoted parameter names (e.g., `"contactRecord"`)
  - Type, description, label, is_required, is_user_input, complex_data_type_name
- **outputs** - Output parameters
  - Quoted parameter names (e.g., `"caseRecord"`)
  - Type, description, label, is_displayable, is_used_by_planner, complex_data_type_name

### Example Output Structure

```yaml
system:
    instructions: "You are a Service Agent responsible for..."
    messages:
        welcome: "Hi, I'm Agentforce Service Agent. How can I help you today?"
        error: "Sorry, it looks like something has gone wrong."

config:
  agent_label: "Agentforce Service Agent"
  developer_name: "AGENTFORCE_SERVICE_AGENT"
  description: "Deliver personalized customer interactions..."

variables:
    Glossary: linked string
        label: "Glossary"
        description: "Glossary Definitions"

language:
    default_locale: "en_US"
    additional_locales: "es, es_MX"

topic case_management:
    label: "Case Management"
    description: "Handles customer inquiries related to support cases..."
    
    reasoning:
        instructions: ->
            | Your job is to help customers retrieve case information.
            | If the customer is not known, ask for their email address.
        actions:
            GetCaseByCaseNumber: @actions.GetCaseByCaseNumber
                with contactRecord = ...
                with caseNumber = ...

    actions:
        GetCaseByCaseNumber:
            description: "Returns a case associated with a given contact record and case number."
            label: "Get Case By Case Number"
            require_user_confirmation: False
            include_in_progress_indicator: True
            source: "SvcCopilotTmpl__GetCaseByCaseNumber"
            target: "flow://SvcCopilotTmpl__GetCaseByCaseNumber"
                
            inputs:
                "contactRecord": object
                    description: "Stores the contact record of the customer."
                    label: "Contact record"
                    is_required: True
                    is_user_input: True
                    complex_data_type_name: "lightning__recordInfoType"
                "caseNumber": string
                    description: "Stores the case number provided by the customer."
                    label: "Case Number"
                    is_required: True
                    is_user_input: True
                    complex_data_type_name: "lightning__textType"
                
            outputs:
                "caseRecord": object
                    description: "Stores the case record based on the contact record and case number."
                    label: "Case record"
                    is_displayable: True
                    is_used_by_planner: True
                    complex_data_type_name: "lightning__recordInfoType"
```

---

## Conversion Report

After a successful conversion, you can view a comprehensive **Conversion Report** that documents everything that was converted. This report helps you understand what changed, verify the conversion accuracy, and identify any items that may need your attention.

### How to Access the Report

1. **Complete a conversion** - Load your agent file and click Convert
2. **Look for the report button** - After successful conversion, a **Conversion Report** button (📄) appears in the output panel
3. **Click the button** - The report opens in a modal window that you can read and scroll through
4. **Close when done** - Click the X button, press Escape, or click outside the modal to close

### What's in the Report?

The conversion report is organized into five main sections:

#### 1. Agent Information

A summary of your agent's basic details:
- **Name and Label** - How your agent is identified
- **Description** - What your agent does
- **Metadata** - Planner role, company context, tone type, and language settings

This section helps you verify that your agent's core information was correctly converted.

#### 2. Topics and Actions

A detailed breakdown of every topic in your agent:
- **Topic details** - Name, label, and description for each topic
- **Action list** - All actions within each topic, including:
  - Action name and label
  - Target (where the action points - flow, apex, standard, etc.)
  - Action type
  - Description
- **Start topic** - Indicates which topic is the starting point

This section shows you exactly how your topics and actions were converted, making it easy to verify that everything is correct.

#### 3. Variables Converted

A complete list of all variables in your agent:
- **Variable name** - How the variable is identified
- **Variable type** - Whether it's mutable (can change) or linked (from action output)
- **Source** - Where linked variables come from (if applicable)
- **Description** - What the variable is used for

This helps you understand what variables your agent can use and where they come from.

#### 4. Variables in Instructions (Requires Review)

If the interpreter found variables in your instructions that needed conversion:
- **Warning message** - Alerts you that variables were automatically converted
- **List of variables** - Shows all variables that were found and converted
- **Action required** - Reminds you to review these conversions

**Why this matters:** When variables like `{!$VariableName}` are converted to `{!@variables.VariableName}`, you should verify they work correctly in your converted agent.

#### 5. Other Important Notes

Helpful warnings and recommendations:
- **Missing descriptions** - Topics, actions, or variables that don't have descriptions
- **Topics without actions** - Topics that may need actions added
- **Conversion notes** - Important information about the conversion process

These notes help you identify areas that might need attention or improvement.

---

## Important Disclaimer

Before your first conversion in each session, a disclaimer will be displayed that requires your acceptance. This disclaimer outlines important information about the conversion process:

- The conversion is automated and based on best-effort interpretation
- The generated output must be thoroughly reviewed, validated, and tested
- You are responsible for reviewing agent topics, actions, execution logic, and dependencies
- No guarantee of functional equivalence between original and converted agents

**To proceed with conversion:**
1. Read the disclaimer carefully
2. Click **Accept and Start Conversion** to proceed
3. Click **Decline** or press **Escape** to cancel

> **Note:** The disclaimer acceptance is valid for the current browser session. You won't need to accept it again until you refresh the page or close the browser.

---

## Special Features

### Automatic Variable Conversion

Variables in your input are automatically converted to the `@variables` format used by AgentScript:

- `{!$Glossary}` → `{!@variables.Glossary}`
- `{$!RecordId}` → `{!@variables.RecordId}`
- `{$ContactEmail}` → `{!@variables.ContactEmail}`
- `{!Glossary}` → `{!@variables.Glossary}`

You'll see a notification when this happens.

### Automatic Topic Generation

The interpreter automatically adds essential topics if they're missing from your input:

- **Escalation topic** - For transferring to human agents
- **Off-topic topic** - For handling unrelated questions
- **Ambiguous question topic** - For unclear requests

### Knowledge Section (Data Library)

The interpreter automatically generates a `knowledge` section in the output for Data Library configuration:

```yaml
knowledge:
    rag_feature_config_id: ""
    citations_enabled: False
```

- **rag_feature_config_id** - The ID of the RAG (Retrieval Augmented Generation) feature configuration. Leave empty if not using a Data Library.
- **citations_enabled** - Whether to enable citations in responses. Set to `True` if you want the agent to include source citations.

> **Note:** Data Library assignments are **NOT included** from the input. You must **manually configure** the `rag_feature_config_id` in Agentforce Builder if your agent requires a Data Library.

### Topic Transitions

The interpreter automatically creates transitions between all topics, allowing your agent to move between different conversation topics smoothly.

---

## Tips for Best Results

1. **Use complete exports** - Export your full agent configuration from Salesforce for best results
2. **Check your input** - Make sure your JSON/YAML is valid before converting
3. **Review the output** - Always review the converted output to ensure everything looks correct
4. **Check the conversion report** - Use the Conversion Report to verify what was converted and identify any items needing attention
5. **Save your work** - Download the output file to keep a copy of your converted agent
6. **Verify variables** - Only variables actually referenced in your agent are included in the output
7. **Review action inputs** - Some internal system parameters (like `mode`, `retrieverMode`) are automatically filtered out as they're not user inputs

---

## Common Questions

### What if my file is too large?

The interpreter accepts files up to 5MB. If your file is larger, consider:
- Splitting your agent into smaller configurations
- Removing unnecessary data from your export
- Using text input instead (up to 10MB)

### What if the conversion fails?

Check the status bar at the bottom for error messages. Common issues:
- Invalid JSON/YAML format
- Missing required fields
- File type not supported

### Can I edit the output?

Yes! The output is in YAML format, which is human-readable and easy to edit. You can modify it directly in the output panel or after downloading.

### What happens to my original file?

Nothing! The interpreter only reads your file - it never modifies or deletes your original data. Your input remains unchanged.

### Why are some variables missing from my output?

The interpreter only includes variables that are **actually referenced** in your agent's instructions, descriptions, or other text fields. Variables defined in function inputs/outputs but never used in the agent are intentionally excluded to keep your output clean and focused.

### Why are some action inputs missing?

Action inputs that have `isUserInput` set to `false` (like `mode` or `retrieverMode`) are filtered out. These are internal system parameters that don't need to be defined in the AgentScript output. Only user-facing inputs are included.

### Why doesn't my action have a `source` field?

The `source` field is only included when it contains a readable API name (like `SvcCopilotTmpl__GetCaseByCaseNumber`). Salesforce record IDs (like `172Wt00000HG6ShIAL`) are automatically excluded since they're not meaningful in the converted output.

---

## Keyboard Shortcuts

- **Ctrl+Enter** (or Cmd+Enter on Mac) - Convert the input
- **Escape** - Close modals (Learn More, Conversion Report)

---

## Need More Help?

For detailed information about the AgentScript format, visit:
- [Agent Script Documentation](https://developer.salesforce.com/docs/ai/agentforce/guide/agent-script.html)
- [Agentforce Builder Guide](https://developer.salesforce.com/docs/ai/agentforce/guide/agentforce-builder.html)

---

## Status Indicators

The status bar at the bottom shows:
- **✓ Green** - Success (conversion completed)
- **✕ Red** - Error (something went wrong)
- **• Gray** - Info (ready or processing)

Watch the status bar for helpful messages about your conversion!
