mod error;
mod logging;
mod server;

use logging::init_logger;
use tracing::{debug, error};

use crate::server::Server;

#[tokio::main]
async fn main() {
    if let Err(err) = init_logger("reverse_tcp_server") {
        println!("Error initializing logging: {:?}", err);
        return;
    }

    let s = Server::new().unwrap_or_else(|err| {
        error!(error = %err, "server initialization failed");
        std::process::exit(1);
    });
    debug!(server = ?s, "server initialized");
    s.start().await.unwrap();
}
