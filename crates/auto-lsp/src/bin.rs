use auto_lsp::Backend;
use tower_lsp::LspService;
use tower_lsp::Server;

#[tokio::main]
async fn main() {
    // NOTE: The auto-lang parser has debug println! statements.
    // On Windows, we can redirect these to NUL at startup.
    #[cfg(target_os = "windows")]
    {
        use std::fs::OpenOptions;
        

        // Open NUL device for discarding output
        if let Ok(nul) = OpenOptions::new().write(true).open("NUL") {
            
            use std::mem;

            // NOTE: This doesn't actually redirect stdout/stderr at the OS level.
            // The real fix is to remove println! statements from the parser.
            // For now, this is a placeholder showing intent.
            mem::forget(nul);
        }
    }

    // Create stdin/stdout transport
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    // Create and run the LSP server
    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}
