# ShapeRunner

ShapeRunner is a gRPC service that executes "shapes" - structured LLM operations with schema validation. It provides a type-safe way to generate and validate structured outputs from language models.

## Overview

ShapeRunner allows you to:
- Execute typed operations ("shapes") that use LLMs to generate structured outputs
- Automatically validate LLM outputs against predefined schemas
- Retry with validation feedback when outputs don't match the schema
- Use efficient MessagePack serialization for internal communication

Currently, ShapeRunner implements one shape:
- **FeatureDesign**: Takes a repository summary and constraints, generates a feature design with components, rationale, and risks

## Architecture

```
┌─────────────┐      gRPC       ┌──────────────┐      HTTP      ┌──────────┐
│   Client    │ ───────────────> │ ShapeRunner  │ ─────────────> │   LLM    │
│   (CLI)     │                  │   Service    │                │  Server  │
└─────────────┘                  └──────────────┘                └──────────┘
                                        │
                                        │ Validate & Retry
                                        ▼
                                 ┌──────────────┐
                                 │   Schema     │
                                 │  Validator   │
                                 └──────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.70+ (with Cargo)
- An LLM endpoint (or use the included mock server)

### Building

```bash
cargo build --release
```

### Running the Service

#### Option 1: Using Ollama (Recommended)

1. **Install and start Ollama** (if not already running):
   - Download from [ollama.com](https://ollama.com)
   - Pull a model: `ollama pull llama3.2:3b`
   - Ollama runs automatically on `http://localhost:11434`

2. **Start the ShapeRunner server**:
   ```bash
   # Using default Ollama endpoint and model
   cargo run
   
   # Or specify a custom model
   OLLAMA_MODEL=phi3:mini cargo run
   ```
   The server listens on `0.0.0.0:50051` by default.

#### Option 2: Using Mock Server (for testing)

1. **Start the mock LLM server**:
   ```bash
   cargo run --bin mock-llm-server
   ```
   This starts a mock LLM server on `http://localhost:8081/llm`.

2. **Start the ShapeRunner server**:
   ```bash
   LLM_BASE_URL=http://localhost:8081/llm cargo run
   ```

3. **Run the CLI client**:
   ```bash
   cargo run --bin shape-runner-cli -- \
     --input examples/feature-design-input.json
   ```

## Configuration

### Environment Variables

- `LLM_BASE_URL`: URL of the LLM endpoint (default: `http://localhost:11434/api/generate` for Ollama, or `http://localhost:8081/llm` for mock server)
- `OLLAMA_MODEL`: Model name to use with Ollama (default: `llama3.2:3b`)
- `MOCK_LLM_PORT`: Port for mock LLM server (default: `8081`)
- `MOCK_LLM_FAIL_ATTEMPTS`: Number of failed attempts before success (default: `1`)

See `.env.example` for a template.

## CLI Usage

The `shape-runner-cli` tool provides a command-line interface to the ShapeRunner service.

### Basic Usage

```bash
cargo run --bin shape-runner-cli -- \
  --shape FeatureDesign \
  --server http://localhost:50051 \
  --input examples/feature-design-input.json
```

### Options

- `--shape, -s`: Shape ID to execute (default: `FeatureDesign`)
- `--server, -S`: Server address (default: `http://localhost:50051`)
- `--input, -i`: Input file path or `-` for stdin (default: `-`)
- `--format, -f`: Output format: `json` or `msgpack` (default: `json`)
- `--timeout, -t`: Request timeout in seconds (default: `60`)

### Examples

**Read from stdin:**
```bash
echo '{"repo_summary": "A blog", "constraints": ["Use SQLite"]}' | \
  cargo run --bin shape-runner-cli -- --input -
```

**Output as MessagePack:**
```bash
cargo run --bin shape-runner-cli -- \
  --input examples/feature-design-input.json \
  --format msgpack > output.msgpack
```

**Custom timeout:**
```bash
cargo run --bin shape-runner-cli -- \
  --input examples/feature-design-input.json \
  --timeout 120
```

## API Documentation

### gRPC Service

The ShapeRunner service exposes a single gRPC method:

```protobuf
service ShapeRunner {
  rpc Run (RunRequest) returns (RunResponse);
}

message RunRequest {
  string shape_id = 1;
  bytes input = 2;
}

message RunResponse {
  bytes output = 1;
  bool ok = 2;
  string error = 3;
}
```

### FeatureDesign Shape

**Input** (`FeatureDesignInput`):
```json
{
  "repo_summary": "Description of the repository",
  "constraints": ["Constraint 1", "Constraint 2"]
}
```

**Output** (`FeatureDesignOutput`):
```json
{
  "name": "Feature name",
  "rationale": "Markdown rationale",
  "components": [
    {
      "id": "component-id",
      "responsibility": "What this component does",
      "api": "API description in markdown"
    }
  ],
  "risks": ["Risk 1", "Risk 2"]
}
```

## Development

### Project Structure

```
shape-runner/
├── src/
│   ├── main.rs           # gRPC server implementation
│   ├── client.rs         # gRPC client library
│   ├── codec.rs          # Serialization codecs (MsgPack, JSON)
│   ├── llm.rs            # LLM client with retry logic
│   ├── shape.rs          # Shape definitions (FeatureDesign)
│   ├── types.rs          # Type system and validation
│   ├── rpc.rs            # Generated gRPC code
│   └── bin/
│       ├── shape-runner-cli.rs    # CLI tool
│       └── mock-llm-server.rs     # Mock LLM server
├── proto/
│   └── shaperunner.proto # gRPC service definition
├── examples/
│   ├── feature-design-input.json
│   └── client-usage.md
└── Cargo.toml
```

### Adding New Shapes

To add a new shape:

1. Define input/output types in `src/shape.rs`
2. Create a TypeDef for validation in `src/shape.rs`
3. Add a handler in `src/main.rs` in the `run` method
4. Update the CLI if needed

### Testing

Run tests:
```bash
cargo test
```

Test with mock LLM:
```bash
# Terminal 1: Start mock LLM
cargo run --bin mock-llm-server

# Terminal 2: Start ShapeRunner
LLM_BASE_URL=http://localhost:8080/llm cargo run

# Terminal 3: Run CLI
cargo run --bin shape-runner-cli -- --input examples/feature-design-input.json
```

## How It Works

1. **Client** sends a shape request with input data (encoded as MessagePack)
2. **ShapeRunner** decodes the input and validates the shape ID
3. **ShapeRunner** calls the **LLM** with a prompt that includes:
   - The input data
   - A schema description for the expected output
   - Any validation errors from previous attempts (for retries)
4. **LLM** returns JSON output
5. **ShapeRunner** validates the output against the schema
6. If validation fails, it retries (up to 3 times) with error feedback
7. Once valid, the output is encoded and returned to the client

## License

[Add your license here]

## Contributing

[Add contribution guidelines here]

