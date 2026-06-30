mod engine;
mod collectors;
mod storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting Static-Memory...");
    Ok(())
}
