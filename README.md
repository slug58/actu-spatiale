# 🚀 ActuSpatiale

Site d'actualité spatiale centré sur le **Starship de SpaceX** : prochain lancement avec
compte à rebours, historique de tous les vols, statistiques, fil d'actu, posts des
journalistes spatiaux et blog.

Pas de serveur ni de base de données : un **robot en Rust** récupère les données toutes les
3 heures, puis **Astro** génère un site statique publié sur **GitHub Pages**.

## Fonctionnement

```
 ┌──────────────────────┐
 │ GitHub Actions (cron │  toutes les 3 h, à chaque push sur main, ou à la main
 │ 17 */3 * * *)        │
 └──────────┬───────────┘
            ▼
 ┌──────────────────────┐   Launch Library 2 ─┐
 │ Robot Rust           │◄─ Spaceflight News ─┤ API publiques gratuites
 │ fetcher/             │   Bluesky ──────────┘
 └──────────┬───────────┘
            ▼  écrit
 ┌──────────────────────────────────────────────────┐
 │ site/src/data/*.json        (lancements, stats,  │
 │                              actus, réseaux)     │
 │ site/src/content/blog/*.md  (récap auto par vol) │
 └──────────┬───────────────────────────────────────┘
            ▼  commit auto « data: mise à jour automatique »
 ┌──────────────────────┐
 │ Astro (site/)        │  npm run build → site/dist
 └──────────┬───────────┘
            ▼
 ┌──────────────────────┐
 │ GitHub Pages         │  https://<utilisateur>.github.io/actu-spatiale/
 └──────────────────────┘
```

Toute la chaîne tient dans un seul workflow, `.github/workflows/deploy.yml`. C'est voulu :
un push fait avec `GITHUB_TOKEN` ne déclenche pas d'autre workflow. Si une API est en
panne, l'étape de collecte échoue sans bloquer le reste, et le site est publié avec les
données précédentes.

## Sources de données

| Source | Usage | Remarques |
|---|---|---|
| [Launch Library 2](https://thespacedevs.com/llapi) (The Space Devs) | lancements, statuts, pas de tir, images | sans clé, **limite de 15 requêtes/heure** ; le robot n'en fait qu'une par passage |
| [Spaceflight News API](https://www.spaceflightnewsapi.net/) | articles de presse spatiale | gratuite, sans clé |
| [Bluesky](https://docs.bsky.app/) | posts de comptes spatiaux | API publique, sans clé |

> **Pourquoi pas X.com ?** Il faut payer pour lire via l'API de X, et récupérer les pages
> automatiquement (scraping) va contre leurs conditions d'utilisation. Bluesky prend le
> relais : son API est ouverte et beaucoup de journalistes spatiaux y publient.

## Structure du dépôt

```
actu-spatiale/
├── .github/workflows/deploy.yml   # cron + collecte + build + déploiement
├── fetcher/                       # robot de collecte (Rust)
│   ├── Cargo.toml
│   └── src/main.rs
└── site/                          # site statique (Astro)
    ├── astro.config.mjs           # lit SITE_URL / BASE_PATH (fournis par la CI)
    ├── public/                    # fichiers statiques (favicon…)
    └── src/
        ├── data/                  # JSON générés par le robot (versionnés)
        ├── content/blog/          # articles Markdown (manuels + auto)
        ├── components/  layouts/  pages/  styles/  lib/
        └── content.config.ts      # schéma des articles
```

## Lancer en local

Prérequis : [Rust](https://rustup.rs/) et [Node.js](https://nodejs.org/) 22+
(sur Arch : `sudo pacman -S rust nodejs npm`).

```bash
# 1. Récupérer les données fraîches
cargo run --release --manifest-path fetcher/Cargo.toml -- --out site/src

# 2. Lancer le site
cd site
npm install        # crée site/package-lock.json : à committer (la CI fait `npm ci`)
npm run dev        # → http://localhost:4321
```

Tests du robot : `cargo test --manifest-path fetcher/Cargo.toml`.

## Écrire un article

Crée un fichier `site/src/content/blog/mon-article.md` :

```markdown
---
title: "Flight 15 : mon analyse"
date: 2026-11-14T10:00:00Z
description: "Ce que ce vol change pour la suite du programme."
tags: ["analyse", "Starship V3"]
image: "https://exemple.org/photo.jpg"   # optionnel
---

Ton texte en **Markdown**…
```

Le robot écrit un récap automatique (`auto: true`) pour chaque vol intégré terminé.
**Il n'écrase jamais un fichier qui existe déjà** : tu peux réécrire ces récaps à la main
sans risque.

## Mise en ligne, pas à pas

1. Sur GitHub, crée un dépôt **public** vide nommé `actu-spatiale`. Ne coche aucune
   option (pas de README, pas de .gitignore).
2. Dans ce dossier :
   ```bash
   git add .
   git commit -m "Premier commit ActuSpatiale"
   git remote add origin git@github.com:<utilisateur>/actu-spatiale.git
   git push -u origin main
   ```
3. Sur GitHub : **Settings → Pages → Build and deployment → Source : « GitHub Actions »**.
4. La première fois, lance le workflow à la main : **Actions → « Mise à jour &
   déploiement » → Run workflow**.
5. Le site est en ligne sur `https://<utilisateur>.github.io/actu-spatiale/`.

> ⚠️ Dans un dépôt public, GitHub **désactive les workflows planifiés après 60 jours sans
> activité**. Les commits automatiques de données suffisent normalement à l'éviter. Si le
> cron s'arrête quand même, réactive-le dans l'onglet Actions.

## Licence et crédits

Données © The Space Devs (Launch Library 2, Spaceflight News API) et leurs sources. Les
images appartiennent à leurs auteurs, crédités sur le site.
