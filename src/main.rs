
use tokio::{io::BufStream, net::TcpListener};
use tracing:: info;


mod request;

static DEFAULT_PORT: &str = "8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_PORT.to_string())
        .parse()?;

    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();

        info!("Listening on port {}", listener.local_addr()?);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let mut stream = BufStream::new(stream);

            // spawn task for async behavior (noBlock)
            tokio::spawn(async move {
                info!(?peer_addr, "new connection");

                match request::parse_request(&mut stream).await {
                    Ok(request) => info!(?request, "successful request"),
                    Err(err) => {
                        info!(?err, "failure request");
                    }
                    }

            });
        }
}