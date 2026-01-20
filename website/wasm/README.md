# NGA Converter - WebAssembly Module

This directory contains the Rust implementation of the core conversion logic, compiled to WebAssembly for better code protection and performance.

## Prerequisites

Before building, ensure you have the following installed:

1. **Rust toolchain** (rustc, cargo)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **wasm-pack** - Tool for building Rust to WebAssembly
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

3. **Node.js** (required by wasm-pack)
   - Install from [nodejs.org](https://nodejs.org/)

## Building

### Development Build

```bash
cd WASM
wasm-pack build --target no-modules --dev
```

This creates optimized but debuggable WASM files in `pkg/` directory.

### Production Build

```bash
cd WASM
wasm-pack build --target no-modules --release
```

This creates optimized, minified WASM files for production.

**Note:** We use `--target no-modules` for easier integration without ES module imports. The WASM module is loaded via a script tag and initialized with `wasm_bindgen()`.

## Deployment

After building, copy the generated files to the website directory:

```bash
# From project root
cp -r WASM/pkg/* website/wasm/
```

Or update the import path in `converter.js` to point to `../WASM/pkg/nga_converter.js`

## File Structure

```
WASM/
├── Cargo.toml              # Rust project configuration
├── src/
│   ├── lib.rs              # WASM entry point and exports
│   ├── models.rs           # Data structures
│   ├── converter.rs        # Core conversion logic
│   ├── yaml_generator.rs   # YAML output generation
│   ├── variable_processor.rs   # Variable conversion
│   ├── report_generator.rs # Conversion report generation
│   └── helpers.rs          # Utility functions
└── pkg/                    # Generated WASM package (after build)
    ├── nga_converter.js
    ├── nga_converter_bg.wasm
    └── ...
```

## Integration

The WASM module is loaded via a script tag in `index.html` and initialized in `converter.js`:

```javascript
// Initialize WASM (no-modules format)
await wasm_bindgen();

// Then use WASM for conversion
const result = wasm_bindgen.convert_agent(inputJson, rulesJson);
```

## Exported Functions

The WASM module exports the following functions:

### `convert_agent(input_json, rules_json)`

Main conversion function that transforms input JSON to NGA YAML.

**Arguments:**
- `input_json` - JSON string of the input agent configuration
- `rules_json` - JSON string of conversion rules (can be empty string)

**Returns:** JSON object with:
- `yaml` - The converted YAML string
- `has_variables_with_dollar` - Boolean indicating if variables were converted
- `topic_count` - Number of topics
- `action_count` - Number of actions
- `alert_message` - Alert message for variable conversion (if applicable)
- `status_suffix` - Status suffix for variable conversion (if applicable)

### `generate_report_data(input_json, output_yaml, metadata_json)`

Generates structured conversion report data (IP protected analysis).

**Arguments:**
- `input_json` - JSON string of the input agent configuration
- `output_yaml` - The converted YAML string
- `metadata_json` - JSON string with conversion metadata

**Returns:** JSON object with:
- `agent_info` - Agent name, label, description, and metadata
- `topics` - Array of topic reports with actions
- `variables` - Array of variable reports
- `variables_in_instructions` - Variables detected in instructions requiring review
- `notes` - Analysis notes and warnings

### `check_dollar_variables(input, rules_json)`

Check if input contains variables with $ sign patterns.

**Returns:** Boolean

### `get_alert_message(rules_json)`

Get the variable conversion alert message from rules.

**Returns:** String

### `get_status_suffix(rules_json)`

Get the variable conversion status suffix from rules.

**Returns:** String

### `count_topics(nga_json)` / `count_actions(nga_json)`

Utility functions for counting topics and actions (debugging/testing).

## Report Generation Module

The `report_generator.rs` module provides comprehensive conversion report functionality.

### Features

| Feature | Description |
|---------|-------------|
| **Agent Info Extraction** | Extracts name, label, description, planner details, locale |
| **Topics & Actions Analysis** | Documents all topics and their actions with targets |
| **Variable Tracking** | Lists all variables with types, sources, and descriptions |
| **Variable Detection** | Identifies variables in instructions that were converted |
| **Missing Description Warnings** | Flags topics, actions, and variables without descriptions |
| **Flow Action Review** | Detects flow actions with alphanumeric target names |

### Flow Action Alphanumeric Target Detection

The report generator automatically detects flow actions where `invocationTargetType` equals `flow` and the `invocationTargetName` appears to be a Salesforce record ID rather than a human-readable flow API name.

**Detection Criteria:**
- Target name contains both letters and numbers
- No underscores or spaces (typical in flow API names)
- Starts with a number OR has 3+ consecutive digits
- Examples: `3A7x00000004CqWEAU`, `001xx000003DGbYAAW`

**Report Output:**
When detected, the report includes:
- Warning header with count of flagged actions
- For each flagged action: **Topic** → **Action** → **Target**
- Instruction to verify and replace with correct flow API names

**Example:**
```
- ⚠️ **REVIEW REQUIRED:** 1 flow action(s) have alphanumeric target names...
  - **Topic:** `customer_support` → **Action:** `GetCustomerData` → **Target:** `3A7x00000004CqWEAU`
  - Please verify these flow references and replace with the correct flow API names if needed.
```

### Analysis Functions

| Function | Description |
|----------|-------------|
| `analyze_topics_missing_descriptions` | Find topics without descriptions |
| `analyze_topics_without_actions` | Find topics with no actions |
| `analyze_actions_missing_descriptions` | Count actions without descriptions |
| `analyze_variables_missing_descriptions` | Find variables without descriptions |
| `analyze_flow_actions_with_alphanumeric_targets` | Find flow actions needing review |

---

## Troubleshooting

### Build Errors

1. **Missing dependencies**: Run `cargo build` first to download dependencies
2. **wasm-pack not found**: Ensure wasm-pack is in your PATH
3. **Rust version**: Requires Rust 1.70+ (edition 2021)

### Runtime Errors

1. **WASM not loading**: Check browser console for import errors
2. **CORS issues**: Ensure WASM files are served from same origin or CORS is configured
3. **Module not found**: Verify `pkg/` files are copied to `website/wasm/`

### Performance

- Development build: ~500KB-1MB
- Production build: ~200-500KB (with optimization)
- Use `wasm-opt` for further optimization (see below)

## Optimization

### Using wasm-opt (optional)

After building, further optimize with wasm-opt:

```bash
# Install wasm-opt (part of binaryen package)
# On Ubuntu/Debian: sudo apt-get install binaryen
# On macOS: brew install binaryen

# Optimize the WASM file
wasm-opt -Os pkg/nga_converter_bg.wasm -o pkg/nga_converter_bg.wasm
```

### Build Optimization Flags

The `Cargo.toml` already includes optimization settings:
- `opt-level = "z"` - Optimize for size
- `lto = true` - Link Time Optimization
- `codegen-units = 1` - Better optimization
- `strip = true` - Strip debug symbols

## Testing

To test the WASM module:

1. Build the module: `wasm-pack build --target no-modules --dev`
2. Copy to website: `cp -r pkg/* ../website/wasm/`
3. Open website in browser
4. Check browser console for "WASM module loaded successfully"
5. Try converting a sample file

## Security Notes

- WASM binary is harder to reverse engineer than JavaScript
- Still analyzable with effort, but provides significant protection
- Consider additional obfuscation of remaining JavaScript UI code
- For maximum protection, consider backend API approach

## Maintenance

### Updating Conversion Logic

1. Edit Rust source files in `src/`
2. Rebuild: `wasm-pack build --target no-modules --release`
3. Copy updated files to website directory
4. Test thoroughly

### Adding New Features

1. Add Rust functions in appropriate module
2. Export in `lib.rs` using `#[wasm_bindgen]`
3. Update JavaScript to call new WASM functions
4. Rebuild and deploy

## Support

For issues:
- Check browser console for errors
- Verify WASM files are accessible
- Ensure all dependencies are installed
- Review build output for warnings
