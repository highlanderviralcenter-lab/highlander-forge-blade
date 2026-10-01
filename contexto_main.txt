fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()?;

    runtime.block_on(async {
        highlander_forge_blade::logging::init_logging(
            highlander_forge_blade::logging::LogFormat::Human
        );

        #[cfg(feature = "tui")]
        highlander_forge_blade::ui::ratatui::run().await?;

        Ok(())
    })
}
