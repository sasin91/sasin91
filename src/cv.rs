//! The CV domain model: `content/cv.toml` deserialized into `Cv` and the
//! types it's built from, plus the date validation and formatting Askama
//! templates rely on. `main.rs` owns loading the file and calling
//! `Cv::validate`; everything else here is the shape of a person's CV.

use anyhow::{Context, Result, anyhow};
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Cv {
    pub site: Profile,
    pub contact: Contact,
    pub intro: Vec<String>,
    /// Prose for /about only. `intro` is the CV's own wording and belongs on
    /// /cv; this is written for a reader arriving at the site cold.
    pub about: Vec<String>,
    pub roles: Vec<Role>,
    pub skills: Vec<Skill>,
    pub education: Vec<Education>,
    /// Descriptor for the education section as a whole (e.g. "Two
    /// short-cycle higher educations."), not tied to any single entry.
    #[serde(default)]
    pub education_note: Option<String>,
}

impl Cv {
    /// Every `start`/`end` in `roles` and `education` must parse as
    /// `YYYY-MM`. This is a person's employment history used in job
    /// applications, so a malformed date must stop the build — with the
    /// offending value and field named — rather than let `month()` fall back
    /// to rendering the raw TOML string on the live `/cv` page and emitting
    /// an invalid `<time datetime>`. Called once, right after parsing
    /// `content/cv.toml`, so every template-facing `*_label()` method below
    /// can assume its date is already valid and stay infallible.
    pub fn validate(&self) -> Result<()> {
        for role in &self.roles {
            month(&role.start).with_context(|| {
                format!("role {:?} at {:?}: start date", role.title, role.company)
            })?;
            if let Some(end) = &role.end {
                month(end).with_context(|| {
                    format!("role {:?} at {:?}: end date", role.title, role.company)
                })?;
            }
        }
        for edu in &self.education {
            month(&edu.start).with_context(|| {
                format!("education {:?} at {:?}: start date", edu.title, edu.school)
            })?;
            if let Some(end) = &edu.end {
                month(end).with_context(|| {
                    format!("education {:?} at {:?}: end date", edu.title, edu.school)
                })?;
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Profile {
    pub name: String,
    pub title: String,
    pub stack: String,
    pub available: bool,
    pub available_note: String,
    pub links: Links,
}

#[derive(Deserialize)]
pub struct Links {
    pub github: String,
    pub linkedin: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct Contact {
    pub town: String,
    pub postcode: String,
    pub phone: String,
    pub email: String,
}

impl Contact {
    /// `phone` with its spaces stripped, for a `tel:` href — RFC 3966 has no
    /// concept of the visual grouping spaces `+45 50106917` has for a
    /// reader, and a raw space in the URI is invalid there. `phone` itself
    /// is left untouched for display, where the spacing helps.
    pub fn phone_href(&self) -> String {
        self.phone.replace(' ', "")
    }
}

#[derive(Deserialize)]
pub struct Role {
    pub start: String,
    pub end: Option<String>,
    pub title: String,
    pub company: String,
    pub location: String,
    pub summary: String,
    #[serde(default)]
    pub achievements: Vec<String>,
}

/// "2024-09" -> "September 2024". Errors name the bad value so a caller can
/// report which field it came from; see `Cv::validate`, which is the only
/// place this can fail once the build is past parsing `content/cv.toml` —
/// every `*_label()` method below relies on that and unwraps.
fn month(m: &str) -> Result<String> {
    NaiveDate::parse_from_str(&format!("{m}-01"), "%Y-%m-%d")
        .map(|d| d.format("%B %Y").to_string())
        .map_err(|e| anyhow!("invalid date {m:?} (want YYYY-MM): {e}"))
}

impl Role {
    /// "September 2024", from `start`.
    pub fn start_label(&self) -> String {
        month(&self.start).expect("date already validated by Cv::validate")
    }

    /// "February 2026", from `end` — `None` while the role is still open, so
    /// a template can tell "ended" from "ongoing" rather than guessing from
    /// an empty string.
    pub fn end_label(&self) -> Option<String> {
        self.end
            .as_deref()
            .map(|e| month(e).expect("date already validated by Cv::validate"))
    }
}

#[derive(Deserialize)]
pub struct Skill {
    pub name: String,
}

#[derive(Deserialize)]
pub struct Education {
    pub start: String,
    pub end: Option<String>,
    pub title: String,
    pub school: String,
    pub location: String,
    #[serde(default)]
    pub note: Option<String>,
}

impl Education {
    /// "February 2014", from `start`.
    pub fn start_label(&self) -> String {
        month(&self.start).expect("date already validated by Cv::validate")
    }

    /// "August 2015", from `end` — `None` while ongoing.
    pub fn end_label(&self) -> Option<String> {
        self.end
            .as_deref()
            .map(|e| month(e).expect("date already validated by Cv::validate"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal but complete `Cv` TOML: every field with no `#[serde(default)]`
    /// is present, so this only fails to parse if the date under test is bad.
    fn cv_with_dates(role_start: &str, role_end: &str, edu_start: &str, edu_end: &str) -> Cv {
        let src = format!(
            r#"
intro = []
about = []

[site]
name = "x"
title = "x"
stack = "x"
available = true
available_note = "x"

[site.links]
github = "x"
linkedin = "x"
email = "x"

[contact]
town = "x"
postcode = "x"
phone = "x"
email = "x"

[[roles]]
start = "{role_start}"
end = "{role_end}"
title = "Web developer"
company = "Acme"
location = "x"
summary = "x"

[[skills]]
name = "x"

[[education]]
start = "{edu_start}"
end = "{edu_end}"
title = "Diploma"
school = "Acme Tech"
location = "x"
"#
        );
        toml::from_str(&src).expect("fixture TOML must itself be well-formed")
    }

    #[test]
    fn month_formats_a_valid_year_month() {
        assert_eq!(month("2024-09").unwrap(), "September 2024");
    }

    #[test]
    fn month_rejects_an_out_of_range_month() {
        let err = month("2024-13").unwrap_err();
        assert!(
            err.to_string().contains("2024-13"),
            "error should name the bad value: {err}"
        );
    }

    #[test]
    fn validate_accepts_well_formed_dates() {
        let cv = cv_with_dates("2024-09", "2026-02", "2014-02", "2015-08");
        assert!(cv.validate().is_ok());
    }

    #[test]
    fn validate_rejects_a_malformed_role_start_date() {
        let cv = cv_with_dates("2024-13", "2026-02", "2014-02", "2015-08");
        let err = cv.validate().unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("2024-13"), "should name the bad value: {msg}");
        assert!(msg.contains("start date"), "should name the field: {msg}");
        assert!(msg.contains("Acme"), "should name which role: {msg}");
    }

    #[test]
    fn validate_rejects_a_malformed_education_end_date() {
        let cv = cv_with_dates("2024-09", "2026-02", "2014-02", "2015-99");
        let err = cv.validate().unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("2015-99"), "should name the bad value: {msg}");
        assert!(msg.contains("end date"), "should name the field: {msg}");
        assert!(
            msg.contains("Acme Tech"),
            "should name which education entry: {msg}"
        );
    }
}
