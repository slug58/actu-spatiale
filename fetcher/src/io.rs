//! Petits utilitaires HTTP / fichiers / dates partagés par toutes les sources.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, time::SystemTime};

pub type Client = reqwest::blocking::Client;

pub fn get<T: for<'de> Deserialize<'de>>(client: &Client, url: &str) -> Result<T> {
    let resp = client.get(url).send().with_context(|| format!("requête {url}"))?;
    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!("{url} → HTTP {status}");
    }
    resp.json().with_context(|| format!("JSON invalide depuis {url}"))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut s = serde_json::to_string_pretty(value)?;
    s.push('\n');
    fs::write(path, s).with_context(|| format!("écriture de {}", path.display()))
}

/// Date UTC au format ISO 8601 (`YYYY-MM-DDTHH:MM:SSZ`) pour un timestamp Unix.
/// Algorithme « civil_from_days » de Howard Hinnant : évite une dépendance à chrono.
pub fn iso_utc(unix_secs: i64) -> String {
    let days = unix_secs.div_euclid(86_400);
    let secs = unix_secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

pub fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_iso() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(iso_utc(1_794_418_055), "2026-11-11T17:27:35Z");
    }
}
