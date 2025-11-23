# ShapeRunner Demo: Schema Validation & Retry Logic

## How It Works

ShapeRunner uses **schema validation** and **intelligent retry logic** to ensure LLM outputs match your exact requirements.

## Step-by-Step Process

### 1. **Schema Definition**
The system defines what the output should look like:

```rust
{
  "name": string,           // Required field
  "rationale": markdown,     // Required field  
  "components": [           // Required array
    {
      "id": string,
      "responsibility": string,
      "api": markdown
    }
  ],
  "risks": [string]         // Required array of strings
}
```

### 2. **LLM Request**
The system sends a prompt to the LLM that includes:
- The input data (repo summary, constraints)
- A human-readable schema description
- Instructions to output valid JSON

### 3. **Validation**
When the LLM responds, the system validates:
- ✅ All required fields are present
- ✅ Each field has the correct type (string, number, array, object)
- ✅ Nested structures match the schema
- ✅ Arrays contain the correct item types

### 4. **Retry Logic** (if validation fails)

If validation fails, the system:
1. Collects **all** validation errors (not just the first one)
2. Builds a new prompt that includes the errors
3. Retries up to **3 times** total
4. Each retry includes feedback like:
   - "Missing required field at path $.name"
   - "Type mismatch at $.risks[0]: expected string, found number"

### 5. **Success**
Once validation passes, the typed output is returned.

## Example: Successful Flow

**Input:**
```json
{
  "repo_summary": "A web application for managing tasks",
  "constraints": ["Use PostgreSQL", "Use WebSockets"]
}
```

**Output (validated):**
```json
{
  "name": "TaskAndProjectManager",
  "rationale": "# Repository Summary\n...",
  "components": [
    {
      "id": "databaseManager",
      "responsibility": "Handles PostgreSQL connections",
      "api": "POST /database/connection"
    }
  ],
  "risks": ["Risk of data breach..."]
}
```

✅ **All fields present, all types correct** → Success on first attempt!

## Example: Retry Flow

**Attempt 1:**
```json
{
  "rationale": "...",
  "components": [...]
}
```
❌ **Error:** Missing required field at path $.name  
❌ **Error:** Missing required field at path $.risks

**Attempt 2 (with error feedback):**
```json
{
  "name": "MyApp",
  "rationale": "...",
  "components": [...],
  "risks": [123]  // Wrong type!
}
```
❌ **Error:** Type mismatch at $.risks[0]: expected string, found number

**Attempt 3 (with error feedback):**
```json
{
  "name": "MyApp",
  "rationale": "...",
  "components": [...],
  "risks": ["Risk 1", "Risk 2"]
}
```
✅ **All validation passes!** → Success!

## Key Features

1. **Type Safety**: Ensures outputs match your exact schema
2. **Error Feedback**: LLM gets specific error messages to fix issues
3. **Multiple Errors**: Collects all errors at once, not just the first
4. **Automatic Retry**: Up to 3 attempts automatically
5. **JSON Cleaning**: Handles markdown fences, extra text, control characters

## Try It Yourself

Run the demo:
```powershell
cargo run --release --bin shape-runner-cli -- --input examples/feature-design-input.json
```

Watch the server logs to see retry attempts in action!

