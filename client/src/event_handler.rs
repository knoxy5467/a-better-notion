use std::io;

use futures::{FutureExt, StreamExt};
use tokio::{sync::mpsc, task::JoinHandle};

#[derive(Clone, Debug)]
pub enum Event {
    Error,
    Tick,
    Term(crossterm::event::Event),
}

#[derive(Debug)]
pub struct EventHandler {
    _tx: mpsc::UnboundedSender<Event>,
    rx: mpsc::UnboundedReceiver<Event>,
    _task: Option<JoinHandle<()>>,
}

impl EventHandler {
    pub fn new() -> Self {
        let tick_rate = std::time::Duration::from_millis(500);

        let (tx, rx) = mpsc::unbounded_channel();
        let _tx = tx.clone();

        let task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                let delay = interval.tick();
                let crossterm_event = reader.next().fuse();
				tokio::select! {
					maybe_event = crossterm_event => {
						match maybe_event {
						Some(Ok(evt)) => tx.send(Event::Term(evt)).unwrap(),
						Some(Err(_)) => {
							tx.send(Event::Error).unwrap();
						}
						None => {},
						}
					},
					_ = delay => {
						tx.send(Event::Tick).unwrap();
					},
                }
            }
        });

        Self {
            _tx,
            rx,
            _task: Some(task),
        }
    }

    pub async fn next(&mut self) -> io::Result<Event> {
        self.rx
            .recv()
            .await
            .ok_or(io::Error::other::<String>("tokio channel error".into()))
    }
}
