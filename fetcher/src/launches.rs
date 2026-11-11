//! Lancements Starship depuis Launch Library 2 (+ statistiques).

use crate::{blog, io};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

const LAUNCHES_URL: &str =
    "https://ll.thespacedevs.com/2.3.0/launches/?search=starship&limit=100&mode=normal&ordering=net";

#[derive(Deserialize)]
struct Page<T> {
    results: Vec<T>,
}

#[derive(Deserialize)]
struct Named {
    name: String,
    abbrev: Option<String>,
}

#[derive(Deserialize)]
struct RawStatus {
    name: String,
    abbrev: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct RawImage {
    image_url: Option<String>,
    thumbnail_url: Option<String>,
    credit: Option<String>,
}

#[derive(Deserialize)]
struct RawRocket {
    configuration: Named,
}

#[derive(Deserialize)]
struct RawMission {
    description: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    orbit: Option<Named>,
}

#[derive(Deserialize)]
struct RawPad {
    name: String,
    location: Named,
}

#[derive(Deserialize)]
struct RawLaunch {
    id: String,
    slug: String,
    name: String,
    net: String,
    net_precision: Option<Named>,
    status: RawStatus,
    image: Option<RawImage>,
    failreason: Option<String>,
    rocket: RawRocket,
    mission: Option<RawMission>,
    pad: RawPad,
    webcast_live: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Launch {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub mission: String,
    pub flight_number: Option<u32>,
    pub net: String,
    pub net_precision: Option<String>,
    pub status: String,
    pub status_name: String,
    pub status_description: Option<String>,
    pub vehicle: String,
    pub prototype: bool,
    pub pad: String,
    pub location: String,
    pub image: Option<String>,
    pub thumbnail: Option<String>,
    pub image_credit: Option<String>,
    pub description: Option<String>,
    pub mission_type: Option<String>,
    pub orbit: Option<String>,
    pub fail_reason: Option<String>,
    pub webcast_live: bool,
}

impl Launch {
    pub fn is_upcoming(&self) -> bool {
        matches!(self.status.as_str(), "Go" | "TBD" | "TBC" | "Hold" | "In Flight")
    }
    pub fn is_done(&self) -> bool {
        matches!(self.status.as_str(), "Success" | "Failure" | "Partial Failure")
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleStats {
    pub vehicle: String,
    pub flights: u32,
    pub success: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub total: u32,
    pub completed: u32,
    pub success: u32,
    pub partial: u32,
    pub failure: u32,
    pub upcoming: u32,
    /// Taux de succès des vols intégrés (Super Heavy + Ship), en %.
    pub integrated_success_rate: f64,
    pub integrated_flights: u32,
    pub by_vehicle: Vec<VehicleStats>,
    pub next_launch_id: Option<String>,
    pub last_launch_id: Option<String>,
}

/// "Flight 10" → 10, "Integrated Flight Test 3" → 3, "Integrated Flight Test" → 1.
pub fn flight_number(mission: &str) -> Option<u32> {
    if let Some(rest) = mission.strip_prefix("Integrated Flight Test") {
        return Some(rest.trim().parse().unwrap_or(1));
    }
    let idx = mission.rfind("Flight ")?;
    let digits: String = mission[idx + 7..].chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn clean(s: Option<String>) -> Option<String> {
    s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn to_launch(r: RawLaunch) -> Launch {
    let mission = r
        .name
        .split_once(" | ")
        .map(|(_, m)| m.to_string())
        .unwrap_or_else(|| r.name.clone());
    let (image, thumbnail, image_credit) = match r.image {
        Some(i) => (i.image_url, i.thumbnail_url, i.credit),
        None => (None, None, None),
    };
    let (description, mission_type, orbit) = match r.mission {
        Some(m) => (clean(m.description), m.kind, m.orbit.map(|o| o.name)),
        None => (None, None, None),
    };
    let vehicle = r.rocket.configuration.name;
    Launch {
        flight_number: flight_number(&mission),
        prototype: vehicle.contains("Prototype"),
        id: r.id,
        slug: r.slug,
        name: r.name,
        mission,
        net: r.net,
        net_precision: r.net_precision.and_then(|p| p.abbrev.or(Some(p.name))),
        status: r.status.abbrev,
        status_name: r.status.name,
        status_description: r.status.description,
        vehicle,
        pad: r.pad.name,
        location: r.pad.location.name,
        image,
        thumbnail,
        image_credit,
        description,
        mission_type,
        orbit,
        fail_reason: clean(r.failreason),
        webcast_live: r.webcast_live,
    }
}

fn compute_stats(launches: &[Launch]) -> Stats {
    let count = |f: &dyn Fn(&Launch) -> bool| launches.iter().filter(|l| f(l)).count() as u32;
    let integrated: Vec<&Launch> = launches.iter().filter(|l| !l.prototype && l.is_done()).collect();
    let integrated_ok = integrated.iter().filter(|l| l.status == "Success").count();

    let mut by_vehicle: Vec<VehicleStats> = Vec::new();
    for l in launches.iter().filter(|l| l.is_done()) {
        let entry = match by_vehicle.iter_mut().find(|v| v.vehicle == l.vehicle) {
            Some(e) => e,
            None => {
                by_vehicle.push(VehicleStats { vehicle: l.vehicle.clone(), flights: 0, success: 0 });
                by_vehicle.last_mut().unwrap()
            }
        };
        entry.flights += 1;
        entry.success += (l.status == "Success") as u32;
    }

    Stats {
        total: launches.len() as u32,
        completed: count(&|l| l.is_done()),
        success: count(&|l| l.status == "Success"),
        partial: count(&|l| l.status == "Partial Failure"),
        failure: count(&|l| l.status == "Failure"),
        upcoming: count(&|l| l.is_upcoming()),
        integrated_flights: integrated.len() as u32,
        integrated_success_rate: if integrated.is_empty() {
            0.0
        } else {
            (integrated_ok as f64 * 1000.0 / integrated.len() as f64).round() / 10.0
        },
        by_vehicle,
        next_launch_id: launches.iter().find(|l| l.is_upcoming()).map(|l| l.id.clone()),
        last_launch_id: launches.iter().rev().find(|l| l.is_done()).map(|l| l.id.clone()),
    }
}

pub fn update(client: &io::Client, out: &Path) -> Result<()> {
    let page: Page<RawLaunch> = io::get(client, LAUNCHES_URL)?;
    let launches: Vec<Launch> = page.results.into_iter().map(to_launch).collect();
    anyhow::ensure!(!launches.is_empty(), "aucun lancement reçu, on garde les anciennes données");

    let stats = compute_stats(&launches);
    io::write_json(&out.join("data/launches.json"), &launches)?;
    io::write_json(&out.join("data/stats.json"), &stats)?;
    let posts = blog::write_flight_posts(&launches, &out.join("content/blog"))?;

    println!(
        "🚀 {} lancements ({} réussis, {} à venir), {} nouvel(s) article(s)",
        stats.total, stats.success, stats.upcoming, posts
    );
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeros_de_vol() {
        assert_eq!(flight_number("Flight 10"), Some(10));
        assert_eq!(flight_number("Integrated Flight Test"), Some(1));
        assert_eq!(flight_number("Integrated Flight Test 3"), Some(3));
        assert_eq!(flight_number("Starlink Group 31-2 (Starship Flight 15)"), Some(15));
        assert_eq!(flight_number("12.5 km Flight"), None);
        assert_eq!(flight_number("Superbird-9"), None);
    }
}
