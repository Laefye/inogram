mod db;
mod models;
mod services;
mod api;
mod random;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    api::run().await;
}