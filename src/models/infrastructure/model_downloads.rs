use std::io::Read;
use std::time::Duration;

use crate::models::domain::model::RemoteFile;
use crate::models::domain::model_source::ModelSource;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(20);

/// Files of models fetched over HTTPS from the address each one names.
pub struct ModelDownloads;

impl ModelSource for ModelDownloads {
    fn download(&self, file: &RemoteFile) -> Result<Box<dyn Read>, String> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_connect(Some(CONNECTION_TIMEOUT))
            .timeout_recv_response(Some(CONNECTION_TIMEOUT))
            .build()
            .into();
        let response = agent
            .get(file.url)
            .call()
            .map_err(|error| error.to_string())?;
        Ok(Box::new(response.into_body().into_reader()))
    }
}
