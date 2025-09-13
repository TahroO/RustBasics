use tokio::{io::BufStream, net::TcpListener};
use tracing::info;

mod request;
mod response;

static DEFAULT_PORT: &str = "8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt::init();

    // define port number
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
                Ok(request) => {
                    info!(?request, "successful request");
                    // define response html in ok scope
                    let response = response::Response::from_html(
                        response::Status::NotFound,
                        include_str!("../static/404/404.html"),
                    );
                    if let Err(e) = response.write(&mut stream).await {
                        info!(?e, "failed writing response");
                    }
                }
                Err(err) => {
                    info!(?err, "failure request");
                    // define response html in err scope
                    let response = response::Response::from_html(
                        response::Status::NotFound,
                        include_str!("../static/404/404.html"),
                    );
                    // send html response to client
                    response.write(&mut stream).await.unwrap();
                }
            }
        });
    }
}
