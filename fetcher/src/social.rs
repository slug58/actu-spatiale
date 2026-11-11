//! Réseaux sociaux : posts Bluesky de comptes spatiaux (API publique, sans clé).
//!
//! X.com n'est pas utilisé (API de lecture payante, scraping interdit par les CGU).
//! Reddit non plus : r/SpaceX renvoie 403 aux clients non authentifiés.

use crate::io;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};

/// Comptes suivis (vérifiés actifs le 2026-11-11). Ajoute/retire librement.
pub const BLUESKY_HANDLES: &[&str] = &[
    "nasaspaceflight.com",
    "spaceflightnow.com",
    "space.com",
    "esa.int",
    "scottmanley.bsky.social",
    "jfoust.bsky.social",
    "planetarysociety.bsky.social",
];

const MAX_POSTS: usize = 40;
const MAX_PER_AUTHOR: usize = 8;
const MAX_AGE_DAYS: i64 = 14;

// ───────────────────────────── Format brut Bluesky ─────────────────────────────

#[derive(Deserialize)]
struct Feed {
    feed: Vec<FeedItem>,
}

#[derive(Deserialize)]
struct FeedItem {
    post: Post,
    /// Présent quand l'élément est un repost : on ne garde que les posts originaux.
    reason: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Post {
    uri: String,
    author: Author,
    record: Record,
    embed: Option<Embed>,
    like_count: Option<u64>,
    repost_count: Option<u64>,
    indexed_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Author {
    handle: String,
    display_name: Option<String>,
    avatar: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    #[serde(default)]
    text: String,
    created_at: Option<String>,
}

/// Les différentes vues d'embed (images, lien externe, vidéo, citation + média).
#[derive(Deserialize)]
struct Embed {
    images: Option<Vec<EmbedImage>>,
    external: Option<External>,
    thumbnail: Option<String>,
    media: Option<Box<Embed>>,
}

#[derive(Deserialize)]
struct EmbedImage {
    thumb: Option<String>,
    fullsize: Option<String>,
}

#[derive(Deserialize)]
struct External {
    thumb: Option<String>,
}

impl Embed {
    fn image(&self) -> Option<String> {
        self.images
            .as_ref()
            .and_then(|imgs| imgs.first())
            .and_then(|i| i.fullsize.clone().or_else(|| i.thumb.clone()))
            .or_else(|| self.external.as_ref().and_then(|e| e.thumb.clone()))
            .or_else(|| self.thumbnail.clone())
            .or_else(|| self.media.as_ref().and_then(|m| m.image()))
    }
}

// ───────────────────────────── Format écrit pour le site ─────────────────────────────

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SocialPost {
    pub id: String,
    pub platform: &'static str,
    pub author: String,
    pub handle: String,
    pub avatar: Option<String>,
    pub text: String,
    pub url: String,
    pub image: Option<String>,
    pub published_at: String,
    pub likes: Option<u64>,
    pub reposts: Option<u64>,
    pub starship: bool,
}

pub fn feed_url(handle: &str) -> String {
    format!(
        "https://public.api.bsky.app/xrpc/app.bsky.feed.getAuthorFeed?actor={handle}&limit=30&filter=posts_no_replies"
    )
}

/// `at://did:plc:xxx/app.bsky.feed.post/<rkey>` → `https://bsky.app/profile/<handle>/post/<rkey>`
pub fn web_url(handle: &str, uri: &str) -> Option<String> {
    let rkey = uri.strip_prefix("at://")?.rsplit('/').next().filter(|k| !k.is_empty())?;
    Some(format!("https://bsky.app/profile/{handle}/post/{rkey}"))
}

/// Équivalent de /starship|super heavy|starbase|booster \d|ship \d/i sans dépendance regex.
pub fn mentions_starship(text: &str) -> bool {
    let t = text.to_lowercase();
    if ["starship", "super heavy", "starbase"].iter().any(|k| t.contains(k)) {
        return true;
    }
    ["booster ", "ship "].iter().any(|k| {
        t.match_indices(k)
            .any(|(i, _)| t[i + k.len()..].starts_with(|c: char| c.is_ascii_digit()))
    })
}

fn to_posts(feed: Feed) -> Vec<SocialPost> {
    feed.feed
        .into_iter()
        .filter(|item| item.reason.is_none())
        .filter_map(|FeedItem { post: p, .. }| {
            let text = p.record.text.trim().to_string();
            let image = p.embed.as_ref().and_then(Embed::image);
            if text.is_empty() && image.is_none() {
                return None;
            }
            // indexedAt est fixé par le serveur : plus fiable que createdAt (déclaré par le client).
            let published_at = if p.indexed_at.is_empty() {
                p.record.created_at.clone()?
            } else {
                p.indexed_at.clone()
            };
            Some(SocialPost {
                url: web_url(&p.author.handle, &p.uri)?,
                id: p.uri,
                platform: "bluesky",
                author: p
                    .author
                    .display_name
                    .filter(|n| !n.trim().is_empty())
                    .unwrap_or_else(|| p.author.handle.clone()),
                handle: p.author.handle,
                avatar: p.author.avatar,
                starship: mentions_starship(&text),
                text,
                image,
                published_at,
                likes: p.like_count,
                reposts: p.repost_count,
            })
        })
        .collect()
}

/// Dédoublonne, filtre par âge, limite par auteur, trie du plus récent au plus ancien.
pub fn select(mut posts: Vec<SocialPost>, cutoff_iso: &str) -> Vec<SocialPost> {
    posts.retain(|p| p.published_at.as_str() >= cutoff_iso);
    posts.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    let mut seen = HashSet::new();
    let mut per_author: std::collections::HashMap<String, usize> = Default::default();
    posts
        .into_iter()
        .filter(|p| seen.insert(p.id.clone()))
        .filter(|p| {
            let n = per_author.entry(p.handle.clone()).or_default();
            *n += 1;
            *n <= MAX_PER_AUTHOR
        })
        .take(MAX_POSTS)
        .collect()
}

pub fn update(client: &io::Client, out: &Path) -> Result<()> {
    let mut posts = Vec::new();
    let mut ok = 0;
    for handle in BLUESKY_HANDLES {
        match io::get::<Feed>(client, &feed_url(handle)) {
            Ok(feed) => {
                ok += 1;
                posts.extend(to_posts(feed));
            }
            Err(e) => eprintln!("   ↳ Bluesky {handle} ignoré : {e:#}"),
        }
    }
    anyhow::ensure!(ok > 0, "aucun compte Bluesky joignable, on garde les anciennes données");

    let cutoff = io::iso_utc(io::now_unix() - MAX_AGE_DAYS * 86_400);
    let posts = select(posts, &cutoff);
    io::write_json(&out.join("data/social.json"), &posts)?;
    println!(
        "🦋 {} posts Bluesky ({} Starship) depuis {ok}/{} comptes",
        posts.len(),
        posts.iter().filter(|p| p.starship).count(),
        BLUESKY_HANDLES.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"feed":[
      {"post":{"uri":"at://did:plc:abc/app.bsky.feed.post/3n22lgkwlbs2o","cid":"x",
        "author":{"did":"did:plc:abc","handle":"nasaspaceflight.com","displayName":"NASASpaceflight.com","avatar":"https://cdn.bsky.app/a.jpg"},
        "record":{"$type":"app.bsky.feed.post","createdAt":"2026-11-11T18:27:30.504Z","text":"Starship Flight 15 static fire complete!"},
        "embed":{"$type":"app.bsky.embed.images#view","images":[{"thumb":"https://t.jpg","fullsize":"https://f.jpg","alt":""}]},
        "replyCount":1,"repostCount":3,"likeCount":42,"indexedAt":"2026-11-11T18:27:35.770Z"}},
      {"post":{"uri":"at://did:plc:zzz/app.bsky.feed.post/aaa","cid":"y",
        "author":{"did":"did:plc:zzz","handle":"other.bsky.social"},
        "record":{"text":"repost"},"indexedAt":"2026-11-10T00:00:00Z"},
       "reason":{"$type":"app.bsky.feed.defs#reasonRepost"}},
      {"post":{"uri":"at://did:plc:abc/app.bsky.feed.post/bbb","cid":"z",
        "author":{"did":"did:plc:abc","handle":"esa.int","displayName":""},
        "record":{"text":"Ariane 6 ready"},
        "embed":{"$type":"app.bsky.embed.external#view","external":{"uri":"https://esa.int","thumb":"https://ext.jpg"}},
        "indexedAt":"2026-11-01T10:00:00.000Z"}}
    ]}"#;

    #[test]
    fn parse_feed_bluesky() {
        let feed: Feed = serde_json::from_str(SAMPLE).unwrap();
        let posts = to_posts(feed);
        assert_eq!(posts.len(), 2, "le repost doit être ignoré");
        let p = &posts[0];
        assert_eq!(p.url, "https://bsky.app/profile/nasaspaceflight.com/post/3n22lgkwlbs2o");
        assert_eq!(p.image.as_deref(), Some("https://f.jpg"));
        assert_eq!(p.likes, Some(42));
        assert!(p.starship);
        assert_eq!(posts[1].author, "esa.int", "displayName vide → handle");
        assert_eq!(posts[1].image.as_deref(), Some("https://ext.jpg"));
        assert!(!posts[1].starship);
    }

    #[test]
    fn urls() {
        assert_eq!(
            web_url("a.b", "at://did:plc:x/app.bsky.feed.post/k1").as_deref(),
            Some("https://bsky.app/profile/a.b/post/k1")
        );
        assert_eq!(web_url("a.b", "https://pas-un-uri"), None);
        assert!(feed_url("esa.int").contains("actor=esa.int&"));
    }

    #[test]
    fn filtre_starship() {
        assert!(mentions_starship("STARSHIP is go"));
        assert!(mentions_starship("Super Heavy Booster 18 rolls out"));
        assert!(mentions_starship("Ship 39 static fire"));
        assert!(mentions_starship("Back at Starbase"));
        assert!(!mentions_starship("The ship is ready"));
        assert!(!mentions_starship("Falcon 9 booster lands"));
    }

    #[test]
    fn selection_age_tri_doublons() {
        let feed: Feed = serde_json::from_str(SAMPLE).unwrap();
        let mut posts = to_posts(feed);
        posts.push(posts[0].clone());
        let sel = select(posts, "2026-11-05T00:00:00Z");
        assert_eq!(sel.len(), 1);
        assert_eq!(sel[0].handle, "nasaspaceflight.com");
    }
}
