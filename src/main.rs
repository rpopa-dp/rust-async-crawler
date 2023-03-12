mod fs;
mod result;

use futures::channel::mpsc::{self, UnboundedSender, UnboundedReceiver};
use futures::{self, executor, StreamExt, SinkExt};
use crate::result::Result;

#[derive(Debug)]
struct Item {
  name: String,
  year: i32,
}

// -----------------------------------------------------------------------------
struct SeedRequests {
  requests_tx: UnboundedSender<fs::Request>,
}

impl SeedRequests {
  async fn run(&mut self) {
    let request = fs::request()
      .with_path("fixtures/1.txt".into())
      .build();

    self.requests_tx.send(request).await.unwrap();
  }
}

// -----------------------------------------------------------------------------
struct ExtractItems {
  responses_rx: UnboundedReceiver<fs::Response>,
  requests_tx: UnboundedSender<fs::Request>,
  items_tx: UnboundedSender<Item>,
}

impl ExtractItems {
  async fn run(&mut self) {
    while let Some(response) = self.responses_rx.next().await {
      println!("------------");
      println!("{response:?}");

      match self.process_response(&response).await {
        Err(error) => eprintln!("ERROR: {error}"),
        Ok(_) => {},
      }
    }
  }

  async fn process_response(&mut self, response: &fs::Response) -> Result<()> {
    let lines = self.parse_lines(&response)?;
    let item = self.parse_item(&lines)?;

    self.items_tx.send(item).await?;

    if let Some(request) = self.parse_request(&lines)? {
      self.requests_tx.send(request).await?;
    }

    Ok(())
  }

  fn parse_lines(&self, response: &fs::Response) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    let mut slice = &response.content[..];

    while slice.len() > 0 {
      let pos = slice.iter()
        .position(|&b| b == '\n' as u8)
        .ok_or("could not parse line")?;
      let line = std::str::from_utf8(&slice[..pos])?.to_string();

      lines.push(line);
      slice = &slice[pos + 1..];
    }

    Ok(lines)
  }

  fn parse_item(&self, lines: &Vec<String>) -> Result<Item> {
    let name = lines[0].clone();
    let year = lines[1].parse::<i32>()?;
    let item = Item { name, year };

    return Ok(item);
  }

  fn parse_request(&self, lines: &Vec<String>) -> Result<Option<fs::Request>> {
    let next_page = lines[2].parse::<i32>()?;

    if next_page == 0 {
      Ok(None)
    } else {
      let request = fs::request()
        .with_path(format!("fixtures/{next_page}.txt"))
        .build();

      Ok(Some(request))
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
  let (requests_tx, requests_rx) = mpsc::unbounded::<fs::Request>();
  let (responses_tx, responses_rx) = mpsc::unbounded::<fs::Response>();
  let (items_tx, items_rx) = mpsc::unbounded::<Item>();

  let mut seed_requests = SeedRequests {
    requests_tx: requests_tx.clone(),
  };
  let mut fetch_resources = fs::FetchResources {
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
