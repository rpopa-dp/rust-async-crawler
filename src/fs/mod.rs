#[derive(Debug)]
pub struct Request {
  pub path: String,
}

pub struct RequestBuilder {
  path: Option<String>,
}

pub fn request() -> RequestBuilder {
  RequestBuilder {
    path: None,
  }
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

// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Response {
  pub content: Vec<u8>,
}

pub struct ResponseBuilder {
  content: Option<Vec<u8>>,
}

pub fn response() -> ResponseBuilder {
  ResponseBuilder {
    content: None,
  }
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
