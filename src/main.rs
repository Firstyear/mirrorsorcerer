
use tracing::info;



#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Hello, world!");
}
