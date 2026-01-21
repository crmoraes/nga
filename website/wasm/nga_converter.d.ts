declare namespace wasm_bindgen {
    /* tslint:disable */
    /* eslint-disable */

    /**
     * Check if input contains variables with $ sign
     */
    export function check_dollar_variables(input: string, rules_json: string): boolean;

    /**
     * Main conversion function - converts input JSON to NGA YAML
     *
     * # Arguments
     * * `input_json` - JSON string of the input agent configuration
     * * `rules_json` - Optional JSON string of conversion rules (can be empty string)
     *
     * # Returns
     * JSON object with:
     * - `yaml`: The converted YAML string
     * - `has_variables_with_dollar`: Boolean indicating if variables were converted
     * - `topic_count`: Number of topics
     * - `action_count`: Number of actions
     */
    export function convert_agent(input_json: string, rules_json: string): any;

    /**
     * Count actions in NGA output (for testing/debugging)
     */
    export function count_actions(nga_json: string): number;

    /**
     * Count topics in NGA output (for testing/debugging)
     */
    export function count_topics(nga_json: string): number;

    /**
     * Generate conversion report data (IP protected)
     *
     * # Arguments
     * * `input_json` - JSON string of the input agent configuration
     * * `output_yaml` - The converted YAML string
     * * `metadata_json` - JSON string with conversion metadata
     *
     * # Returns
     * JSON object with structured report data (not markdown)
     */
    export function generate_report_data(input_json: string, output_yaml: string, metadata_json: string): any;

    /**
     * Get variable alert message
     */
    export function get_alert_message(rules_json: string): string;

    /**
     * Get variable status suffix
     */
    export function get_status_suffix(rules_json: string): string;

    /**
     * Initialize WASM module with panic hook for better error messages
     */
    export function init(): void;

}
declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly check_dollar_variables: (a: number, b: number, c: number, d: number) => number;
    readonly convert_agent: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly count_actions: (a: number, b: number, c: number) => void;
    readonly count_topics: (a: number, b: number, c: number) => void;
    readonly generate_report_data: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly get_alert_message: (a: number, b: number, c: number) => void;
    readonly get_status_suffix: (a: number, b: number, c: number) => void;
    readonly init: () => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_start: () => void;
}

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
declare function wasm_bindgen (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
