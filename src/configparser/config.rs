use anyhow::{Context, Result};
use fully_pub::fully_pub;
use serde::{Deserialize, Serialize};
use simplelog::*;
use std::collections::HashMap as Map;
use std::fs;

pub fn parse() -> Result<RcdsConfig> {
    debug!("trying to parse rcds.yaml");

    let contents = fs::read_to_string("rcds.yaml").with_context(|| "failed to read rcds.yaml")?;
    let parsed = serde_yaml::from_str(&contents).with_context(|| "failed to parse rcds.yaml")?;

    trace!("got config: {parsed:#?}");

    Ok(parsed)
}

//
// ==== Structs for rcds.yaml parsing ====
//

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[fully_pub]
struct RcdsConfig {
    flag_regex: String,
    registry: Registry,
    defaults: Defaults,
    deploy: Map<String, ProfileDeploy>,
    profiles: Map<String, ProfileConfig>,
    points: Vec<ChallengePoints>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[fully_pub]
struct Registry {
    domain: String,
    build: UserPass,
    cluster: UserPass,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[fully_pub]
struct UserPass {
    user: String,
    pass: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[fully_pub]
struct Resource {
    cpu: i64,
    memory: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[fully_pub]
struct Defaults {
    difficulty: i64,
    resources: Resource,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[fully_pub]
struct ProfileDeploy {
    #[serde(flatten)]
    challenges: Map<String, bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[fully_pub]
struct ProfileConfig {
    // deployed_challenges: HashMap<String, bool>,
    frontend_url: String,
    frontend_token: Option<String>,
    challenges_domain: String,
    kubeconfig: Option<String>,
    kubecontext: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[fully_pub]
struct ChallengePoints {
    difficulty: i64,
    min: i64,
    max: i64,
}
