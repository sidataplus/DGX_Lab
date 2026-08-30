#![forbid(unsafe_code)]

//! Deterministic knowledge assessment and certification scoring.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Question {
    SingleChoice {
        id: String,
        competency: String,
        prompt: String,
        options: Vec<OptionItem>,
        correct: String,
        points: u32,
        explanation: String,
    },
    MultiSelect {
        id: String,
        competency: String,
        prompt: String,
        options: Vec<OptionItem>,
        correct: BTreeSet<String>,
        points: u32,
        incorrect_penalty_basis_points: u16,
        explanation: String,
    },
    FillBlank {
        id: String,
        competency: String,
        prompt: String,
        blanks: Vec<BlankDefinition>,
        points: u32,
        explanation: String,
    },
}

impl Question {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::SingleChoice { id, .. }
            | Self::MultiSelect { id, .. }
            | Self::FillBlank { id, .. } => id,
        }
    }

    #[must_use]
    pub fn points(&self) -> u32 {
        match self {
            Self::SingleChoice { points, .. }
            | Self::MultiSelect { points, .. }
            | Self::FillBlank { points, .. } => *points,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionItem {
    pub id: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlankDefinition {
    pub id: String,
    pub accepted: Vec<AcceptedAnswer>,
    #[serde(default)]
    pub case_insensitive: bool,
    #[serde(default = "default_true")]
    pub trim: bool,
    #[serde(default = "default_true")]
    pub normalize_whitespace: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AcceptedAnswer {
    Literal { value: String },
    Numeric { value: f64, tolerance: f64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Answer {
    SingleChoice { option_id: String },
    MultiSelect { option_ids: BTreeSet<String> },
    FillBlank { values: BTreeMap<String, String> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuestionScore {
    pub question_id: String,
    pub earned_milli_points: u32,
    pub possible_milli_points: u32,
    pub correct: bool,
    pub feedback: String,
}

pub fn score_question(
    question: &Question,
    answer: &Answer,
) -> Result<QuestionScore, AssessmentError> {
    let possible = question.points().saturating_mul(1_000);
    match (question, answer) {
        (Question::SingleChoice { id, correct, .. }, Answer::SingleChoice { option_id }) => {
            let is_correct = option_id == correct;
            Ok(QuestionScore {
                question_id: id.clone(),
                earned_milli_points: if is_correct { possible } else { 0 },
                possible_milli_points: possible,
                correct: is_correct,
                feedback: if is_correct { "Correct".into() } else { "Incorrect selection".into() },
            })
        }
        (
            Question::MultiSelect { id, correct, incorrect_penalty_basis_points, .. },
            Answer::MultiSelect { option_ids },
        ) => {
            let correct_selected = option_ids.intersection(correct).count() as u32;
            let incorrect_selected = option_ids.difference(correct).count() as u32;
            let denominator = correct.len().max(1) as u32;
            let positive = possible.saturating_mul(correct_selected) / denominator;
            let penalty = possible
                .saturating_mul(incorrect_selected)
                .saturating_mul(u32::from(*incorrect_penalty_basis_points))
                / 10_000;
            let earned = positive.saturating_sub(penalty).min(possible);
            Ok(QuestionScore {
                question_id: id.clone(),
                earned_milli_points: earned,
                possible_milli_points: possible,
                correct: option_ids == correct,
                feedback: format!(
                    "selected {correct_selected} correct and {incorrect_selected} incorrect option(s)"
                ),
            })
        }
        (Question::FillBlank { id, blanks, .. }, Answer::FillBlank { values }) => {
            let correct_count = blanks
                .iter()
                .filter(|blank| {
                    values.get(&blank.id).is_some_and(|value| blank_matches(blank, value))
                })
                .count() as u32;
            let denominator = blanks.len().max(1) as u32;
            let earned = possible.saturating_mul(correct_count) / denominator;
            Ok(QuestionScore {
                question_id: id.clone(),
                earned_milli_points: earned,
                possible_milli_points: possible,
                correct: correct_count == blanks.len() as u32,
                feedback: format!("{correct_count}/{} blank(s) correct", blanks.len()),
            })
        }
        _ => Err(AssessmentError::AnswerTypeMismatch { question_id: question.id().into() }),
    }
}

fn blank_matches(blank: &BlankDefinition, submitted: &str) -> bool {
    let normalized =
        normalize_text(submitted, blank.case_insensitive, blank.trim, blank.normalize_whitespace);
    blank.accepted.iter().any(|accepted| match accepted {
        AcceptedAnswer::Literal { value } => {
            normalized
                == normalize_text(
                    value,
                    blank.case_insensitive,
                    blank.trim,
                    blank.normalize_whitespace,
                )
        }
        AcceptedAnswer::Numeric { value, tolerance } => normalized
            .parse::<f64>()
            .ok()
            .is_some_and(|candidate| (candidate - value).abs() <= *tolerance),
    })
}

fn normalize_text(value: &str, case_insensitive: bool, trim: bool, whitespace: bool) -> String {
    let mut output = if trim { value.trim().to_string() } else { value.to_string() };
    if whitespace {
        output = output.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    if case_insensitive {
        output = output.to_lowercase();
    }
    output
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassPolicy {
    pub overall_percent: u8,
    pub knowledge_percent: u8,
    pub require_all_critical_practical: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificationWeights {
    pub practical: u8,
    pub multiple_choice: u8,
    pub fill_blank: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CertificationResult {
    pub practical_percent: u8,
    pub multiple_choice_percent: u8,
    pub fill_blank_percent: u8,
    pub knowledge_percent: u8,
    pub overall_percent: u8,
    pub critical_practical_passed: bool,
    pub passed: bool,
}

pub fn finalize_certification(
    policy: &PassPolicy,
    weights: &CertificationWeights,
    practical_percent: u8,
    multiple_choice_percent: u8,
    fill_blank_percent: u8,
    critical_practical_passed: bool,
) -> Result<CertificationResult, AssessmentError> {
    if u16::from(weights.practical)
        + u16::from(weights.multiple_choice)
        + u16::from(weights.fill_blank)
        != 100
    {
        return Err(AssessmentError::InvalidWeights);
    }
    let knowledge_weight = u16::from(weights.multiple_choice) + u16::from(weights.fill_blank);
    let knowledge_numerator = u16::from(multiple_choice_percent)
        * u16::from(weights.multiple_choice)
        + u16::from(fill_blank_percent) * u16::from(weights.fill_blank);
    let knowledge_percent =
        knowledge_numerator.checked_div(knowledge_weight).map(|value| value as u8).unwrap_or(0);
    let overall_percent = ((u16::from(practical_percent) * u16::from(weights.practical)
        + u16::from(multiple_choice_percent) * u16::from(weights.multiple_choice)
        + u16::from(fill_blank_percent) * u16::from(weights.fill_blank))
        / 100) as u8;
    let critical_ok = !policy.require_all_critical_practical || critical_practical_passed;
    let passed = overall_percent >= policy.overall_percent
        && knowledge_percent >= policy.knowledge_percent
        && critical_ok;
    Ok(CertificationResult {
        practical_percent,
        multiple_choice_percent,
        fill_blank_percent,
        knowledge_percent,
        overall_percent,
        critical_practical_passed,
        passed,
    })
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AssessmentError {
    #[error("answer type does not match question {question_id}")]
    AnswerTypeMismatch { question_id: String },
    #[error("certification weights must sum to 100")]
    InvalidWeights,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_blank_accepts_aliases_and_normalized_case() {
        let question = Question::FillBlank {
            id: "gpu".into(),
            competency: "C4".into(),
            prompt: "srun --gres=____:1".into(),
            blanks: vec![BlankDefinition {
                id: "blank-1".into(),
                accepted: vec![
                    AcceptedAnswer::Literal { value: "gpu:h200".into() },
                    AcceptedAnswer::Literal { value: "gpu".into() },
                ],
                case_insensitive: true,
                trim: true,
                normalize_whitespace: true,
            }],
            points: 2,
            explanation: String::new(),
        };
        let answer =
            Answer::FillBlank { values: BTreeMap::from([("blank-1".into(), " GPU:H200 ".into())]) };
        let score = score_question(&question, &answer).unwrap();
        assert!(score.correct);
    }

    #[test]
    fn certification_requires_critical_practical() {
        let result = finalize_certification(
            &PassPolicy {
                overall_percent: 80,
                knowledge_percent: 70,
                require_all_critical_practical: true,
            },
            &CertificationWeights { practical: 60, multiple_choice: 25, fill_blank: 15 },
            100,
            100,
            100,
            false,
        )
        .unwrap();
        assert!(!result.passed);
    }
}
