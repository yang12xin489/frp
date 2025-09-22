use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ProgressPayload {
    pub name: String,
    pub progress: u32,
}