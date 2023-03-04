use futures::channel::mpsc::{self, UnboundedSender, UnboundedReceiver};
use futures::{self, executor, StreamExt, SinkExt};
use async_std::fs::File;
use async_std::io::ReadExt;

// -----------------------------------------------------------------------------
#[derive(Debug)]
struct Request {
  path: String,
}

#[derive(Debug)]
struct Response {
  content: Vec<u8>,
}

#[derive(Debug)]
struct Item {
  name: String,
  year: i32,
}

// -----------------------------------------------------------------------------
struct SeedRequests {
  requests_tx: UnboundedSender<Request>,
}

impl SeedRequests {
  async fn run(&mut self) {
    let request = Request {
      path: format!("fixtures/1.txt"),
    };

    self.requests_tx.send(request).await.unwrap();
  }
}

// -----------------------------------------------------------------------------
struct FetchResources {
  requests_rx: UnboundedReceiver<Request>,
  responses_tx: UnboundedSender<Response>,
}

impl FetchResources {
  async fn run(&mut self) {
    while let Some(request) = self.requests_rx.next().await {
      println!("------------");
      println!("{request:?}");

      let mut file = File::open(request.path).await.unwrap();
      let mut content = Vec::new();

      file.read_to_end(&mut content).await.unwrap();

      let response = Response { content };

      self.responses_tx.send(response).await.unwrap();
    }
  }
}

// -----------------------------------------------------------------------------
struct ExtractItems {
  responses_rx: UnboundedReceiver<Response>,
  requests_tx: UnboundedSender<Request>,
  items_tx: UnboundedSender<Item>,
}

impl ExtractItems {
  async fn run(&mut self) {
    while let Some(response) = self.responses_rx.next().await {
      println!("------------");
      println!("{response:?}");

      let slice = &response.content[..];
      let pos = slice.iter().position(|&b| b == '\n' as u8).unwrap();
      let name = std::str::from_utf8(&slice[..pos]).unwrap().to_string();

      let slice = &slice[pos + 1..];
      let pos = slice.iter().position(|&b| b == '\n' as u8).unwrap();
      let year = std::str::from_utf8(&slice[..pos]).unwrap()
        .parse::<i32>().unwrap();

      let slice = &slice[pos + 1..];
      let pos = slice.iter().position(|&b| b == '\n' as u8).unwrap();
      let next_page = std::str::from_utf8(&slice[..pos]).unwrap()
        .parse::<i32>().unwrap();

      let item = Item { name, year };

      self.items_tx.send(item).await.unwrap();

      if next_page != 0 {
        let request = Request {
          path: format!("fixtures/{next_page}.txt"),
        };
        self.requests_tx.send(request).await.unwrap();
      }
    }
  }
}

// -----------------------------------------------------------------------------
struct DumpItems {
  items_rx: UnboundedReceiver<Item>,
}

impl DumpItems {
  async fn run(&mut self) {
    while let Some(item) = self.items_rx.next().await {
      println!("------------");
      println!("{} {}", item.name, item.year);
    }
  }
}

// -----------------------------------------------------------------------------
async fn async_main() {
  let (requests_tx, requests_rx) = mpsc::unbounded::<Request>();
  let (responses_tx, responses_rx) = mpsc::unbounded::<Response>();
  let (items_tx, items_rx) = mpsc::unbounded::<Item>();

  let mut seed_requests = SeedRequests {
    requests_tx: requests_tx.clone(),
  };
  let mut fetch_resources = FetchResources {
    requests_rx,
    responses_tx: responses_tx.clone(),
  };
  let mut extract_items = ExtractItems {
    responses_rx,
    requests_tx: requests_tx.clone(),
    items_tx: items_tx.clone(),
  };
  let mut dump_items = DumpItems {
    items_rx,
  };

  futures::join!(
    seed_requests.run(),
    fetch_resources.run(),
    extract_items.run(),
    dump_items.run(),
  );
}

fn main() {
  executor::block_on(async_main())
}
