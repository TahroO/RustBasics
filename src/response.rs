use maplit::hashmap;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    io::Cursor,
};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone)]

// generic data streams in body possible (files, sockets...)
pub struct Response<S: AsyncRead + Unpin> {
    pub status: Status,
    pub headers: HashMap<String, String>,
    pub body: S,
}

// represents response codes
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Status {
    NotFound,
}

// write response to generic output stream
impl<S: AsyncRead + Unpin> Response<S> {
    // build response
    pub fn status_and_headers(&self) -> String {
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            // join kv pairs with crlf for HTTP
            .join("\r\n");

        // form response 1. status {}, 2. headers + extra \r\n for end of headers
        format!("HTTP/1.1 {}\r\n{headers}\r\n\r\n", self.status)
    }

    // write output
    pub async fn write<O: AsyncWrite + Unpin>(mut self, stream: &mut O) -> anyhow::Result<()> {
        // send header section
        stream
            .write_all(self.status_and_headers().as_bytes())
            .await?;

        // stream body section (files, cursor...)
        tokio::io::copy(&mut self.body, stream).await?;

        // flush stream to empty it adn shutdown gracefully
        stream.flush().await?;
        stream.shutdown().await?;

        // return ok afterward
        Ok(())
    }
}

// allows printing a status using {} -- println!("{}", Status::NotFound)
impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::NotFound => write!(f, "404 Not Found"),
        }
    }
}

// in memory html response
impl Response<Cursor<Vec<u8>>> {
    pub fn from_html(status: Status, body: impl ToString) -> Self {
        // convert body into a dynamic array of bytes
        let bytes = body.to_string().into_bytes();

        // build headers
        let headers = hashmap! {
            "Content-Type".to_string() => "text/html".to_string(),
            "Content-Length".to_string() =>  bytes.len().to_string(),
        };

        // return rdy to send response
        Self {
            status,
            headers,
            // wrap bytes into cursor for stream utility
            body: Cursor::new(bytes),
        }
    }
}
