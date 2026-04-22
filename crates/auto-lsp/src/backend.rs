use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::completion;
use crate::diagnostics;
use crate::hover_info;
use crate::goto_def;

/// Document state stored by the LSP
#[derive(Debug, Clone)]
struct DocumentState {
    content: String,
    #[allow(dead_code)]
    version: i32,
}

/// Backend struct implementing the LanguageServer trait
pub struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<String, DocumentState>>>,
    /// Debounce handles to prevent excessive parsing
    /// Maps URI to the handle of the scheduled parse task
    debounce_handles: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

/// Clone implementation for debounced tasks
impl Clone for Backend {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            documents: self.documents.clone(),
            debounce_handles: self.debounce_handles.clone(),
        }
    }
}

impl Backend {
    /// Create a new Backend instance
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(RwLock::new(HashMap::new())),
            debounce_handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get document content by URI
    async fn get_document(&self, uri: &str) -> Option<String> {
        let docs = self.documents.read().await;
        docs.get(uri).map(|doc| doc.content.clone())
    }

    /// Update document content
    async fn update_document(&self, uri: String, content: String, version: i32) {
        let mut docs = self.documents.write().await;
        docs.insert(
            uri,
            DocumentState {
                content,
                version,
            },
        );
    }

    /// Remove document from cache
    async fn close_document(&self, uri: String) {
        // Cancel any pending parse task for this URI
        let mut handles = self.debounce_handles.write().await;
        if let Some(handle) = handles.remove(&uri) {
            handle.abort();
        }
        drop(handles); // Release lock before next operation

        let mut docs = self.documents.write().await;
        docs.remove(&uri);
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    /// Handle initialization request from client
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("AutoLang LSP initializing with workspace: {:?}", params.root_uri),
            )
            .await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Define which operations we support
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![":".to_string(), ".".to_string(), "(".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "auto-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    /// Called after initialization is complete
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "AutoLang LSP initialized!")
            .await;
    }

    /// Handle shutdown request
    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "AutoLang LSP shutting down")
            .await;
        Ok(())
    }

    /// Handle document open
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let content = params.text_document.text.clone();
        let version = params.text_document.version;

        self.client
            .log_message(
                MessageType::INFO,
                format!("Opened document: {}", uri),
            )
            .await;

        self.update_document(uri.clone(), content, version).await;

        // Parse and publish diagnostics
        self.publish_diagnostics_for_uri(&uri).await;
    }

    /// Handle document change
    ///
    /// **Performance Fix**: Debounce parsing to avoid excessive CPU usage (10%+ → <1%)
    /// Only parses 150ms after the last change, not on every keystroke
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let version = params.text_document.version;

        // Apply changes
        // Check if this is a full document update or incremental changes
        let current_content = self.get_document(&uri).await.unwrap_or_default();
        let new_content = if let Some(first_change) = params.content_changes.first() {
            // If the first change has no range, it's a full document update
            if first_change.range.is_none() {
                first_change.text.clone()
            } else {
                // Apply incremental changes to the existing content
                let mut content = current_content;
                for change in &params.content_changes {
                    if let Some(range) = change.range {
                        // Apply the change to the content
                        content = apply_text_change(&content, &change.text, range);
                    }
                }
                content
            }
        } else {
            current_content
        };

        self.update_document(uri.clone(), new_content.clone(), version).await;

        // **Debouncing**: Cancel any existing parse task for this URI
        {
            let mut handles = self.debounce_handles.write().await;
            if let Some(old_handle) = handles.remove(&uri) {
                old_handle.abort();
            }
        }

        // Clone necessary data for the spawned task
        let uri_clone = uri.clone();
        let backend = self.clone(); // Clone backend to use in spawned task

        // Spawn a new debounced parse task
        let handle = tokio::spawn(async move {
            // Wait for debounce delay (150ms)
            tokio::time::sleep(Duration::from_millis(150)).await;

            // Parse and publish diagnostics
            backend.publish_diagnostics_for_uri(&uri_clone).await;

            // Remove handle from map after completion
            let mut handles = backend.debounce_handles.write().await;
            handles.remove(&uri_clone);
        });

        // Store the handle
        let mut handles = self.debounce_handles.write().await;
        handles.insert(uri, handle);
    }

    /// Handle document close
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();

        self.client
            .log_message(
                MessageType::INFO,
                format!("Closed document: {}", uri),
            )
            .await;

        self.close_document(uri).await;
    }

    /// Provide code completion
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params
            .text_document_position
            .text_document
            .uri
            .to_string();

        let position = params.text_document_position.position;

        // Get document content
        if let Some(content) = self.get_document(&uri).await {
            // Extract trigger character if available
            let trigger_char = params.context.as_ref().and_then(|ctx| {
                ctx.trigger_character.as_ref().and_then(|s| s.chars().next())
            });

            let items = completion::complete(&content, position, &uri, trigger_char);

            Ok(Some(CompletionResponse::Array(items)))
        } else {
            // No content available, return empty completion
            Ok(Some(CompletionResponse::Array(vec![])))
        }
    }

    /// Provide hover information
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        // Get document content
        let content = match self.get_document(&uri).await {
            Some(content) => content,
            None => return Ok(None),
        };

        // Provide hover information
        Ok(hover_info::hover(&content, position, &uri))
    }

    /// Go to definition
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        // Get document content
        let content = match self.get_document(&uri).await {
            Some(content) => content,
            None => return Ok(None),
        };

        // Find the definition
        Ok(goto_def::find_definition(&content, position, &uri))
    }

    /// Find references
    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params
            .text_document_position
            .text_document
            .uri
            .to_string();

        self.client
            .log_message(
                MessageType::LOG,
                format!("Find references requested in: {}", uri),
            )
            .await;

        // TODO: Implement actual find references from AST/symbol table
        // For now, return None
        Ok(None)
    }

    /// Document symbols (outline view)
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.to_string();

        // Get document content
        let content = match self.get_document(&uri).await {
            Some(content) => content,
            None => return Ok(None),
        };

        let symbols = extract_document_symbols(&content);

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(DocumentSymbolResponse::Array(symbols)))
        }
    }

    /// Workspace symbols (search across project)
    async fn symbol(&self, params: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        self.client
            .log_message(
                MessageType::LOG,
                format!("Workspace symbols requested: {:?}", params.query),
            )
            .await;

        // TODO: Search across all documents
        // For now, return None
        Ok(None)
    }

    /// Rename symbol
    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params
            .text_document_position
            .text_document
            .uri
            .to_string();
        let new_name = params.new_name;

        self.client
            .log_message(
                MessageType::LOG,
                format!("Rename requested in {}: new_name = {}", uri, new_name),
            )
            .await;

        // TODO: Implement actual rename
        // For now, return None
        Ok(None)
    }

    /// Code actions (quick fixes)
    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri.to_string();

        self.client
            .log_message(
                MessageType::LOG,
                format!("Code action requested for: {}", uri),
            )
            .await;

        // TODO: Provide code actions for diagnostics
        // For now, return None
        Ok(None)
    }
}

impl Backend {
    /// Publish diagnostics for a document with timeout protection
    async fn publish_diagnostics_for_uri(&self, uri: &str) {
        // Get document content
        if let Some(content) = self.get_document(uri).await {
            let uri_clone = uri.to_string(); // Clone to move into spawn_blocking
            let content_clone = content.clone(); // Clone content to move into spawn_blocking

            self.client
                .log_message(
                    MessageType::LOG,
                    format!("Parsing document for diagnostics: {}", uri),
                )
                .await;

            // Parse with timeout to prevent hanging
            let diagnostics = tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                tokio::task::spawn_blocking(move || {
                    diagnostics::parse_diagnostics(&uri_clone, &content_clone, 0)
                })
            ).await;

            match diagnostics {
                Ok(Ok(diagnostics)) => {
                    // Publish diagnostics to VSCode
                    self.client
                        .log_message(
                            MessageType::LOG,
                            format!("Publishing {} diagnostics for {}", diagnostics.len(), uri),
                        )
                        .await;

                    if let Ok(url) = Url::parse(uri) {
                        self.client
                            .publish_diagnostics(
                                url,
                                diagnostics,
                                None,
                            )
                            .await;
                    } else {
                        self.client
                            .log_message(
                                MessageType::ERROR,
                                format!("Failed to parse URI: {}", uri),
                            )
                            .await;
                    }
                }
                Ok(Err(e)) => {
                    // Task join error
                    self.client
                        .log_message(
                            MessageType::ERROR,
                            format!("Task join error while parsing: {:?}", e),
                        )
                        .await;
                }
                Err(_) => {
                    // Timeout
                    self.client
                        .log_message(
                            MessageType::ERROR,
                            format!("Parsing timed out for: {}", uri),
                        )
                        .await;

                    // Clear diagnostics on timeout to avoid showing stale errors
                    if let Ok(url) = Url::parse(uri) {
                        self.client
                            .publish_diagnostics(url, vec![], None)
                            .await;
                    }
                }
            }
        }
    }

    /// Publish diagnostics for a document (legacy method - deprecated)
    #[allow(dead_code)]
    async fn publish_diagnostics(&self, uri: &str) {
        self.publish_diagnostics_for_uri(uri).await;
    }
}

/// Extract document symbols from source code using regex
fn extract_document_symbols(content: &str) -> Vec<DocumentSymbol> {
    use regex::Regex;
    let mut symbols = Vec::new();

    // Extract fn declarations
    if let Ok(re) = Regex::new(r"^\s*(?:pub\s+)?(?:mut\s+|static\s+)?fn\s+(\w+)") {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                if let Some(name) = caps.get(1) {
                    let start_char = line.find(name.as_str()).unwrap_or(0) as u32;
                    symbols.push(DocumentSymbol {
                        name: name.as_str().to_string(),
                        detail: Some("fn".to_string()),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position { line: line_num as u32, character: 0 },
                            end: Position { line: line_num as u32, character: line.len() as u32 },
                        },
                        selection_range: Range {
                            start: Position { line: line_num as u32, character: start_char },
                            end: Position { line: line_num as u32, character: start_char + name.as_str().len() as u32 },
                        },
                        children: None,
                    });
                }
            }
        }
    }

    // Extract type declarations
    if let Ok(re) = Regex::new(r"^\s*(?:pub\s+)?type\s+(\w+)") {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                if let Some(name) = caps.get(1) {
                    let start_char = line.find(name.as_str()).unwrap_or(0) as u32;
                    symbols.push(DocumentSymbol {
                        name: name.as_str().to_string(),
                        detail: Some("type".to_string()),
                        kind: SymbolKind::CLASS,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position { line: line_num as u32, character: 0 },
                            end: Position { line: line_num as u32, character: line.len() as u32 },
                        },
                        selection_range: Range {
                            start: Position { line: line_num as u32, character: start_char },
                            end: Position { line: line_num as u32, character: start_char + name.as_str().len() as u32 },
                        },
                        children: None,
                    });
                }
            }
        }
    }

    // Extract enum declarations
    if let Ok(re) = Regex::new(r"^\s*(?:pub\s+)?enum\s+(\w+)") {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                if let Some(name) = caps.get(1) {
                    let start_char = line.find(name.as_str()).unwrap_or(0) as u32;
                    symbols.push(DocumentSymbol {
                        name: name.as_str().to_string(),
                        detail: Some("enum".to_string()),
                        kind: SymbolKind::ENUM,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position { line: line_num as u32, character: 0 },
                            end: Position { line: line_num as u32, character: line.len() as u32 },
                        },
                        selection_range: Range {
                            start: Position { line: line_num as u32, character: start_char },
                            end: Position { line: line_num as u32, character: start_char + name.as_str().len() as u32 },
                        },
                        children: None,
                    });
                }
            }
        }
    }

    // Extract spec declarations (interface)
    if let Ok(re) = Regex::new(r"^\s*(?:pub\s+)?spec\s+(\w+)") {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                if let Some(name) = caps.get(1) {
                    let start_char = line.find(name.as_str()).unwrap_or(0) as u32;
                    symbols.push(DocumentSymbol {
                        name: name.as_str().to_string(),
                        detail: Some("spec".to_string()),
                        kind: SymbolKind::INTERFACE,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position { line: line_num as u32, character: 0 },
                            end: Position { line: line_num as u32, character: line.len() as u32 },
                        },
                        selection_range: Range {
                            start: Position { line: line_num as u32, character: start_char },
                            end: Position { line: line_num as u32, character: start_char + name.as_str().len() as u32 },
                        },
                        children: None,
                    });
                }
            }
        }
    }

    // Extract const declarations
    if let Ok(re) = Regex::new(r"^\s*(?:pub\s+)?const\s+(\w+)") {
        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                if let Some(name) = caps.get(1) {
                    let start_char = line.find(name.as_str()).unwrap_or(0) as u32;
                    symbols.push(DocumentSymbol {
                        name: name.as_str().to_string(),
                        detail: Some("const".to_string()),
                        kind: SymbolKind::CONSTANT,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position { line: line_num as u32, character: 0 },
                            end: Position { line: line_num as u32, character: line.len() as u32 },
                        },
                        selection_range: Range {
                            start: Position { line: line_num as u32, character: start_char },
                            end: Position { line: line_num as u32, character: start_char + name.as_str().len() as u32 },
                        },
                        children: None,
                    });
                }
            }
        }
    }

    symbols
}

/// Apply a text change to content at the given range
fn apply_text_change(content: &str, new_text: &str, range: Range) -> String {
    let _lines: Vec<&str> = content.lines().collect();

    // Convert LSP position to byte offset
    let start = position_to_offset(content, &range.start);
    let end = position_to_offset(content, &range.end);

    // Build new content
    let mut result = String::new();
    result.push_str(&content[..start]);
    result.push_str(new_text);
    result.push_str(&content[end..]);

    result
}

/// Convert LSP Position to byte offset in the content
fn position_to_offset(content: &str, position: &Position) -> usize {
    let lines: Vec<&str> = content.lines().collect();

    let mut offset = 0;
    // Add all lines before the target line
    for i in 0..position.line as usize {
        if let Some(line) = lines.get(i) {
            offset += line.len();
            // Add newline character (assuming \n)
            offset += 1;
        }
    }

    // Add characters in the target line up to the target character
    if let Some(line) = lines.get(position.line as usize) {
        let chars: Vec<char> = line.chars().collect();
        let char_offset = position.character.min(chars.len() as u32) as usize;
        offset += chars[..char_offset].iter().collect::<String>().len();
    }

    offset
}
