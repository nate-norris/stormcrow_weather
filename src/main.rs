mod logger;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init_logger(None);
    logger::info("Weather started");

    logger::error("sample error", None);
    Ok(())
}
