mod app;
use app::*;
use futures::executor::block_on;

async fn async_main() {
	App::new(1280, 720).await.run();
}

fn main() {
	env_logger::init();
	block_on(async_main());
}
