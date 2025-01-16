use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProfileRequest {
    pub user_id: String,
    pub bio: String,
    pub visibility: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutProfileRequest {
    pub bio: String,
    pub visibility: String,
}