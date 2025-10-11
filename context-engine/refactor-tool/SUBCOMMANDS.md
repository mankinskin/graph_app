# Refactor Tool - Subcommand Structure

The refactor-tool has been restructured to use subcommands for better organization and future extensibility.

## Available Subcommands

### `imports` - Import Refactoring
Refactor import statements between crates or within a single crate.

```bash
# Cross-crate refactoring
refactor-tool imports source_crate target_crate

# Self-refactoring within a crate
refactor-tool imports --self my_crate

# With additional options
refactor-tool imports --dry-run --verbose source_crate target_crate
```

### `analyze` - Code Duplication Analysis
Analyze the codebase for duplicate and similar functions.

```bash
# Basic analysis
refactor-tool analyze

# AI-powered analysis
refactor-tool analyze --ai

# With specific AI provider
refactor-tool analyze --ai --ai-provider openai
refactor-tool analyze --ai --ai-provider claude --ai-model claude-3-5-sonnet-20241022
```

### `serve` - Start LLM Server
Start the Candle LLM server for hosting models locally.

```bash
# Start server with default config
refactor-tool serve

# Start with specific configuration
refactor-tool serve --config custom-config.toml --host 0.0.0.0 --port 8080
```

### `download-model` - Download AI Models
Download models for the Candle server.

```bash
# Download a specific model
refactor-tool download-model microsoft/DialoGPT-medium
refactor-tool download-model microsoft/CodeBERT-base
```

### `list-models` - List Available Models
List available models and check system compatibility.

```bash
refactor-tool list-models
```

### `init-config` - Initialize Configuration
Generate a default configuration file.

```bash
# Generate default config
refactor-tool init-config

# Generate config at specific path
refactor-tool init-config --config /path/to/config.toml
```

## Global Options

The following options are available for all subcommands:

- `-w, --workspace-root <PATH>` - Workspace root directory (default: .)
- `--dry-run` - Show what would be changed without modifying files
- `-v, --verbose` - Verbose output

## Backward Compatibility

The new subcommand structure maintains all existing functionality while providing better organization:

- **Import refactoring** is now under the `imports` subcommand
- **Code analysis** is now under the `analyze` subcommand  
- **Server operations** are split into dedicated subcommands (`serve`, `download-model`, etc.)

All existing flags and options are preserved within their respective subcommands.