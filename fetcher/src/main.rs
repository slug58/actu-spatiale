//! Robot de collecte d'ActuSpatiale : interroge des API publiques et écrit des
//! fichiers JSON / Markdown que le site Astro consomme au build.
//!
//! Usage : actu-spatiale-fetcher [--out <dossier src du site>]   (défaut : site/src)

mod blog;
mod io;
mod launches;
mod news;
mod social;

use anyhow::{Context, Result};
use std::{fs, path::PathBuf, time::Duration};

const USER_AGENT: &str = concat!("actu-spatiale-bot/", env!("CARGO_PKG_VERSION"));

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let out = match args.iter().position(|a| a == "--out") {
        Some(i) => PathBuf::from(args.get(i + 1).context("--out attend un chemin")?),
        None => PathBuf::from("site/src"),
    };
    fs::create_dir_all(out.join("data"))?;

    let client = io::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()?;

    // Chaque source est indépendante : si l'une tombe, les autres sont quand même
    // mises à jour, et le fichier de la source en panne n'est pas écrasé.
    let results = [
        ("lancements", launches::update(&client, &out)),
        ("actualités", news::update(&client, &out)),
        ("réseaux sociaux", social::update(&client, &out)),
    ];
    let mut failed = false;
    for (what, r) in results {
        if let Err(e) = r {
            eprintln!("⚠️  {what} : {e:#}");
            failed = true;
        }
    }
    if failed {
        std::process::exit(1);
    }
    Ok(())
}
