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
wasm-pack build --target web --dev
```

This creates optimized but debuggable WASM files in `pkg/` directory.

### Production Build

```bash
cd WASM
wasm-pack build --target web --release
```

This creates optimized, minified WASM files for production.

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
├── Cargo.toml          # Rust project configuration
├── src/
│   ├── lib.rs         # WASM entry point and exports
│   ├── models.rs      # Data structures
│   ├── converter.rs   # Core conversion logic
│   ├── yaml_generator.rs  # YAML output generation
│   ├── variable_processor.rs  # Variable conversion
│   └── helpers.rs     # Utility functions
└── pkg/               # Generated WASM package (after build)
    ├── nga_converter.js
    ├── nga_converter_bg.wasm
    └── ...
```

## Integration

The WASM module is automatically loaded in `converter.js`:

```javascript
// Automatically imports and initializes WASM
await initWasm();

// Then uses WASM for conversion
const result = wasmModule.convert_agent(inputJson, rulesJson);
```

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

1. Build the module: `wasm-pack build --target web --dev`
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
2. Rebuild: `wasm-pack build --target web --release`
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
