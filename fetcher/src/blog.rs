//! Articles de blog « récap de vol » générés automatiquement.

use crate::launches::Launch;
use anyhow::{Context, Result};
use std::{fs, path::Path};

fn status_fr(status: &str) -> &'static str {
    match status {
        "Success" => "succès",
        "Failure" => "échec",
        "Partial Failure" => "échec partiel",
        _ => "à venir",
    }
}

/// Une chaîne JSON est aussi une chaîne YAML valide : pratique pour échapper le frontmatter.
fn yaml(s: &str) -> String {
    serde_json::to_string(s).unwrap()
}

/// Crée un article de récap pour chaque vol intégré terminé. Un fichier existant
/// n'est JAMAIS écrasé : tu peux donc réécrire/enrichir l'article à la main.
pub fn write_flight_posts(launches: &[Launch], blog_dir: &Path) -> Result<u32> {
    fs::create_dir_all(blog_dir)?;
    let mut created = 0;
    for l in launches.iter().filter(|l| l.is_done() && !l.prototype) {
        let path = blog_dir.join(format!("{}.md", l.slug));
        if path.exists() {
            continue;
        }
        let title = format!("{} : {}", l.mission, status_fr(&l.status));
        let mut md = String::new();
        md += "---\n";
        md += &format!("title: {}\n", yaml(&title));
        md += &format!("date: {}\n", l.net);
        md += &format!(
            "description: {}\n",
            yaml(&format!("Récap automatique du vol {} de {} ({}).", l.mission, l.vehicle, status_fr(&l.status)))
        );
        md += &format!("tags: [\"vol\", {}, \"auto\"]\n", yaml(&l.vehicle));
        md += "auto: true\n";
        if let Some(img) = &l.image {
            md += &format!("image: {}\n", yaml(img));
        }
        md += "---\n\n";
        md += "> 🤖 Article généré automatiquement par le robot du site à partir de Launch Library 2.\n";
        md += "> Il ne sera jamais écrasé : n'hésite pas à le réécrire avec ton analyse !\n\n";
        md += &format!("- **Résultat :** {}\n", status_fr(&l.status));
        md += &format!("- **Véhicule :** {}\n", l.vehicle);
        md += &format!("- **Pas de tir :** {} ({})\n", l.pad, l.location);
        if let Some(o) = &l.orbit {
            md += &format!("- **Orbite visée :** {o}\n");
        }
        if let Some(d) = &l.description {
            md += &format!("\n## La mission\n\n{d}\n");
        }
        if let Some(f) = &l.fail_reason {
            md += &format!("\n## Ce qui s'est passé\n\n{f}\n");
        }
        fs::write(&path, md).with_context(|| format!("écriture de {}", path.display()))?;
        created += 1;
    }
    Ok(created)
}

