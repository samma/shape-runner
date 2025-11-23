# Ollama Setup Guide for ShapeRunner

## Quick Start

1. **Install Ollama** (if not already installed):
   - Download from [ollama.com](https://ollama.com)
   - Install and start Ollama

2. **Pull a model**:
   ```bash
   ollama pull llama3.2:3b
   ```
   
   Recommended models for 10+ concurrent instances on GTX 3090:
   - `llama3.2:3b` - Best for many instances (2-3GB per instance)
   - `phi3:mini` - Good balance (3-4GB per instance)
   - `mistral:7b-instruct-q4_K_M` - Higher quality (4-5GB per instance)

3. **Start ShapeRunner**:
   ```bash
   # Using default model (llama3.2:3b)
   cargo run --release --bin shape-runner
   
   # Or specify a different model
   OLLAMA_MODEL=phi3:mini cargo run --release --bin shape-runner
   ```

4. **Run the CLI**:
   ```bash
   cargo run --release --bin shape-runner-cli -- --input examples/feature-design-input.json
   ```

## Configuration

### Environment Variables

- `LLM_BASE_URL`: Ollama API endpoint (default: `http://localhost:11434/api/generate`)
- `OLLAMA_MODEL`: Model name to use (default: `llama3.2:3b`)

### Examples

**Use a specific model:**
```bash
OLLAMA_MODEL=mistral:7b-instruct-q4_K_M cargo run --release --bin shape-runner
```

**Use custom Ollama endpoint:**
```bash
LLM_BASE_URL=http://localhost:11434/api/generate OLLAMA_MODEL=phi3:mini cargo run --release --bin shape-runner
```

**Use mock server instead:**
```bash
LLM_BASE_URL=http://localhost:8081/llm cargo run --release --bin shape-runner
```

## Testing Ollama

Test that Ollama is working:
```bash
curl http://localhost:11434/api/generate -d '{
  "model": "llama3.2:3b",
  "prompt": "Say hello",
  "stream": false
}'
```

## Troubleshooting

**Ollama not responding:**
- Make sure Ollama is running (check the Ollama UI or run `ollama serve`)
- Verify the model is downloaded: `ollama list`
- Check if port 11434 is accessible: `curl http://localhost:11434/api/tags`

**Model not found:**
- Pull the model: `ollama pull llama3.2:3b`
- Check available models: `ollama list`

**Out of memory:**
- Use a smaller model (e.g., `llama3.2:3b` instead of `mistral:7b`)
- Reduce concurrent instances
- Check GPU memory usage

