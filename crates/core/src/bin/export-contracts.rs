fn main() -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer(std::io::stdout().lock(), &workspace_core::codegen::render())?;
    Ok(())
}
