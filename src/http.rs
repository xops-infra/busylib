use crate::prelude::EnhancedUnwrap;

pub type ReqwestError = reqwest::Error;
pub type ReqwestClient = reqwest::Client;

pub fn default_reqwest_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwp()
}
