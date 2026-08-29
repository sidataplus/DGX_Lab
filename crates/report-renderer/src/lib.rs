#![forbid(unsafe_code)]

//! Deterministic Markdown and standalone HTML certification reports.

use assessment::CertificationResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CertificateData {
    pub learner_display_name: String,
    pub course_title: String,
    pub course_revision: String,
    pub app_version: String,
    pub completed_at_display: String,
    pub result: CertificationResult,
    pub evidence_digest: String,
    pub verification_level: VerificationLevel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationLevel {
    Standalone,
    InstructorVerified,
    InstitutionallyVerified,
}

pub fn render_markdown(data: &CertificateData) -> String {
    format!(
        "# DGX Lab Certificate of Competency\n\n\
         **Learner:** {}  \n\
         **Course:** {}  \n\
         **Course revision:** {}  \n\
         **Completed:** {}  \n\
         **Result:** {}  \n\
         **Overall score:** {}%  \n\
         **Knowledge score:** {}%  \n\
         **Practical score:** {}%  \n\
         **Verification level:** {:?}  \n\
         **Evidence digest:** `{}`\n\n\
         This certificate was generated locally by DGX Lab. Standalone certificates do not independently verify learner identity.\n",
        data.learner_display_name,
        data.course_title,
        data.course_revision,
        data.completed_at_display,
        if data.result.passed { "PASS" } else { "NOT PASSED" },
        data.result.overall_percent,
        data.result.knowledge_percent,
        data.result.practical_percent,
        data.verification_level,
        data.evidence_digest
    )
}

pub fn render_html(data: &CertificateData) -> String {
    let status = if data.result.passed { "PASS" } else { "NOT PASSED" };
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>DGX Lab Certificate</title>
<style>
body {{ font-family: system-ui, sans-serif; color: #142033; margin: 0; background: #eef2f7; }}
.certificate {{ width: min(900px, calc(100% - 48px)); margin: 48px auto; background: white; border: 2px solid #234a86; padding: 64px; box-sizing: border-box; }}
h1 {{ color: #123f7d; margin-bottom: 0; }}
.score {{ font-size: 56px; font-weight: 700; color: #176b43; }}
.grid {{ display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }}
.small {{ color: #526070; font-size: 14px; }}
@media print {{ body {{ background: white; }} .certificate {{ margin: 0; width: 100%; min-height: 100vh; }} }}
</style>
</head>
<body>
<main class="certificate">
<p>DGX Lab · Interactive SLURM Training Simulator</p>
<h1>Certificate of Competency</h1>
<p>This certifies that</p>
<h2>{learner}</h2>
<p>completed <strong>{course}</strong> revision {revision}.</p>
<div class="score">{status}</div>
<div class="grid">
<p><strong>Overall</strong><br>{overall}%</p>
<p><strong>Knowledge</strong><br>{knowledge}%</p>
<p><strong>Practical</strong><br>{practical}%</p>
<p><strong>Completed</strong><br>{completed}</p>
</div>
<hr>
<p class="small">Verification level: {verification:?}<br>
Evidence digest: <code>{digest}</code><br>
Application version: {app}</p>
<p class="small">Generated locally. Standalone evidence does not independently verify learner identity.</p>
</main>
</body>
</html>"#,
        learner = escape_html(&data.learner_display_name),
        course = escape_html(&data.course_title),
        revision = escape_html(&data.course_revision),
        status = status,
        overall = data.result.overall_percent,
        knowledge = data.result.knowledge_percent,
        practical = data.result.practical_percent,
        completed = escape_html(&data.completed_at_display),
        verification = data.verification_level,
        digest = escape_html(&data.evidence_digest),
        app = escape_html(&data.app_version),
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learner_text_is_escaped() {
        let data = CertificateData {
            learner_display_name: "<Max>".into(),
            course_title: "SLURM".into(),
            course_revision: "1".into(),
            app_version: "0.1".into(),
            completed_at_display: "today".into(),
            result: CertificationResult {
                practical_percent: 100,
                multiple_choice_percent: 90,
                fill_blank_percent: 90,
                knowledge_percent: 90,
                overall_percent: 96,
                critical_practical_passed: true,
                passed: true,
            },
            evidence_digest: "abc".into(),
            verification_level: VerificationLevel::Standalone,
        };
        let html = render_html(&data);
        assert!(html.contains("&lt;Max&gt;"));
        assert!(!html.contains("<Max>"));
    }
}
