# NGA Converter - Claude Skills

This folder contains Claude Skills files that enable Claude (or other AI assistants) to convert Salesforce Agentforce JSON exports to NGA (Next Generation Agent Script) YAML format.

## Skills Overview

| File | Purpose |
|------|---------|
| `NGA_CONVERTER_SKILL.md` | Main conversion rules, field mappings, and complete transformation logic |
| `NGA_INPUT_OUTPUT_SCHEMA.md` | Detailed JSON schema for input/output formats with TypeScript definitions |
| `NGA_EXAMPLES_QUICKREF.md` | Practical examples, quick reference, and common pitfalls |

## How to Use

### Option 1: Add to Claude Projects (Recommended)

1. Open Claude.ai and navigate to **Projects**
2. Create a new project or select an existing one
3. Go to **Project Knowledge**
4. Upload all three `.md` files from this folder
5. Start a conversation and ask Claude to convert your JSON

**Example prompt:**
```
Convert this Salesforce Agentforce JSON to NGA YAML format:

{paste your JSON here}
```

### Option 2: Copy/Paste Skills

1. Copy the content from any skill file
2. Paste it at the beginning of your conversation with Claude
3. Then provide your JSON for conversion

### Option 3: Use with Claude API

Include the skill content in your system prompt when using the Claude API:

```python
import anthropic

client = anthropic.Anthropic()

# Read skill files
with open("NGA_CONVERTER_SKILL.md") as f:
    converter_skill = f.read()

message = client.messages.create(
    model="claude-sonnet-4-20250514",
    max_tokens=8192,
    system=converter_skill,
    messages=[
        {"role": "user", "content": "Convert this JSON to NGA YAML:\n\n{your_json}"}
    ]
)
```

## Skill Files Description

### NGA_CONVERTER_SKILL.md

The main skill file containing:
- Input format detection logic
- Output YAML structure
- Core conversion rules:
  - Variable pattern conversion
  - Type mappings
  - Action target formatting
  - Input/output filtering
  - Source field rules
  - complex_data_type_name mapping
- Default topic templates
- Security rules
- Field mapping tables
- Validation checklist

### NGA_INPUT_OUTPUT_SCHEMA.md

Technical reference containing:
- TypeScript interface definitions for input format
- TypeScript interface definitions for output format
- Complete field mapping tables
- Lightning type reference
- YAML formatting rules
- Validation rules
- Error handling defaults

### NGA_EXAMPLES_QUICKREF.md

Practical guidance containing:
- Quick reference card for common conversions
- 8 detailed conversion examples
- Common pitfalls and solutions
- Step-by-step conversion checklist

## Conversion Quick Reference

### Variable Patterns

| Input | Output |
|-------|--------|
| `{!$Name}` | `{!@variables.Name}` |
| `{$!Name}` | `{!@variables.Name}` |
| `{$Name}` | `{!@variables.Name}` |
| `{!Name}` | `{!@variables.Name}` |

### Type Mappings

| JSON Type | NGA Type |
|-----------|----------|
| `string` | `string` |
| `number` | `number` |
| `integer` | `number` |
| `boolean` | `boolean` |
| `object` | `object` |
| `array` | `list[{itemType}]` |

### Action Target Format

```
{invocation_type}://{target_name}
```

Examples:
- `flow://GetCaseFlow`
- `apex://CaseService`
- `standardInvocableAction://streamKnowledgeSearch`
- `generatePromptResponse://0hfKc000000050IIAQ`

## Key Rules Summary

1. **Variable Extraction**: Only include variables that are actually referenced in text
2. **Object Types**: Always use `mutable` category (never `linked`)
3. **Input Filtering**: Exclude inputs where `copilotAction:isUserInput` is `false`
4. **Source Filtering**: Exclude Salesforce IDs (15/18 alphanumeric chars)
5. **Boolean Format**: Use `True` and `False` (capitalized)
6. **Parameter Names**: Always quote in YAML
7. **Default Topics**: Always include `escalation`, `off_topic`, `ambiguous_question`

## License

These skill files are provided under the same license as the parent NGA Interpreter project (MIT/Apache 2.0 dual license).

## Contributing

To improve these skills:
1. Test conversions with various Salesforce Agentforce exports
2. Document any edge cases or new patterns
3. Update the relevant skill file with corrections or additions

## Related Resources

- [Salesforce Agent Script Documentation](https://developer.salesforce.com/docs/ai/agentforce/guide/agent-script.html)
- Main project README in parent directory
- WASM module documentation in `/WASM/README.md`
