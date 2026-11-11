---
title: "Bienvenue sur ActuSpatiale"
date: 2026-11-11T12:00:00Z
description: "Pourquoi ce site, comment il se met à jour tout seul, et ce qui arrive ensuite."
tags: ["meta"]
---

Bienvenue ! **ActuSpatiale** suit l'actualité des lancements spatiaux, avec un focus
particulier sur le **Starship** de SpaceX, le plus grand lanceur jamais construit.

## Comment ça marche ?

Le site n'a pas de serveur. Toutes les 3 heures, un robot écrit en **Rust** tourne dans
GitHub Actions :

1. il interroge [Launch Library 2](https://thespacedevs.com/llapi) (lancements),
   [Spaceflight News API](https://www.spaceflightnewsapi.net/) (articles) et les réseaux sociaux ouverts ;
2. il écrit les données dans le dépôt et rédige un brouillon d'article pour chaque nouveau vol ;
3. le site est reconstruit avec **Astro** puis publié sur GitHub Pages.

## Et ensuite ?

Des analyses de vol écrites à la main, des graphes, et peut-être d'autres fusées
(New Glenn, Ariane 6, Neutron…).
