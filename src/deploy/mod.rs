pub mod frontend;
pub mod kubernetes;
pub mod s3;

use anyhow::{anyhow, bail, Context, Error, Result};
use itertools::Itertools;
use k8s_openapi::api::core::v1::Secret;
use kube::api::ListParams;
use std::env::current_exe;
use tracing::{debug, error, info, trace, warn};

use crate::clients::kube_client;
use crate::cluster_setup;
use crate::configparser::config::ProfileConfig;

/// check to make sure that the needed ingress charts are deployed and running
pub async fn check_setup(profile: &ProfileConfig) -> Result<()> {
    let kube = kube_client(profile).await?;
    let secrets: kube::Api<Secret> = kube::Api::namespaced(kube, cluster_setup::INGRESS_NAMESPACE);

    let all_releases = secrets
        .list_metadata(&ListParams::default().labels("owner=helm"))
        .await?;

    // pull helm release version from secret label
    macro_rules! helm_version {
        ($s:ident) => {
            $s.get("version")
                .unwrap_or(&"".to_string())
                .parse::<usize>()
                .unwrap_or(0)
        };
    }
    let expected_charts = ["ingress-nginx", "cert-manager", "external-dns"];
    let latest_releases = expected_charts
        .iter()
        .map(|chart| {
            // pick latest release
            all_releases
                .iter()
                .map(|r| r.metadata.labels.as_ref().unwrap())
                .filter(|rl| rl.get("name") == Some(&chart.to_string()))
                .max_by(|a, b| helm_version!(a).cmp(&helm_version!(b)))
        })
        .collect_vec();

    enum ChartFailure {
        Missing(String),
        DeploymentFailed(String),
    }

    // make sure all releases are present and deployed successfully
    let missing = latest_releases
        .iter()
        .zip(expected_charts)
        .filter_map(|(r, c)| {
            // is label status=deployed ?
            if r.is_none() {
                return Some(ChartFailure::Missing(c.to_string()));
            }

            if r.unwrap().get("status") == Some(&"deployed".to_string()) {
                // all is good
                None
            } else {
                Some(ChartFailure::DeploymentFailed(c.to_string()))
            }
        })
        .collect_vec();

    if !missing.is_empty() {
        // if any errors are present, collect/reduce them all into one error via
        // anyhow context() calls.
        //
        // TODO: this should probably be returning Vec<Error> instead of a
        // single Error chain. should this be in run() to present errors there
        // instead of chaining and returning one combined Error here?
        #[allow(clippy::manual_try_fold)] // need to build the Result ourselves
        missing
            .iter()
            .fold(Err(anyhow!("")), |e, reason| match reason {
                ChartFailure::Missing(c) => e.with_context(|| {
                    format!(
                        "chart {}/{c} is not deployed",
                        cluster_setup::INGRESS_NAMESPACE
                    )
                }),
                ChartFailure::DeploymentFailed(c) => e.with_context(|| {
                    format!(
                        "chart {}/{c} is in a failed state",
                        cluster_setup::INGRESS_NAMESPACE
                    )
                }),
            })
            .with_context(|| {
                format!(
                    "cluster has not been set up with needed charts (run `{} cluster-setup`)",
                    current_exe()
                        .unwrap()
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                )
            })
    } else {
        Ok(())
    }
}
