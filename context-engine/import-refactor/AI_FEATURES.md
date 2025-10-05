# AI-Powered Code Analysis Features

The import-refactor tool now includes AI-powered semantic code analysis to enhance duplication detection and provide intelligent refactoring suggestions.

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

## Usage

### Basic Analysis (No AI)
```bash
# Traditional duplication analysis
import-refactor --analyze

# Verbose output with additional insights
import-refactor --analyze --verbose
```

### AI-Enhanced Analysis
```bash
# Enable AI analysis (auto-detects provider from environment)
import-refactor --analyze --ai

# Specify AI provider explicitly
import-refactor --analyze --ai --ai-provider openai
import-refactor --analyze --ai --ai-provider claude

# Limit functions analyzed (to control API costs)
import-refactor --analyze --ai --ai-max-functions 10

# Specify AI model
import-refactor --analyze --ai --ai-model gpt-4
import-refactor --analyze --ai --ai-model claude-3-5-sonnet-20241022
```

### Combined with Import Refactoring
```bash
# Standard import refactoring (unchanged)
import-refactor source_crate target_crate

# Self-refactor with AI analysis
import-refactor --self my_crate --analyze --ai
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
import-refactor --analyze --ai --ai-provider openai
```

### Custom OpenAI-Compatible API
```bash
export OPENAI_API_KEY="your-key"
export OPENAI_BASE_URL="https://your-custom-endpoint.com/v1"
export OPENAI_MODEL="your-model"
import-refactor --analyze --ai --ai-provider openai
```

### Azure OpenAI
```bash
export OPENAI_API_KEY="your-azure-key"
export OPENAI_BASE_URL="https://your-resource.openai.azure.com/openai/deployments/your-deployment/chat/completions?api-version=2023-05-15"
import-refactor --analyze --ai
```

## Integration with CI/CD

```yaml
# GitHub Actions example
- name: Analyze code duplications
  env:
    OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
  run: |
    import-refactor --analyze --ai --ai-max-functions 30 > analysis.md
    # Add analysis.md to PR comment or artifact
```

This AI-enhanced analysis provides deeper insights into code patterns and more actionable refactoring recommendations, helping maintain cleaner and more maintainable Rust codebases.