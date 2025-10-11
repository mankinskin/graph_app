# AI-Powered Code Analysis Features

The refactor-tool now includes AI-powered semantic code analysis to enhance duplication detection and provide intelligent refactoring suggestions.

## Features

### 1. Semantic Code Similarity Detection
Beyond simple syntactic matching, the AI analyzes:
- **Functional equivalence**: Functions that solve the same problem using different approaches
- **Algorithmic similarity**: Similar control flow patterns and logic structures  
- **Intent analysis**: Code that serves similar purposes even with different implementations

### 2. Intelligent Refactoring Suggestions
The AI provides contextual recommendations for:
- **Extract utility functions**: Identify common patterns that should be abstracted
- **Parameterization opportunities**: Functions that could be unified with parameters
- **Architectural improvements**: Suggestions for traits, modules, and structural changes

### 3. Supported AI Providers

#### OpenAI/GitHub Copilot
```bash
export OPENAI_API_KEY="your-api-key"
# or
export COPILOT_API_KEY="your-copilot-api-key"

# Optional: specify model
export OPENAI_MODEL="gpt-4"
export OPENAI_BASE_URL="https://api.openai.com/v1"  # Custom endpoint
```

#### Anthropic Claude
```bash
export ANTHROPIC_API_KEY="your-api-key"

# Optional: specify model  
export ANTHROPIC_MODEL="claude-3-5-sonnet-20241022"
export ANTHROPIC_BASE_URL="https://api.anthropic.com/v1"  # Custom endpoint
```

#### Local LLMs (Ollama/llama.cpp)
```bash
# Using Ollama (recommended for local models)
export OLLAMA_BASE_URL="http://localhost:11434"
export OLLAMA_MODEL="codellama:13b"  # or llama3.1, qwen2.5-coder, etc.

# Using llama.cpp server
export LLAMACPP_BASE_URL="http://localhost:8080"
export LLAMACPP_MODEL="codellama-13b-instruct"
```

## Usage

### Basic Analysis (No AI)
```bash
# Traditional duplication analysis
refactor-tool --analyze

# Verbose output with additional insights
refactor-tool --analyze --verbose
```

### AI-Enhanced Analysis
```bash
# Enable AI analysis (auto-detects provider from environment)
refactor-tool --analyze --ai

# Specify AI provider explicitly
refactor-tool --analyze --ai --ai-provider openai
refactor-tool --analyze --ai --ai-provider claude
refactor-tool --analyze --ai --ai-provider ollama

# Limit functions analyzed (to control API costs)
refactor-tool --analyze --ai --ai-max-functions 10

# Specify AI model
refactor-tool --analyze --ai --ai-model gpt-4
refactor-tool --analyze --ai --ai-model claude-3-5-sonnet-20241022
refactor-tool --analyze --ai --ai-model codellama:13b  # for Ollama
```

### Combined with Import Refactoring
```bash
# Standard import refactoring (unchanged)
refactor-tool source_crate target_crate

# Self-refactor with AI analysis
refactor-tool --self my_crate --analyze --ai
```

## Example Output

### Without AI
```
🎯 COMPREHENSIVE DUPLICATION ANALYSIS RESULTS
=====================================

🔄 IDENTICAL FUNCTIONS (2)
-------------------------------
1. Function 'parse_config' - 3 identical copies:
   src/config.rs:45
   src/utils.rs:123  
   src/main.rs:67

💡 BASIC REFACTORING OPPORTUNITIES (1)
-------------------------------------
1. Extract identical function 'parse_config' (3 duplicates) to shared module (Confidence: 95%)
   💾 Estimated lines saved: 45
   📍 Suggested location: src/utils/extracted.rs
```

### With AI Enhancement
```
🎯 COMPREHENSIVE DUPLICATION ANALYSIS RESULTS
=====================================

🤖 AI-POWERED SEMANTIC ANALYSIS
--------------------------------
Overall Confidence: 92.3%
Analysis Reasoning: Identified several functions with similar logical patterns despite different implementations

🧠 SEMANTIC SIMILARITY GROUPS (3)
1. Similarity Score: 85.2% (4 functions)
   Common Patterns: error handling, input validation, configuration parsing
   Key Differences: error types, validation methods, config formats
   src/config.rs:23 (load_config)
   src/settings.rs:45 (read_settings)
   src/env.rs:12 (parse_environment)
   src/cli.rs:89 (validate_args)

🎯 AI REFACTORING SUGGESTIONS (2)
1. Create a unified configuration trait (Confidence: 88.7%)
   Type: extract_trait
   Benefit: Reduces code duplication by ~60 lines and standardizes config handling
   Implementation: Define a ConfigLoader trait with validate() and parse() methods
   Affected Functions: load_config, read_settings, parse_environment

2. Extract common validation logic (Confidence: 91.2%)
   Type: extract_function
   Benefit: Eliminates 4 similar validation implementations
   Implementation: Create generic validate_input<T>() function with custom validators
   Affected Functions: validate_args, validate_config, validate_env, validate_file

📊 SUMMARY
----------
• Total duplicate function instances: 5
• Total refactoring opportunities: 3
• AI-identified opportunities: 2
• Estimated lines that could be saved: 67 (basic) + 75 (AI suggestions)
• Files analyzed: 23
• AI analysis enabled: ✅
```

## Cost Management

The AI analysis includes several cost control features:

- **Function limit**: `--ai-max-functions N` (default: 20)
- **Complexity threshold**: Only analyzes functions above minimum complexity
- **Smart sampling**: Prioritizes functions most likely to benefit from refactoring
- **Batch processing**: Groups similar functions to minimize API calls

## Local LLM Setup (Ollama)

For privacy, unlimited usage, or offline analysis, you can use local models via Ollama:

### Installation
```bash
# Install Ollama (Windows/macOS/Linux)
curl -fsSL https://ollama.ai/install.sh | sh

# Or download from: https://ollama.ai/download
```

### Setup Code Models
```bash
# Install recommended code analysis models
ollama pull codellama:13b          # Best balance of size/performance
ollama pull codellama:34b          # Better quality (requires 32GB+ RAM)
ollama pull qwen2.5-coder:14b      # Alternative code model
ollama pull llama3.1:8b            # Smaller general model

# Check installed models
ollama list
```

### Configuration
```bash
# Configure for local LLM
export OLLAMA_MODEL="codellama:13b"           # Model to use
export OLLAMA_BASE_URL="http://localhost:11434"  # Ollama server (default)

# Start Ollama server (if not auto-started)
ollama serve
```

### Usage Examples
```bash
# Use local LLM for analysis
refactor-tool --analyze --ai --ai-provider ollama

# Specify different model
refactor-tool --analyze --ai --ai-provider ollama --ai-model qwen2.5-coder:14b

# Analyze more functions (no API costs with local models)
refactor-tool --analyze --ai --ai-provider ollama --ai-max-functions 50
```

### Hardware Requirements

#### Minimum (7B-8B models)
- **RAM**: 8GB system + 8GB VRAM/swap
- **GPU**: Optional, speeds up inference
- **Speed**: ~5-10 seconds per analysis

#### Recommended (13B-14B models)  
- **RAM**: 16GB system + 16GB VRAM/swap
- **GPU**: RTX 4060/4070 or equivalent
- **Speed**: ~3-5 seconds per analysis

#### Optimal (34B+ models)
- **RAM**: 32GB+ system + 32GB VRAM/swap  
- **GPU**: RTX 4080/4090 or equivalent
- **Speed**: ~2-3 seconds per analysis

### Model Comparison for Code Analysis

| Model | Size | Quality | Speed | Best For |
|-------|------|---------|-------|----------|
| `codellama:7b` | 4GB | Good | Fast | Quick analysis, limited hardware |
| `codellama:13b` | 8GB | Very Good | Medium | **Recommended balance** |
| `codellama:34b` | 20GB | Excellent | Slow | High-quality analysis |
| `qwen2.5-coder:14b` | 9GB | Very Good | Medium | Alternative to CodeLlama |
| `llama3.1:8b` | 5GB | Good | Fast | General purpose with some code ability |

### Troubleshooting

#### Connection Issues
```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# Start Ollama server manually
ollama serve

# Check logs
journalctl -u ollama  # Linux
# Or check Ollama app logs on Windows/macOS
```

#### Model Not Found
```bash
# List available models
ollama list

# Pull missing model
ollama pull codellama:13b

# Remove old/unused models to free space
ollama rm old-model:version
```

#### Performance Issues
```bash
# Use smaller model for faster analysis
export OLLAMA_MODEL="codellama:7b"

# Limit analysis scope  
refactor-tool --analyze --ai --ai-provider ollama --ai-max-functions 10

# Check system resources
htop  # Linux/macOS
# Task Manager on Windows
```

## Error Handling

If AI analysis fails (network issues, API limits, etc.), the tool gracefully falls back to traditional analysis:

```
🤖 Running AI-powered semantic analysis...
⚠️  AI analysis failed: API rate limit exceeded
   Continuing with basic analysis...
✅ Analysis completed! Scanned 156 functions across 23 files.
```

## API Requirements

- **Internet connection** required for AI features
- **Valid API key** for chosen provider
- **API quotas** may apply (especially for free tiers)
- **Rate limiting** handled automatically with retries

## Privacy & Security

- Code snippets are sent to AI providers for analysis
- No permanent storage by default (check provider policies)
- Consider using environment variables for API keys
- Review your organization's AI usage policies before enabling

## Configuration Examples

### GitHub Copilot Enterprise
```bash
export COPILOT_API_KEY="ghp_xxxxxxxxxxxx"
export OPENAI_BASE_URL="https://api.githubcopilot.com/v1"
refactor-tool --analyze --ai --ai-provider openai
```

### Custom OpenAI-Compatible API
```bash
export OPENAI_API_KEY="your-key"
export OPENAI_BASE_URL="https://your-custom-endpoint.com/v1"
export OPENAI_MODEL="your-model"
refactor-tool --analyze --ai --ai-provider openai
```

### Azure OpenAI
```bash
export OPENAI_API_KEY="your-azure-key"
export OPENAI_BASE_URL="https://your-resource.openai.azure.com/openai/deployments/your-deployment/chat/completions?api-version=2023-05-15"
refactor-tool --analyze --ai
```

## Integration with CI/CD

```yaml
# GitHub Actions example
- name: Analyze code duplications
  env:
    OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
  run: |
    refactor-tool --analyze --ai --ai-max-functions 30 > analysis.md
    # Add analysis.md to PR comment or artifact
```

This AI-enhanced analysis provides deeper insights into code patterns and more actionable refactoring recommendations, helping maintain cleaner and more maintainable Rust codebases.