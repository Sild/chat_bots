use std::fs;
use std::path::PathBuf;
use csv::WriterBuilder;

fn ensure_storage(path: &PathBuf) -> anyhow::Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        // Create with header for readability
        let mut wtr = WriterBuilder::new().from_path(path)?;
        // header
        let _ = wtr.write_record(["user_id", "user_name", "number"])?;
        wtr.flush()?;
    }
    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub storage_path: PathBuf,
}

impl AppState {
    pub async fn new(storage_path: PathBuf) -> anyhow::Result<Self> {
        ensure_storage(&storage_path)?;
        Ok(AppState { storage_path })
    }
}
