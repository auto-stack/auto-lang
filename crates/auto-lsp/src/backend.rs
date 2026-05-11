use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer};

use crate::completion;
use crate::diagnostics;
use crate::hover_info;
use crate::goto_def;
use crate::workspace;
use crate::signature_help;

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
    /// Workspace root directory (detected from initialize params)
    workspace_root: Arc<RwLock<Option<PathBuf>>>,
    /// Filesystem resolver for cross-file module resolution
    resolver: Arc<RwLock<Option<auto_lang::resolver::FilesystemResolver>>>,
}

/// Clone implementation for debounced tasks
impl Clone for Backend {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            documents: self.documents.clone(),
            debounce_handles: self.debounce_handles.clone(),
            workspace_root: self.workspace_root.clone(),
            resolver: self.resolver.clone(),
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
            workspace_root: Arc::new(RwLock::new(None)),
            resolver: Arc::new(RwLock::new(None)),
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

    /// Build workspace state for a document, resolving its imports
    async fn build_workspace_state(&self, uri: &str, content: &str) -> Option<workspace::WorkspaceState> {
        let resolver = self.resolver.read().await.clone()?;
        let document_path = std::path::PathBuf::from(uri.strip_prefix("file://").unwrap_or(uri));
        Some(workspace::build_workspace_state(content, &document_path, &resolver))
    }
}

impl LanguageServer for Backend {
    /// Handle initialization request from client
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "AutoLang LSP v{} initializing with workspace: {:?}",
                    env!("CARGO_PKG_VERSION"),
                    params.root_uri
                ),
            )
            .await;

        // Detect workspace root and create resolver
        if let Some(root_uri) = params.root_uri {
            if let Some(root_path) = root_uri.to_file_path() {
                let root_path = root_path.into_owned();
                let mut workspace_root = self.workspace_root.write().await;
                *workspace_root = Some(root_path.clone());
                drop(workspace_root);

                let resolver = workspace::create_resolver(&root_path);
                let mut resolver_lock = self.resolver.write().await;
                *resolver_lock = Some(resolver);
                drop(resolver_lock);

                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("Workspace root set to: {:?}", root_path),
                    )
                    .await;
            }
        }

        Ok(InitializeResult {
            offset_encoding: None,
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
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: None,
                    work_done_progress_options: Default::default(),
                }),
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
            .log_message(
                MessageType::INFO,
                format!("AutoLang LSP v{} initialized!", env!("CARGO_PKG_VERSION")),
            )
            .await;
    }

    /// Handle shutdown request
    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("AutoLang LSP v{} shutting down", env!("CARGO_PKG_VERSION")),
            )
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

    /// Provide signature help
    async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> Result<Option<SignatureHelp>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        let content = match self.get_document(&uri).await {
            Some(content) => content,
            None => return Ok(None),
        };

        Ok(signature_help::get_signature_help(&content, position))
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

        // Try workspace-aware goto-definition first
        if let Some(ws_state) = self.build_workspace_state(&uri, &content).await {
            if let Some(result) = goto_def::find_definition_workspace(&content, position, &uri, &ws_state) {
                return Ok(Some(result));
            }
        }

        // Fall back to single-file goto-definition
        Ok(goto_def::find_definition(&content, position, &uri))
    }

    /// Find references
    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params
            .text_document_position
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position.position;

        let content = match self.get_document(&uri).await {
            Some(content) => content,
            None => return Ok(None),
        };

        // Get the word at the cursor position
        let lines: Vec<&str> = content.lines().collect();
        let line = match lines.get(position.line as usize) {
            Some(l) => l,
            None => return Ok(None),
        };
        let word = match get_word_at_position(line, position.character as usize) {
            Some(w) => w,
            None => return Ok(None),
        };

        let mut locations = Vec::new();

        // Search workspace for definitions of this symbol
        if let Some(ws_state) = self.build_workspace_state(&uri, &content).await {
            let db = &ws_state.db;
            for frag_id in db.all_fragment_ids() {
                if let Some(meta) = db.get_fragment_meta(&frag_id) {
                    if meta.name.as_str() == word {
                        let file_path = db.get_file_path(meta.file_id)
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| uri.clone());

                        let target_uri: Uri = if file_path.starts_with("file://") {
                            file_path.parse().unwrap_or_else(|_| uri.parse().unwrap())
                        } else {
                            format!("file://{}", file_path).parse().unwrap_or_else(|_| uri.parse().unwrap())
                        };

                        locations.push(Location {
                            uri: target_uri,
                            range: Range {
                                start: Position {
                                    line: meta.span.line.saturating_sub(1) as u32,
                                    character: meta.span.column.saturating_sub(1) as u32,
                                },
                                end: Position {
                                    line: meta.span.line.saturating_sub(1) as u32,
                                    character: meta.span.column.saturating_sub(1) as u32,
                                },
                            },
                        });
                    }
                }
            }
        }

        // Also find all occurrences in the current file by simple text search
        for (line_num, line_str) in content.lines().enumerate() {
            for (offset, _) in line_str.match_indices(&word) {
                // Skip if this is the definition (already added above)
                let is_definition = locations.iter().any(|loc| {
                    loc.uri.to_string() == uri && loc.range.start.line == line_num as u32
                });
                if !is_definition {
                    locations.push(Location {
                        uri: uri.parse().unwrap(),
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: offset as u32,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: (offset + word.len()) as u32,
                            },
                        },
                    });
                }
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
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
            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        }
    }

    /// Workspace symbols (search across project)
    async fn symbol(&self, params: WorkspaceSymbolParams) -> Result<Option<WorkspaceSymbolResponse>> {
        self.client
            .log_message(
                MessageType::LOG,
                format!("Workspace symbols requested: {:?}", params.query),
            )
            .await;

        let query = params.query.to_lowercase();
        let mut symbols = Vec::new();

        // Search across all open documents
        let docs = self.documents.read().await;
        for (uri, doc) in docs.iter() {
            if let Some(ws_state) = self.build_workspace_state(uri, &doc.content).await {
                let db = &ws_state.db;
                for frag_id in db.all_fragment_ids() {
                    if let Some(meta) = db.get_fragment_meta(&frag_id) {
                        let name = meta.name.as_str();
                        if query.is_empty() || name.to_lowercase().contains(&query) {
                            let kind = match meta.kind {
                                auto_lang::database::FragKind::Function => SymbolKind::FUNCTION,
                                auto_lang::database::FragKind::Struct => SymbolKind::STRUCT,
                                auto_lang::database::FragKind::Enum => SymbolKind::ENUM,
                                auto_lang::database::FragKind::Const => SymbolKind::CONSTANT,
                                auto_lang::database::FragKind::Spec => SymbolKind::INTERFACE,
                                auto_lang::database::FragKind::Impl => SymbolKind::METHOD,
                            };

                            let file_path = db.get_file_path(meta.file_id)
                                .map(|p| p.to_string())
                                .unwrap_or_else(|| uri.clone());

                            let target_uri = if file_path.starts_with("file://") {
                                file_path.parse().unwrap_or_else(|_| uri.parse().unwrap())
                            } else {
                                format!("file://{}", file_path).parse().unwrap_or_else(|_| uri.parse().unwrap())
                            };

                            symbols.push(WorkspaceSymbol {
                                name: name.to_string(),
                                kind,
                                location: OneOf::Left(Location {
                                    uri: target_uri,
                                    range: Range {
                                        start: Position {
                                            line: meta.span.line.saturating_sub(1) as u32,
                                            character: meta.span.column.saturating_sub(1) as u32,
                                        },
                                        end: Position {
                                            line: meta.span.line.saturating_sub(1) as u32,
                                            character: meta.span.column.saturating_sub(1) as u32,
                                        },
                                    },
                                }),
                                container_name: None,
                                data: None,
                                tags: None,
                            });
                        }
                    }
                }
            }
        }
        drop(docs);

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(WorkspaceSymbolResponse::Nested(symbols)))
        }
    }

    /// Rename symbol
    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params
            .text_document_position
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        let content = match self.get_document(&uri).await {
            Some(content) => content,
            None => return Ok(None),
        };

        // Get the word at the cursor position
        let lines: Vec<&str> = content.lines().collect();
        let line = match lines.get(position.line as usize) {
            Some(l) => l,
            None => return Ok(None),
        };
        let word = match get_word_at_position(line, position.character as usize) {
            Some(w) => w,
            None => return Ok(None),
        };

        // Build text edits for all occurrences in the current file
        let mut text_edits = Vec::new();
        for (line_num, line_str) in content.lines().enumerate() {
            for (offset, _) in line_str.match_indices(&word) {
                text_edits.push(TextEdit {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: offset as u32,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: (offset + word.len()) as u32,
                        },
                    },
                    new_text: new_name.clone(),
                });
            }
        }

        if text_edits.is_empty() {
            Ok(None)
        } else {
            let mut changes = std::collections::HashMap::new();
            changes.insert(uri.parse().unwrap(), text_edits);
            Ok(Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }))
        }
    }

    /// Code actions (quick fixes)
    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri.to_string();
        let range = params.range;

        let content = match self.get_document(&uri).await {
            Some(content) => content,
            None => return Ok(None),
        };

        let mut actions = Vec::new();

        // Build workspace state for auto-import suggestions
        if let Some(ws_state) = self.build_workspace_state(&uri, &content).await {
            if let Ok(store) = ws_state.type_store.read() {
                // Get the word at the start of the range
                let lines: Vec<&str> = content.lines().collect();
                if let Some(line) = lines.get(range.start.line as usize) {
                    if let Some(word) = get_word_at_position(line, range.start.character as usize) {
                        // Check if the word exists in workspace TypeStore
                        if store.lookup_fn_decl_str(&word).is_some()
                            || store.lookup_type_decl_str(&word).is_some()
                            || store.lookup_spec_decl_str(&word).is_some()
                        {
                            // Suggest auto-import
                            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                                title: format!("Auto-import '{}'", word),
                                kind: Some(CodeActionKind::QUICKFIX),
                                diagnostics: None,
                                edit: Some(WorkspaceEdit {
                                    changes: Some({
                                        let mut changes = std::collections::HashMap::new();
                                        changes.insert(
                                            uri.parse().unwrap(),
                                            vec![TextEdit {
                                                range: Range {
                                                    start: Position { line: 0, character: 0 },
                                                    end: Position { line: 0, character: 0 },
                                                },
                                                new_text: format!("use {}\n", word),
                                            }],
                                        );
                                        changes
                                    }),
                                    document_changes: None,
                                    change_annotations: None,
                                }),
                                command: None,
                                is_preferred: Some(false),
                                disabled: None,
                                data: None,
                            }));
                        }
                    }
                }
            }
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
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

                    if let Ok(uri_parsed) = uri.parse::<Uri>() {
                        self.client
                            .publish_diagnostics(
                                uri_parsed,
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
                    if let Ok(uri_parsed) = uri.parse::<Uri>() {
                        self.client
                            .publish_diagnostics(uri_parsed, vec![], None)
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

/// Extract document symbols from source code using the compiler's Indexer + Database
fn extract_document_symbols(content: &str) -> Vec<DocumentSymbol> {
    use auto_lang::database::{Database, FragKind};
    use auto_lang::indexer::Indexer;
    use auto_val::AutoStr;

    let mut symbols = Vec::new();

    // Parse the code
    let mut parser = auto_lang::Parser::from(content);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(_) => {
            // Even on parse error, try to extract symbols from what we have
            // The parser may have partially populated the AST
            return symbols;
        }
    };

    // Create a Database and index the AST
    let mut db = Database::new();
    let file_id = db.insert_source("document.at", AutoStr::from(content));
    let mut indexer = Indexer::new(&mut db);
    let _ = indexer.index_ast(&ast, file_id);

    // Convert fragments to DocumentSymbols
    for frag_id in db.get_fragments_in_file(file_id) {
        if let Some(meta) = db.get_fragment_meta(&frag_id) {
            let kind = match meta.kind {
                FragKind::Function => SymbolKind::FUNCTION,
                FragKind::Struct => SymbolKind::STRUCT,
                FragKind::Enum => SymbolKind::ENUM,
                FragKind::Const => SymbolKind::CONSTANT,
                FragKind::Spec => SymbolKind::INTERFACE,
                FragKind::Impl => SymbolKind::METHOD,
            };

            let line = meta.span.line.saturating_sub(1) as u32;
            let col = meta.span.column.saturating_sub(1) as u32;

            // Find the line content to determine range
            let lines: Vec<&str> = content.lines().collect();
            let line_len = lines.get(line as usize).map(|l| l.len() as u32).unwrap_or(0);

            symbols.push(DocumentSymbol {
                name: meta.name.to_string(),
                detail: Some(format!("{:?}", meta.kind).to_lowercase()),
                kind,
                tags: None,
                deprecated: None,
                range: Range {
                    start: Position { line, character: 0 },
                    end: Position { line, character: line_len },
                },
                selection_range: Range {
                    start: Position { line, character: col },
                    end: Position { line, character: col + meta.name.len() as u32 },
                },
                children: None,
            });
        }
    }

    symbols
}

/// Get the word at the given cursor position
fn get_word_at_position(line: &str, cursor: usize) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();

    if cursor > chars.len() {
        return None;
    }

    // Find the start of the word
    let mut start = cursor;
    while start > 0 && is_identifier_char(chars[start - 1]) {
        start -= 1;
    }

    // Find the end of the word
    let mut end = cursor;
    while end < chars.len() && is_identifier_char(chars[end]) {
        end += 1;
    }

    if start < end {
        Some(chars[start..end].iter().collect())
    } else {
        None
    }
}

/// Check if a character is part of an identifier
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
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
