use color_eyre::eyre::OptionExt;
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::Duration;

const TICK_FPS: f64 = 60.0;

#[derive(Clone, Debug)]
pub enum AppEvent {
    Tick,
    Quit,
    Crossterm(crossterm::event::Event),
}

pub struct EventHandler {
    sender: mpsc::UnboundedSender<AppEvent>,
    receiver: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone());
        tokio::spawn(async { actor.run().await });
        Self { sender, receiver }
    }

    pub async fn next(&mut self) -> color_eyre::Result<AppEvent> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Failed to receive event")
    }

    pub fn send(&mut self, app_event: AppEvent) {
        let _ = self.sender.send(app_event);
    }
}

struct EventTask {
    sender: mpsc::UnboundedSender<AppEvent>,
}

impl EventTask {
    fn new(sender: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self { sender }
    }

    async fn run(self) -> color_eyre::Result<()> {
        let tick_rate = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut reader = crossterm::event::EventStream::new();
        let mut tick = tokio::time::interval(tick_rate);
        loop {
            let tick_delay = tick.tick();
            let crossterm_event = reader.next().fuse();
            tokio::select! {
                _ = self.sender.closed() => {
                    break;
                }
                _ = tick_delay => {
                    self.send(AppEvent::Tick)
                }
                Some(Ok(event)) = crossterm_event => {
                    self.send(AppEvent::Crossterm(event));
                }

            };
        }
        Ok(())
    }

    fn send(&self, event: AppEvent) {
        let _ = self.sender.send(event);
    }
}
