# Quick Start Guide - ShapeRunner with Ollama

## Prerequisites

1. **Ollama installed and running**
   - Download from [ollama.com](https://ollama.com)
   - Verify it's running: `ollama list`

2. **Model downloaded**
   ```bash
   ollama pull phi3:3.8b
   # or
   ollama pull llama3.2:3b
   ```

## Running ShapeRunner

### Step 1: Start ShapeRunner Server

```powershell
# Set up environment
$env:PATH = ".\protoc\bin;$env:PATH"
$env:OLLAMA_MODEL = "phi3:3.8b"  # or your preferred model

# Start server
cargo run --release --bin shape-runner
```

You should see:
```
ShapeRunner listening on 0.0.0.0:50051
Using LLM endpoint: http://localhost:11434/api/generate
Using Ollama model: phi3:3.8b
```

### Step 2: Run the CLI Client

In another terminal:

```powershell
$env:PATH = ".\protoc\bin;$env:PATH"
cargo run --release --bin shape-runner-cli -- --input examples/feature-design-input.json
```

## Configuration

### Environment Variables

- `OLLAMA_MODEL`: Model name (default: `llama3.2:3b`)
- `LLM_BASE_URL`: Ollama endpoint (default: `http://localhost:11434/api/generate`)

### Examples

**Use a different model:**
```powershell
$env:OLLAMA_MODEL = "mistral:7b-instruct-q4_K_M"
cargo run --release --bin shape-runner
```

**Use mock server instead:**
```powershell
$env:LLM_BASE_URL = "http://localhost:8081/llm"
cargo run --release --bin shape-runner
```

## Troubleshooting

**"LLM did not return valid JSON"**
- The model might be wrapping JSON in markdown. The code should handle this automatically, but if issues persist, try a different model.

**"Connection refused"**
- Make sure Ollama is running: `ollama list`
- Check if port 11434 is accessible

**"Model not found"**
- Pull the model: `ollama pull phi3:3.8b`
- Verify: `ollama list`

## Next Steps

- Try different models to see which gives the best results
- Experiment with different input files
- Check the server logs for detailed error messages
- See `OLLAMA_SETUP.md` for more detailed information

