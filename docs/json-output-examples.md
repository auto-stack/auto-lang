# JSON Output Format Examples

AutoLang compiler supports machine-readable JSON output for IDE integration and automated tooling.

## Usage

Use the `--format json` flag with any command:

```bash
auto --format json eval "1 +"
auto --format json run script.at
auto --format json transpile script.at
```

## JSON Schema

### Single Error

```json
{
  "message": "Error message",
  "code": "auto_syntax_E0007",
  "severity": "error",
  "spans": [
    {
      "offset": 10,
      "len": 5,
      "label": "Expected identifier"
    }
  ],
  "help": "Fix the syntax error"
}
```

### Multiple Errors

When error recovery is enabled, multiple errors are aggregated:

```json
{
  "message": "aborting due to 2 previous errors",
  "code": "auto_syntax_E0099",
  "severity": "error",
  "help": "Fix the reported errors and try again",
  "related": [
    {
      "message": "Undefined identifier: foo",
      "code": "auto_syntax_E0007"
    },
    {
      "message": "Missing semicolon",
      "code": "auto_syntax_E0007"
    }
  ]
}
```

### Fields

- **message** (string): Human-readable error message
- **code** (string, optional): Error code (e.g., `auto_syntax_E0007`)
- **severity** (string): Either "error" or "warning"
- **spans** (array, optional): Array of source code locations
  - **offset** (number): Byte offset in source
  - **len** (number): Length of error span
  - **label** (string, optional): Label text for the span
- **help** (string, optional): Help text with suggestions
- **related** (array, optional): Related errors (for multi-error output)

## Examples

### Syntax Error

```bash
$ auto --format json eval "1 +"
```

Output:
```json
{
  "code": "auto_syntax_E0099",
  "help": "Fix the reported errors and try again",
  "message": "aborting due to 1 previous error",
  "related": [
    {
      "code": "auto_syntax_E0007",
      "message": "Expected term, got EOF, pos: 1:2:0, next: <eof>"
    }
  ],
  "severity": "error"
}
```

### File Execution

```bash
$ auto --format json run script.at
```

If successful, outputs the result (plain text).  
If errors occur, outputs JSON to stderr.

### IDE Integration

The JSON format is designed for easy parsing by IDEs and editors:

```python
import subprocess
import json
import sys

def check_auto_file(filename):
    result = subprocess.run(
        ["auto", "--format", "json", "run", filename],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        try:
            errors = json.loads(result.stderr)
            for error in errors.get("related", [errors]):
                print(f"Error {error.get('code')}: {error.get('message')}")
        except json.JSONDecodeError:
            print(result.stderr)
    else:
        print(result.stdout)

check_auto_file("example.at")
```

## Comparison: Text vs JSON

### Text Output (default)

```bash
$ auto eval "1 +"
Error: auto_syntax_E0007

  × Expected term, got EOF, pos: 1:2:0, next: <eof>
  ╰─▶ Expected term, got EOF, pos: 1:2:0, next: <eof>
   ╭─[<input>:1:2]
 1 │ 1 +
   ·  ┬
   ·  ╰── Expected term, got EOF, pos: 1:2:0, next: <eof>
   ╰────
```

### JSON Output

```bash
$ auto --format json eval "1 +"
{
  "code": "auto_syntax_E0099",
  "severity": "error",
  "message": "aborting due to 1 previous error",
  "related": [
    {
      "code": "auto_syntax_E0007",
      "message": "Expected term, got EOF, pos: 1:2:0, next: <eof>"
    }
  ],
  "help": "Fix the reported errors and try again"
}
```

## LSP Integration

The JSON format is compatible with Language Server Protocol diagnostics:

```typescript
interface AutoError {
  message: string;
  code?: string;
  severity: 'error' | 'warning';
  spans?: Array<{
    offset: number;
    len: number;
    label?: string;
  }>;
  help?: string;
  related?: Array<{
    message: string;
    code?: string;
  }>;
}
```

Convert to LSP Diagnostic:

```typescript
function autoToLSP(error: AutoError, uri: string): Diagnostic {
  return {
    range: error.spans?.[0] 
      ? { 
          start: positionAt(error.spans[0].offset),
          end: positionAt(error.spans[0].offset + error.spans[0].len)
        }
      : { start: {line: 0, character: 0}, end: {line: 0, character: 0} },
    severity: error.severity === 'error' 
      ? DiagnosticSeverity.Error 
      : DiagnosticSeverity.Warning,
    message: error.message,
    code: error.code,
    relatedInformation: error.related?.map(r => ({
      message: r.message,
      location: { uri, range: { start: {line: 0, character: 0}, end: {line: 0, character: 0} } }
    }))
  };
}
```
