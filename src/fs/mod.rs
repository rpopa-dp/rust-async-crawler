use async_std::fs::File;
use async_std::io::ReadExt;
use futures::channel::mpsc::{UnboundedSender, UnboundedReceiver};
use futures::{StreamExt, SinkExt};

use crate::result::Result;

#[derive(Debug)]
pub struct Request {
  pub path: String,
}

pub struct RequestBuilder {
  path: Option<String>,
}

impl RequestBuilder {
  pub fn with_path(mut self, path: String) -> Self {
    self.path = Some(path);
    self
  }

  pub fn build(self) -> Request {
    Request {
      path: self.path.expect("path is required"),
    }
  }
}

pub fn request() -> RequestBuilder {
  RequestBuilder {
    path: None,
  }
}

// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Response {
  pub content: Vec<u8>,
}

pub struct ResponseBuilder {
  content: Option<Vec<u8>>,
}

impl ResponseBuilder {
  pub fn with_content(mut self, content: Vec<u8>) -> Self {
    self.content = Some(content);
    self
  }

  pub fn build(self) -> Response {
    Response {
      content: self.content.expect("content is required"),
    }
  }
}

pub fn response() -> ResponseBuilder {
  ResponseBuilder {
    content: None,
  }
}

// -----------------------------------------------------------------------------
pub struct FetchResources {
  pub requests_rx: UnboundedReceiver<Request>,
  pub responses_tx: UnboundedSender<Response>,
}

impl FetchResources {
  pub async fn run(&mut self) {
    while let Some(request) = self.requests_rx.next().await {
      println!("------------");
      println!("{request:?}");

      match self.read_file(&request).await {
        Err(error) => eprintln!("ERROR: {error}"),
        Ok(response) => self.responses_tx.send(response).await.unwrap(),
      }
    }
  }

  async fn read_file(&self, request: &Request) -> Result<Response> {
    let mut file = File::open(&request.path).await?;
    let mut content = Vec::new();

    file.read_to_end(&mut content).await?;

    let response = response()
      .with_content(content)
      .build();

    Ok(response)
  }
}
