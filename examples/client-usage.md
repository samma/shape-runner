# ShapeRunner CLI Usage Examples

## Basic Usage

Run a FeatureDesign shape with input from a file:

```bash
cargo run --bin shape-runner-cli -- \
  --shape FeatureDesign \
  --server http://localhost:50051 \
  --input examples/feature-design-input.json
```

## Using stdin

Pipe input from another command:

```bash
echo '{"repo_summary": "A simple blog", "constraints": ["Use SQLite", "No authentication"]}' | \
  cargo run --bin shape-runner-cli -- --input -
```

## Custom Server Address

Connect to a remote ShapeRunner server:

```bash
cargo run --bin shape-runner-cli -- \
  --server http://example.com:50051 \
  --input examples/feature-design-input.json
```

## Output Format

Output as MessagePack (binary format):

```bash
cargo run --bin shape-runner-cli -- \
  --input examples/feature-design-input.json \
  --format msgpack > output.msgpack
```

## Timeout Configuration

Set a custom timeout (in seconds):

```bash
cargo run --bin shape-runner-cli -- \
  --input examples/feature-design-input.json \
  --timeout 120
```

## Complete Example Workflow

1. Start the mock LLM server:
   ```bash
   cargo run --bin mock-llm-server
   ```

2. In another terminal, start the ShapeRunner server:
   ```bash
   LLM_BASE_URL=http://localhost:8080/llm cargo run
   ```

3. Run the CLI client:
   ```bash
   cargo run --bin shape-runner-cli -- \
     --input examples/feature-design-input.json
   ```

