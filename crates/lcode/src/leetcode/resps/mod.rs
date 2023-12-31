pub mod checkin;
pub mod run_res;
pub mod submit_list;
pub mod user_data;

use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct TestInfo {
    #[serde(default)]
    pub interpret_id:          String,
    #[serde(default)]
    pub test_case:             String,
    #[serde(default)]
    pub interpret_expected_id: String,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
pub struct SubmitInfo {
    #[serde(default)]
    pub submission_id: u32,
}
