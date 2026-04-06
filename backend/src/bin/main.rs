use backend::app::App;
use dotenv::dotenv;
use loco_rs::cli;
use migration::Migrator;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    dotenv().ok();

    cli::main::<App, Migrator>().await
}
