use std::collections::HashMap;
use tokio::io::{AsyncBufRead, AsyncBufReadExt};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Method {
    Get,
}

// TryFrom enables TryInto
// If I have a &str, I know how to attempt converting it into a Method
impl TryFrom<&str> for Method {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "GET" => Ok(Method::Get),
            message => Err(anyhow::anyhow!("Unknown method: {message}")),
        }
    }
}

pub async fn parse_request(mut stream: impl AsyncBufRead + Unpin) -> anyhow::Result<Request> {
    // reads the request line
    let mut line_buffer = String::new();
    // socket still waits for bytes, but thread isn’t parked doing nothing
    stream.read_line(&mut line_buffer).await?;

    // splits request line on ASCII whitespaces
    let mut parts = line_buffer.split_whitespace();

    // If you implement TryFrom<A> for B, the compiler automatically gives you TryInto<B> for A
    // extract values
    let method: Method = parts
        .next()
        .ok_or(anyhow::anyhow!("missing method"))
        .and_then(TryInto::try_into)?;

    let path: String = parts
        .next()
        .ok_or(anyhow::anyhow!("missing path"))
        .map(Into::into)?;

    let mut headers = HashMap::new();

    // read header lines until blank
    loop {
        line_buffer.clear();
        stream.read_line(&mut line_buffer).await?;

        if line_buffer.is_empty() || line_buffer == "\n" || line_buffer == "\r\n" {
            break;
        }

        // parse header lines
        let mut comps = line_buffer.split(":");
        let key = comps.next().ok_or(anyhow::anyhow!("missing header name"))?;
        let value = comps
            .next()
            .ok_or(anyhow::anyhow!("missing header value"))?
            .trim();

        headers.insert(key.to_string(), value.to_string());
    }

    // returns parsed request
    Ok(Request {
        method,
        path,
        headers,
    })
}
