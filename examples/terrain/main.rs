mod app;
use app::*;
use futures::executor::block_on;
mod terrain;
pub use terrain::*;

async fn async_main() {
	App::new(1024, 576).await.run();
}

fn main() {
	env_logger::init();
	block_on(async_main());
}
