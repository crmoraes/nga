# Agentforce AgentScript Converter - User Guide

Welcome! This tool helps you convert your Salesforce Agentforce agent configurations into the **AgentScript format** - a readable, YAML-based format that makes it easy to understand and modify your AI agents.

> **Reference Documentation**: [Agent Script | Agentforce Developer Guide](https://developer.salesforce.com/docs/ai/agentforce/guide/agent-script.html)

---

## What Is This Tool?

The Agentforce AgentScript Converter transforms your agent definitions from JSON or YAML format into the AgentScript format used by Agentforce Builder. Think of it as a translator that converts your agent configuration into a more readable and manageable format.

### Why Use It?

- **Convert existing agents** to the new script-based format
- **Review agent logic** in a clear, readable YAML format
- **Understand agent structure** and how topics flow together
- **Migrate agents** between environments
- **Edit and modify** agent behavior more easily

---

## How to Use the Converter

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
- The converter will automatically detect your input format
- The converted output will appear in the right panel

### Step 3: Export Your Result

After conversion, you can:

- **Copy** - Click the copy button to copy the output to your clipboard
- **Download** - Click the download button to save as `converted_agent.yaml`
- **View Conversion Report** - Click the Conversion Report button (📄) to see a detailed report of what was converted

> **Note:** The Conversion Report button only appears after a successful conversion.

---

## What Input Formats Are Supported?

The converter recognizes three types of input formats:

### 1. Salesforce Agentforce Export (Most Common)

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

### 2. Simple Topics Format

A simplified format that's easier to read and edit manually.

**What it looks like:**
```yaml
name: "My Agent"
label: "My Custom Agent"
description: "Agent description"
welcome_message: "Hello! How can I help?"
locale: "en_US"

topics:
  - name: greeting
    label: "Greeting"
    description: "Handle greetings"
    instructions: "Welcome the user warmly."
    actions:
      - name: go_to_support
        target: support
```

### 3. Generic Format

Basic key-value pairs. The converter will create a default agent structure from this.

---

## What Gets Converted?

The converter transforms your agent configuration into several sections:

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
- **Actions** → All functions converted to actions

### Variables

Automatically extracted from your functions:
- **Input variables** → Variables the agent can use
- **Output variables** → Variables from action results

### Language Settings

- **Default locale** → Primary language (e.g., "en_US")
- **Additional locales** → Supported languages

---

## Understanding the Output

The converted output follows the AgentScript format, which is organized into clear sections:

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
- **Reasoning** - Instructions for the agent
- **Actions** - Available actions the agent can take

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
    query: mutable string
        description: "A string created by generative AI for search."

language:
    default_locale: "en_US"
    additional_locales: "es, es_MX"

start_agent general_web_search:
    label: "General Web Search"
    description: "Enables the agent to perform real-time web searches..."
    
    reasoning:
        instructions: ->
            | Your job is limited to accessing publicly available information.
            | If the customer's question is too vague, ask for clarification.
        actions:
            AnswerQuestionsWithKnowledge: @action.standard://streamKnowledgeSearch
            go_to_escalation: @utils.transition to @topic.escalation
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

If the converter found variables in your instructions that needed conversion:
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

## Special Features

### Automatic Variable Conversion

Variables in your input are automatically converted to the `@variables` format used by AgentScript:

- `{!$Glossary}` → `{!@variables.Glossary}`
- `{$!RecordId}` → `{!@variables.RecordId}`
- `{$ContactEmail}` → `{!@variables.ContactEmail}`
- `{!Glossary}` → `{!@variables.Glossary}`

You'll see a notification when this happens.

### Automatic Topic Generation

The converter automatically adds essential topics if they're missing from your input:

- **Escalation topic** - For transferring to human agents
- **Off-topic topic** - For handling unrelated questions
- **Ambiguous question topic** - For unclear requests

### Topic Transitions

The converter automatically creates transitions between all topics, allowing your agent to move between different conversation topics smoothly.

---

## Tips for Best Results

1. **Use complete exports** - Export your full agent configuration from Salesforce for best results
2. **Check your input** - Make sure your JSON/YAML is valid before converting
3. **Review the output** - Always review the converted output to ensure everything looks correct
4. **Check the conversion report** - Use the Conversion Report to verify what was converted and identify any items needing attention
5. **Save your work** - Download the output file to keep a copy of your converted agent

---

## Common Questions

### What if my file is too large?

The converter accepts files up to 5MB. If your file is larger, consider:
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

Nothing! The converter only reads your file - it never modifies or deletes your original data. Your input remains unchanged.

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
