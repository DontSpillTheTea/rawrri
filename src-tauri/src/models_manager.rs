use std::path::{Path, PathBuf};

pub struct ModelManager {
    base_dir: PathBuf,
}

impl ModelManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let base_dir = app_data_dir.join("models");
        std::fs::create_dir_all(&base_dir).expect("failed to create models dir");
        Self { base_dir }
    }

    pub fn get_model_path(&self, model_name: &str) -> PathBuf {
        self.base_dir.join(format!("{}.onnx", model_name))
    }

    pub fn ensure_models(&self) -> Result<(), String> {
        // Placeholder for automated downloader
        // For now just check if they exist
        let required = vec!["yolov8n", "lprnet"];
        for model in required {
            let path = self.get_model_path(model);
            if !path.exists() {
                println!("Warning: Model {} missing at {:?}", model, path);
            }
        }
        Ok(())
    }
}
