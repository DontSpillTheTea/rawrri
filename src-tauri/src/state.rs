use std::sync::Mutex;

use crate::settings::AppSettings;

#[derive(Default)]
pub struct AppState {
    pub settings: Mutex<AppSettings>,
}
