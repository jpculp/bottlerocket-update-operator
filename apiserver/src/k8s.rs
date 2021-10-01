use crate::error::{self, Result};
use kube::api::{Api, Patch, PatchParams, PostParams, ResourceExt};
use serde::{de::DeserializeOwned, Serialize};
use snafu::ResultExt;
use std::fmt::Debug;

const MAX_RETRIES: u32 = 3;
const BACKOFF_MS: u64 = 500;

/// Create or update an object in `api` with `data`'s name
pub async fn create_or_update<T>(api: &Api<T>, data: T, what: &str) -> Result<()>
where
    T: Clone + DeserializeOwned + Debug + kube::Resource + Serialize,
{
    let mut error = None;

    for _ in 0..MAX_RETRIES {
        match create_or_update_internal(api, data.clone(), what).await {
            Ok(()) => return Ok(()),
            Err(e) => error = Some(e),
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(BACKOFF_MS)).await;
    }
    match error {
        None => Ok(()),
        Some(error) => Err(error),
    }
}

async fn create_or_update_internal<T>(api: &Api<T>, data: T, what: &str) -> Result<()>
where
    T: Clone + DeserializeOwned + Debug + kube::Resource + Serialize,
{
    // If the data already exists, update it with the new one using a `Patch`. If not create a new one.
    match api.get(&data.name()).await {
        Ok(deployment) => {
            api.patch(
                &deployment.name(),
                &PatchParams::default(),
                &Patch::Merge(data),
            )
            .await
        }
        Err(_err) => api.create(&PostParams::default(), &data).await,
    }
    .context(error::Creation { what })?;

    Ok(())
}
