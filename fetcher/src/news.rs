//! Articles d'actualité depuis Spaceflight News API.

use crate::io;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

const NEWS_URL: &str =
    "https://api.spaceflightnewsapi.net/v4/articles/?search=starship&limit=30&ordering=-published_at";

#[derive(Deserialize)]
struct Page<T> {
    results: Vec<T>,
}

#[derive(Deserialize)]
struct RawArticle {
    id: u64,
    title: String,
    url: String,
    image_url: Option<String>,
    news_site: String,
    summary: String,
    published_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Article {
    id: u64,
    title: String,
    url: String,
    image: Option<String>,
    source: String,
    summary: String,
    published_at: String,
}

/// Les résumés de NASASpaceflight finissent par "The post X appeared first on ..." : on le retire.
fn clean_summary(s: &str) -> String {
    let s = match s.find("The post ") {
        Some(i) if s[i..].contains("appeared first") => &s[..i],
        _ => s,
    };
    s.replace("\r\n", " ").replace('\n', " ").trim().to_string()
}

pub fn update(client: &io::Client, out: &Path) -> Result<()> {
    let page: Page<RawArticle> = io::get(client, NEWS_URL)?;
    let news: Vec<Article> = page
        .results
        .into_iter()
        .map(|a| Article {
            id: a.id,
            title: a.title.trim().to_string(),
            url: a.url,
            image: a.image_url.filter(|u| !u.is_empty()),
            source: a.news_site,
            summary: clean_summary(&a.summary),
            published_at: a.published_at,
        })
        .collect();
    anyhow::ensure!(!news.is_empty(), "aucun article reçu, on garde les anciennes données");
    io::write_json(&out.join("data/news.json"), &news)?;
    println!("📰 {} articles d'actu", news.len());
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_nettoye() {
        let s = "Starship is set to…\r\nThe post Launch Preview appeared first on NASASpaceFlight.com.";
        assert_eq!(clean_summary(s), "Starship is set to…");
    }
}
