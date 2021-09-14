use tokio::time::{sleep, Duration};

const CONTROLLER_SLEEP_DURATION: Duration = Duration::from_millis(5000);

#[derive(Debug)]
pub struct BrupopController {}

impl BrupopController {
    pub fn new() -> BrupopController {
        BrupopController {}
    }

    pub async fn run(&mut self) {
        loop {
            log::debug!(
                "Controller loop completed. Sleeping for {:?}.",
                CONTROLLER_SLEEP_DURATION
            );
            sleep(CONTROLLER_SLEEP_DURATION).await;
        }
    }
}
