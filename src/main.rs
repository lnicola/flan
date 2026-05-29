use axum::Router;
use clap::Parser;
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[derive(Parser, Debug)]
#[command(name = "flan", author, version, about = "Simple static file server")]
struct Args {
    /// Directory to serve. Defaults to the current directory.
    #[arg(default_value = ".")]
    directory: PathBuf,

    /// IP address to listen on.
    #[arg(short, long, default_value = "127.0.0.1")]
    address: IpAddr,

    /// Port to listen on.
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

async fn bind_to_port(address: IpAddr, mut port: u16) -> Result<TcpListener, Error> {
    loop {
        let socket_addr = SocketAddr::new(address, port);
        match TcpListener::bind(&socket_addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == ErrorKind::AddrInUse => {
                if port == u16::MAX {
                    return Err(Error::new(ErrorKind::AddrInUse, "No available ports found"));
                }
                port += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let canonical_path = args.directory.canonicalize()?;
    println!("Serving directory: {}", canonical_path.display());
    let serve_dir = ServeDir::new(&canonical_path);

    let listener = bind_to_port(args.address, args.port).await?;
    println!("Server running at http://{}", listener.local_addr()?);

    let app = Router::new().fallback_service(serve_dir);
    axum::serve(listener, app).await?;
    Ok(())
}
