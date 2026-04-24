use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use crate::db::DbManager;
use crate::metadata::extract_metadata;
use crate::analysis::AnalysisEngine;

pub struct JobWorker {
    db: Arc<DbManager>,
    app_handle: AppHandle,
    analysis_engine: Arc<AnalysisEngine>,
}

impl JobWorker {
    pub fn new(db: Arc<DbManager>, app_handle: AppHandle) -> Self {
        Self {
            db,
            app_handle,
            analysis_engine: Arc::new(AnalysisEngine::new()),
        }
    }

    pub async fn start(self: Arc<Self>) {
        println!("JobWorker: Starting background loop...");
        loop {
            match self.db.get_next_pending_job() {
                Ok(Some(job)) => {
                    println!("JobWorker: Processing job {} (type: {})", job.id, job.job_type);
                    self.process_job(job).await;
                }
                Ok(None) => {
                    // No jobs, sleep for a bit
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    eprintln!("JobWorker: Database error: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn process_job(&self, job: crate::db::JobRecord) {
        let _ = self.db.update_job_status(&job.id, "processing", 0.0, None);
        
        match job.job_type.as_str() {
            "metadata_extraction" => {
                match extract_metadata(&job.asset_path) {
                    Ok(metadata) => {
                        let _ = self.db.save_metadata(&job.asset_id, &metadata);
                        let _ = self.db.update_job_status(&job.id, "completed", 1.0, None);
                        let _ = self.app_handle.emit("metadata_extracted", metadata);
                    }
                    Err(e) => {
                        let _ = self.db.update_job_status(&job.id, "failed", 0.0, Some(&e));
                    }
                }
            }
            "ai_analysis" => {
                // Phase 2 placeholder
                let _ = self.db.update_job_status(&job.id, "completed", 1.0, None);
            }
            _ => {
                let _ = self.db.update_job_status(&job.id, "failed", 0.0, Some("Unknown job type"));
            }
        }
    }
}
