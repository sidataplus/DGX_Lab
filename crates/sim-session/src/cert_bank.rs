//! Embedded certification question subset (English v1).
//! Loaded offline — no network and no host file reads at runtime.

use assessment::{
    finalize_certification, score_question, Answer, BlankDefinition, CertificationResult,
    CertificationWeights, OptionItem, PassPolicy, Question, QuestionScore,
};
use std::collections::BTreeSet;

#[must_use]
pub fn certification_questions() -> Vec<Question> {
    vec![
        Question::SingleChoice {
            id: "mc-001".into(),
            competency: "cluster-fundamentals".into(),
            prompt: "Which Slurm object groups nodes under common scheduling rules?".into(),
            options: opts(&[
                ("a", "Job step"),
                ("b", "Partition"),
                ("c", "Environment module"),
                ("d", "Checkpoint"),
            ]),
            correct: "b".into(),
            points: 2,
            explanation: "A partition groups nodes and applies scheduling/access policies.".into(),
        },
        Question::SingleChoice {
            id: "mc-002".into(),
            competency: "scheduling".into(),
            prompt: "A job is PENDING with reason Resources. What does that most directly mean?"
                .into(),
            options: opts(&[
                ("a", "The script has already failed"),
                ("b", "Eligible resources are currently unavailable"),
                ("c", "The account is invalid"),
                ("d", "The time limit has expired"),
            ]),
            correct: "b".into(),
            points: 2,
            explanation: "Resources means an eligible job cannot currently be allocated.".into(),
        },
        Question::SingleChoice {
            id: "mc-003".into(),
            competency: "scheduling".into(),
            prompt: "Which command is most useful for seeing active and pending jobs?".into(),
            options: opts(&[
                ("a", "squeue"),
                ("b", "sacctmgr"),
                ("c", "nvidia-smi"),
                ("d", "lfs"),
            ]),
            correct: "a".into(),
            points: 2,
            explanation: "squeue presents queued and running jobs.".into(),
        },
        Question::SingleChoice {
            id: "mc-004".into(),
            competency: "accounting".into(),
            prompt: "Which command is designed to inspect completed job accounting?".into(),
            options: opts(&[("a", "sacct"), ("b", "srun"), ("c", "salloc"), ("d", "module")]),
            correct: "a".into(),
            points: 2,
            explanation: "sacct queries accounting records for terminal jobs.".into(),
        },
        Question::SingleChoice {
            id: "mc-005".into(),
            competency: "gpu-isolation".into(),
            prompt: "What should a one-GPU job normally see when device isolation is working?"
                .into(),
            options: opts(&[
                ("a", "All GPUs"),
                ("b", "Only its allocated GPU"),
                ("c", "No GPU until root enables it"),
                ("d", "GPUs on every node"),
            ]),
            correct: "b".into(),
            points: 2,
            explanation: "CUDA_VISIBLE_DEVICES remaps only allocated devices into the job."
                .into(),
        },
        Question::MultiSelect {
            id: "ms-001".into(),
            competency: "batch-jobs".into(),
            prompt: "Which are appropriate places for #SBATCH directives?".into(),
            options: opts(&[
                ("a", "At the top of a batch script before the first command"),
                ("b", "After the first non-comment command line"),
                ("c", "On consecutive comment lines starting with #SBATCH"),
                ("d", "Inside a running interactive bash prompt as shell aliases"),
            ]),
            correct: BTreeSet::from(["a".into(), "c".into()]),
            points: 3,
            incorrect_penalty_basis_points: 500,
            explanation: "Directives are read from leading #SBATCH comments only.".into(),
        },
        Question::FillBlank {
            id: "fb-001".into(),
            competency: "gpu-allocation".into(),
            prompt: "Complete: srun --gres=____:h200:1".into(),
            blanks: vec![BlankDefinition {
                id: "blank-1".into(),
                accepted: vec![assessment::AcceptedAnswer::Literal {
                    value: "gpu".into(),
                }],
                case_insensitive: true,
                trim: true,
                normalize_whitespace: true,
            }],
            points: 2,
            explanation: "GRES GPU requests use the gpu type token.".into(),
        },
        Question::FillBlank {
            id: "fb-002".into(),
            competency: "batch-jobs".into(),
            prompt: "Command that submits a batch script: ____ train.sbatch".into(),
            blanks: vec![BlankDefinition {
                id: "blank-1".into(),
                accepted: vec![assessment::AcceptedAnswer::Literal {
                    value: "sbatch".into(),
                }],
                case_insensitive: true,
                trim: true,
                normalize_whitespace: true,
            }],
            points: 2,
            explanation: "sbatch submits batch work to the scheduler.".into(),
        },
    ]
}

fn opts(items: &[(&str, &str)]) -> Vec<OptionItem> {
    items
        .iter()
        .map(|(id, text)| OptionItem {
            id: (*id).into(),
            text: (*text).into(),
        })
        .collect()
}

#[must_use]
pub fn default_pass_policy() -> PassPolicy {
    PassPolicy {
        overall_percent: 80,
        knowledge_percent: 70,
        require_all_critical_practical: true,
    }
}

#[must_use]
pub fn default_weights() -> CertificationWeights {
    CertificationWeights {
        practical: 60,
        multiple_choice: 25,
        fill_blank: 15,
    }
}

/// Score a full set of answers; practical percent is supplied from the lab engine.
pub fn score_certification(
    answers: &[(String, Answer)],
    practical_percent: u8,
    critical_practical_passed: bool,
) -> Result<(Vec<QuestionScore>, CertificationResult), assessment::AssessmentError> {
    let bank = certification_questions();
    let mut scores = Vec::new();
    let mut mc_earned = 0_u32;
    let mut mc_possible = 0_u32;
    let mut fb_earned = 0_u32;
    let mut fb_possible = 0_u32;
    for question in &bank {
        let answer = answers
            .iter()
            .find(|(id, _)| id == question.id())
            .map(|(_, answer)| answer.clone())
            .unwrap_or_else(|| default_empty_answer(question));
        let score = score_question(question, &answer)?;
        match question {
            Question::SingleChoice { .. } | Question::MultiSelect { .. } => {
                mc_earned = mc_earned.saturating_add(score.earned_milli_points);
                mc_possible = mc_possible.saturating_add(score.possible_milli_points);
            }
            Question::FillBlank { .. } => {
                fb_earned = fb_earned.saturating_add(score.earned_milli_points);
                fb_possible = fb_possible.saturating_add(score.possible_milli_points);
            }
        }
        scores.push(score);
    }
    let mc_percent = percent(mc_earned, mc_possible);
    let fb_percent = percent(fb_earned, fb_possible);
    let result = finalize_certification(
        &default_pass_policy(),
        &default_weights(),
        practical_percent,
        mc_percent,
        fb_percent,
        critical_practical_passed,
    )?;
    Ok((scores, result))
}

fn percent(earned: u32, possible: u32) -> u8 {
    if possible == 0 {
        0
    } else {
        ((earned * 100) / possible) as u8
    }
}

fn default_empty_answer(question: &Question) -> Answer {
    match question {
        Question::SingleChoice { .. } => Answer::SingleChoice {
            option_id: String::new(),
        },
        Question::MultiSelect { .. } => Answer::MultiSelect {
            option_ids: BTreeSet::new(),
        },
        Question::FillBlank { blanks, .. } => Answer::FillBlank {
            values: blanks.iter().map(|blank| (blank.id.clone(), String::new())).collect(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_knowledge_can_pass_with_strong_practical() {
        let bank = certification_questions();
        let mut answers = Vec::new();
        for question in &bank {
            let answer = match question {
                Question::SingleChoice { correct, .. } => Answer::SingleChoice {
                    option_id: correct.clone(),
                },
                Question::MultiSelect { correct, .. } => Answer::MultiSelect {
                    option_ids: correct.clone(),
                },
                Question::FillBlank { blanks, .. } => Answer::FillBlank {
                    values: blanks
                        .iter()
                        .map(|blank| {
                            let value = match &blank.accepted[0] {
                                assessment::AcceptedAnswer::Literal { value } => value.clone(),
                                assessment::AcceptedAnswer::Numeric { value, .. } => {
                                    value.to_string()
                                }
                            };
                            (blank.id.clone(), value)
                        })
                        .collect(),
                },
            };
            answers.push((question.id().to_string(), answer));
        }
        let (scores, result) = score_certification(&answers, 100, true).unwrap();
        assert_eq!(scores.len(), bank.len());
        assert!(result.passed, "overall={}", result.overall_percent);
        assert!(result.knowledge_percent >= 70);
    }
}
