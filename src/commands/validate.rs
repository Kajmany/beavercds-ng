use simplelog::*;
use std::path::Path;
use std::process::exit;

use crate::configparser::{get_challenges, get_config, get_profile_deploy};

pub fn run() {
    info!("validating config...");
    let config = match get_config() {
        Ok(c) => c,
        Err(err) => {
            error!("{err:#}");
            exit(1);
        }
    };
    info!("  config ok!");

    info!("validating challenges...");
    let chals = match get_challenges() {
        Ok(c) => c,
        Err(errors) => {
            for e in errors.iter() {
                error!("{e:#}");
            }
            exit(1);
        }
    };
    info!("  challenges ok!");

    // check global deploy settings for invalid challenges
    info!("validating deploy config...");
    for (profile_name, _pconfig) in config.profiles.iter() {
        // get em
        let deploy_challenges = match get_profile_deploy(profile_name) {
            Ok(d) => &d.challenges,
            Err(err) => {
                error!("{err:#}");
                exit(1);
            }
        };

        // check em
        let missing: Vec<_> = deploy_challenges
            .keys()
            .filter_map(
                // invert match to filter for challenges that *dont* match
                |path| match chals.iter().find(|c| c.directory == Path::new(path)) {
                    Some(_) => None,
                    None => Some(path),
                },
            )
            .collect();
        if missing.len() > 0 {
            error!(
                "Deploy settings for profile '{profile_name}' has challenges that do not exist:"
            );
            missing.iter().for_each(|path| error!("  - {path}"));
            exit(1)
        }
    }
    info!("  deploy ok!")
}
