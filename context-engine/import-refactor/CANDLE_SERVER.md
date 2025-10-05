# Candle LLM Server

This document describes the Candle-based LLM server functionality added to the import-refactor tool. The server allows you to download, host, and run large language models locally using the Candle framework.

## Features

- 🚀 **Local LLM Hosting**: Run models entirely offline on your machine
- 📥 **Model Management**: Download and manage models from Hugging Face Hub
- 🔧 **Hardware Optimization**: Automatic device selection (CPU/CUDA/Metal)
- 🌐 **HTTP API**: OpenAI-compatible REST API endpoints
- ⚙️ **Configuration**: Flexible configuration with TOML files
- 📊 **System Compatibility**: Built-in system requirement checking

## Quick Start

### 1. Build with Embedded LLM Support

```bash
cargo build --release --features embedded-llm
```

### 2. Initialize Configuration

```bash
./target/release/import-refactor --init-config
```

This creates a `candle-server.toml` configuration file with sensible defaults.

### 3. List Available Models

```bash
./target/release/import-refactor --list-models
```

This shows recommended models and checks your system compatibility.

### 4. Download a Model

```bash
./target/release/import-refactor --download-model "microsoft/CodeLlama-7b-Instruct-hf"
```

### 5. Start the Server

```bash
./target/release/import-refactor --serve
```

The server will start on `http://127.0.0.1:8080` by default.

## Available Commands

| Command | Description |
|---------|-------------|
| `--serve` | Start the Candle LLM server |
| `--download-model <model-id>` | Download a specific model |
| `--list-models` | Show available models and system compatibility |
| `--init-config` | Create default configuration file |
| `--config <path>` | Use custom configuration file |
| `--host <host>` | Set server host (default: 127.0.0.1) |
| `--port <port>` | Set server port (default: 8080) |

## Recommended Models

### CodeLlama 7B Instruct
- **Model ID**: `microsoft/CodeLlama-7b-Instruct-hf`
- **Size**: ~13 GB
- **RAM Required**: 16 GB
- **VRAM Required**: 8 GB (for GPU acceleration)
- **Best for**: General code analysis and generation

### CodeLlama 13B Instruct
- **Model ID**: `microsoft/CodeLlama-13b-Instruct-hf`
- **Size**: ~25 GB
- **RAM Required**: 32 GB
- **VRAM Required**: 16 GB
- **Best for**: Complex refactoring analysis

### Qwen2.5 Coder 7B
- **Model ID**: `Qwen/Qwen2.5-Coder-7B-Instruct`
- **Size**: ~14 GB
- **RAM Required**: 16 GB
- **VRAM Required**: 8 GB
- **Best for**: Rust and systems programming

### DeepSeek Coder 6.7B
- **Model ID**: `deepseek-ai/deepseek-coder-6.7b-instruct`
- **Size**: ~12 GB
- **RAM Required**: 12 GB
- **VRAM Required**: 6 GB
- **Best for**: Fast code suggestions

## API Endpoints

### OpenAI-Compatible Chat Completions

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "microsoft/CodeLlama-7b-Instruct-hf",
    "messages": [
      {"role": "user", "content": "Explain this Rust function: fn add(a: i32, b: i32) -> i32 { a + b }"}
    ],
    "max_tokens": 500
  }'
```

### Simple Text Generation

```bash
curl -X POST http://localhost:8080/generate \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "// Rust function to calculate fibonacci numbers\nfn fibonacci(n: u32) -> u32 {",
    "max_tokens": 200
  }'
```

### Health Check

```bash
curl http://localhost:8080/health
```

## Configuration

The configuration file `candle-server.toml` allows you to customize:

```toml
host = "127.0.0.1"
port = 8080

[model]
model_id = "microsoft/CodeLlama-7b-Instruct-hf"
device = "auto"  # "cpu", "cuda", "metal", or "auto"
temperature = 0.1
top_p = 0.9
max_tokens = 2048
context_length = 4096

[server]
max_concurrent_requests = 4
request_timeout_seconds = 300
enable_cors = true
log_level = "info"

[cache]
cache_dir = "/path/to/cache"
max_cache_size_gb = 50.0
cleanup_old_models = true
model_ttl_days = 30
```

## System Requirements

### Minimum Requirements
- **RAM**: 8-16 GB (depending on model)
- **Storage**: 10-30 GB free space per model
- **CPU**: Modern multi-core processor

### Recommended for GPU Acceleration
- **NVIDIA GPU**: 8+ GB VRAM with CUDA support
- **Apple Silicon**: M1/M2/M3 with Metal support
- **AMD GPU**: OpenCL support (experimental)

## Hardware Optimization

The server automatically detects and optimizes for your hardware:

- **CUDA**: Automatically used if NVIDIA GPU with CUDA is available
- **Metal**: Used on Apple Silicon Macs for optimal performance  
- **CPU**: Fallback option that works on all systems

You can override automatic detection by setting `device` in the configuration file.

## Integration with Code Analysis

The Candle server integrates seamlessly with the existing code analysis features:

```bash
# Use the local Candle server for AI analysis
./target/release/import-refactor --analyze --ai --ai-provider embedded
```

## Troubleshooting

### Model Download Issues
- Ensure you have sufficient disk space
- Check your internet connection
- Verify the model ID is correct

### Server Won't Start
- Check if the port is already in use
- Verify the model is downloaded and accessible
- Check system RAM/VRAM requirements

### Performance Issues
- Try a smaller model if running out of memory
- Enable GPU acceleration if available
- Reduce `max_tokens` in requests

### CUDA Issues
- Ensure CUDA drivers are installed and up to date
- Check CUDA compatibility with your GPU
- Try CPU mode if GPU acceleration fails

## Development

### Building from Source

```bash
# Development build with embedded LLM support
cargo build --features embedded-llm

# Release build with optimizations
cargo build --release --features embedded-llm
```

### Testing

```bash
# Run tests for the candle client
cargo test --features embedded-llm candle_client

# Run configuration tests
cargo test --features embedded-llm candle_config
```

## License

This functionality is part of the import-refactor tool and follows the same license terms.