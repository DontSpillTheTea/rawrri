use rusqlite::{params, Connection, Result};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct DbManager {
    conn: Arc<Mutex<Connection>>,
}

impl DbManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let manager = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        manager.init_schema()?;
        Ok(manager)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // Metadata Cache
        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata_cache (
                asset_id TEXT PRIMARY KEY,
                duration_sec REAL,
                width INTEGER,
                height INTEGER,
                codec TEXT,
                has_audio INTEGER,
                is_corrupt INTEGER,
                stream_count INTEGER,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Jobs Queue
        conn.execute(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                job_type TEXT NOT NULL,
                asset_id TEXT NOT NULL,
                asset_path TEXT NOT NULL,
                status TEXT NOT NULL, -- 'pending', 'processing', 'completed', 'failed'
                progress REAL DEFAULT 0.0,
                error_message TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Observations (AI Results)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS observations (
                id TEXT PRIMARY KEY,
                asset_id TEXT NOT NULL,
                pair_id TEXT NOT NULL,
                start_time_sec REAL NOT NULL,
                end_time_sec REAL NOT NULL,
                pair_canonical_time_sec REAL NOT NULL,
                obs_type TEXT NOT NULL, -- 'vehicle', 'license_plate'
                data TEXT NOT NULL, -- JSON blob of observation details
                confidence REAL NOT NULL,
                bounding_box_json TEXT,
                is_user_confirmed INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        Ok(())
    }

    pub fn enqueue_job(&self, id: &str, job_type: &str, asset_id: &str, asset_path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO jobs (id, job_type, asset_id, asset_path, status) VALUES (?, ?, ?, ?, 'pending')",
            params![id, job_type, asset_id, asset_path],
        )?;
        Ok(())
    }

    pub fn get_next_pending_job(&self) -> Result<Option<JobRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, job_type, asset_id, asset_path FROM jobs WHERE status = 'pending' ORDER BY created_at ASC LIMIT 1"
        )?;
        let mut rows = stmt.query([])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(JobRecord {
                id: row.get(0)?,
                job_type: row.get(1)?,
                asset_id: row.get(2)?,
                asset_path: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn update_job_status(&self, id: &str, status: &str, progress: f64, error: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE jobs SET status = ?, progress = ?, error_message = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![status, progress, error, id],
        )?;
        Ok(())
    }

    pub fn save_metadata(&self, asset_id: &str, metadata: &crate::models::MediaMetadata) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO metadata_cache 
            (asset_id, duration_sec, width, height, codec, has_audio, is_corrupt, stream_count) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                asset_id,
                metadata.duration_sec,
                metadata.width,
                metadata.height,
                metadata.codec,
                if metadata.has_audio { 1 } else { 0 },
                if metadata.is_corrupt { 1 } else { 0 },
                metadata.stream_count as i64
            ],
        )?;
        Ok(())
    }

    pub fn get_metadata(&self, asset_id: &str) -> Result<Option<crate::models::MediaMetadata>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT duration_sec, width, height, codec, has_audio, is_corrupt, stream_count 
             FROM metadata_cache WHERE asset_id = ?"
        )?;
        let mut rows = stmt.query(params![asset_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(crate::models::MediaMetadata {
                duration_sec: row.get(0)?,
                width: row.get(1)?,
                height: row.get(2)?,
                codec: row.get(3)?,
                has_audio: row.get::<_, i32>(4)? != 0,
                is_corrupt: row.get::<_, i32>(5)? != 0,
                stream_count: row.get::<_, i64>(6)? as usize,
                creation_time: None, // ffprobe tags might need another table or JSON blob if we want to cache them
            }))
        } else {
            Ok(None)
        }
    }
}

pub struct JobRecord {
    pub id: String,
    pub job_type: String,
    pub asset_id: String,
    pub asset_path: String,
}
