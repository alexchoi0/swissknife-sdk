use swissknife_ai_sdk::memory::{DuckDBMemory, MemoryConfig};

pub fn get_memory() -> Result<DuckDBMemory, Box<dyn std::error::Error>> {
    let db_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("secretary")
        .join("secretary.duckdb");

    let mem_config = MemoryConfig::new()
        .with_db_path(db_path.to_string_lossy());

    Ok(DuckDBMemory::new(mem_config)?)
}
