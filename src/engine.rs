use tokio::sync::mpsc;
use crate::collectors::Event;

pub struct Engine {
    receiver: mpsc::Receiver<Event>,
}

impl Engine {
    pub fn new(receiver: mpsc::Receiver<Event>) -> Self {
        Self { receiver }
    }

    pub async fn run(&mut self) {
        println!("Engine is running, waiting for events...");
        while let Some(event) = self.receiver.recv().await {
            println!("Received event: [{}]: {}", event.event_type, event.content);
            // Logic to save to storage will go here
        }
    }
}
