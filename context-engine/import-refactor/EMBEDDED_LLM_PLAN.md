# Model Management for Import-Refactor Tool

This document outlines how to add embedded LLM capabilities to your import-refactor tool.

## Quick Start

```bash
# Download a recommended model
import-refactor --download-model codellama-7b

# Use embedded model for analysis
import-refactor --analyze --ai --ai-provider embedded --ai-model codellama-7b

# List available models
import-refactor --list-models

# Check system compatibility
import-refactor --check-system
```

## Implementation Plan

### Phase 1: Add Model Management Commands

Update `main.rs` with new CLI options:

```rust
#[derive(Parser, Debug)]
struct Args {
    // ... existing fields ...
    
    /// Download a model for embedded use
    #[arg(long = "download-model")]
    download_model: Option<String>,
    
    /// List available models for download
    #[arg(long = "list-models")]
    list_models: bool,
    
    /// Check system requirements for running models
    #[arg(long = "check-system")]
    check_system: bool,
    
    /// Use embedded LLM (no external server needed)
    #[arg(long = "embedded")]
    use_embedded: bool,
}
```

### Phase 2: Add Dependencies

```toml
[dependencies]
# Existing dependencies...

# For embedded LLM support
llama-cpp-2 = { version = "0.1", optional = true }
# OR
candle-core = { version = "0.3", optional = true }
candle-transformers = { version = "0.3", optional = true }
hf-hub = { version = "0.3", optional = true }
tokenizers = { version = "0.15", optional = true }

[features]
default = []
embedded-llm = ["llama-cpp-2"]  # or candle dependencies
```

### Phase 3: Model Storage Structure

```
~/.import-refactor/
├── models/
│   ├── codellama-7b.gguf
│   ├── qwen2.5-coder-7b.gguf
│   └── llama3.1-8b.gguf
├── config.toml
└── cache/
```

### Phase 4: Configuration

Create `~/.import-refactor/config.toml`:

```toml
[embedded]
default_model = "codellama-7b"
models_dir = "~/.import-refactor/models"
gpu_layers = 0  # Number of layers to run on GPU
context_size = 4096
threads = 4

[models.codellama-7b]
path = "codellama-7b.gguf"
description = "Code Llama 7B for code analysis"
size_gb = 3.8
min_ram_gb = 8

[models.qwen2.5-coder-7b]
path = "qwen2.5-coder-7b.gguf" 
description = "Qwen 2.5 Coder 7B - excellent for Rust"
size_gb = 4.1
min_ram_gb = 8
```

## Advantages of Embedded Models

### ✅ **Benefits**
- **Complete Privacy**: Code never leaves your machine
- **No API Costs**: Free to run after initial download
- **Offline Operation**: Works without internet
- **No Rate Limits**: Analyze as much code as you want
- **Predictable Performance**: No network latency
- **Self-Contained**: No external dependencies

### ⚠️ **Trade-offs**
- **Storage**: Models are 3-20GB each
- **RAM Usage**: Requires 8-32GB RAM depending on model
- **Setup Time**: Initial download and configuration
- **Quality**: Slightly lower than latest cloud models
- **Hardware Requirements**: Better GPU = faster inference

## Recommended Models for Code Analysis

| Model | Size | RAM | Quality | Best For |
|-------|------|-----|---------|----------|
| **CodeLlama-7B** | 3.8GB | 8GB | Good | General code analysis |
| **Qwen2.5-Coder-7B** | 4.1GB | 8GB | Very Good | Rust/systems code |
| **CodeLlama-13B** | 7.3GB | 16GB | Excellent | Complex analysis |
| **Qwen2.5-Coder-14B** | 8.2GB | 16GB | Excellent | Best quality |

## Implementation Priority

1. **High Priority**: llama.cpp bindings (most mature)
2. **Medium Priority**: Candle integration (pure Rust)
3. **Low Priority**: ONNX runtime (broader compatibility)

## Example Usage Scenarios

### Scenario 1: Corporate Environment
```bash
# Download models once per team
import-refactor --download-model codellama-13b

# Use for private code analysis
import-refactor --analyze --ai --ai-provider embedded --ai-model codellama-13b
```

### Scenario 2: Personal Projects
```bash
# Use smaller model for quick analysis
import-refactor --analyze --ai --ai-provider embedded --ai-model codellama-7b

# Unlimited analysis without API costs
import-refactor --analyze --ai --ai-provider embedded --ai-max-functions 100
```

### Scenario 3: CI/CD Integration
```bash
# Automated code quality checks
import-refactor --analyze --ai --ai-provider embedded --quiet > analysis.json
```

## Next Steps

1. Choose implementation approach (llama.cpp vs Candle)
2. Add CLI commands for model management
3. Implement embedded LLM client
4. Add configuration system
5. Create model download/management system
6. Test with real models
7. Document setup and usage

This would make your tool completely self-contained while maintaining high-quality AI analysis capabilities!