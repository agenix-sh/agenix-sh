#[tokio::main]
async fn main() {
    if let Err(error) = agx::run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
