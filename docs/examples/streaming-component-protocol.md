# Streaming Component Protocol

AutoForge supports rich components embedded in AI streaming text via **partial JSON code blocks**.

## Supported Components

### `table`

Render a data table. The AI should emit:

````markdown
Here is the analysis of your codebase:

```json
{"type": "table", "columns": ["File", "Lines", "Complexity"], "rows": [{"File": "src/main.rs", "Lines": 120, "Complexity": "Low"}]}
```

As you can see, the main entry point is quite small.
````

**Rules:**
- Output the JSON **inline** (single line or compact) so it can be parsed even when truncated mid-stream
- Use `"type": "table"` exactly
- `columns` is an array of strings
- `rows` is an array of objects with keys matching `columns`
- Always close the code fence (`\n```\n`) when done

### `chart` (planned)

Not yet implemented. Future components: `form`, `diagram`, `diff`.

## How the Frontend Parses It

1. `useStreamingDocument(rawText)` scans for `` ```json `` fences
2. `parsePartialJSON()` attempts to parse even **incomplete** JSON:
   - Adds missing closing `}` / `]`
   - Adds missing closing `"`
3. If `value.type === 'table'` → render `<StreamingTable>`
4. If not recognized → render as normal markdown code block
5. `final=true` only when the fence is closed

## Testing It Manually

You can test the table rendering without the AI by pasting this into the Furnace chat (the AI will echo it back, or you can hardcode a mock response):

````
I've analyzed the API endpoints. Here's a summary:

```json
{"type": "table", "columns": ["Endpoint", "Method", "Status"], "rows": [{"Endpoint": "/api/users", "Method": "GET", "Status": "OK"}, {"Endpoint": "/api/users", "Method": "POST", "Status": "OK"}, {"Endpoint": "/api/auth", "Method": "POST", "Status": "Needs Tests"}]}
```

All endpoints are documented. Let me know if you want me to add more.
````

## Prompt Template for the AI

Add this to the system prompt when you want the AI to use tables:

```
When presenting structured data (comparisons, lists, analysis results),
output a JSON table inside a markdown code block:

```json
{"type": "table", "columns": ["Col1", "Col2"], "rows": [{"Col1": "...", "Col2": "..."}]}
```
```
