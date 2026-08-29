# Certification Design

## Purpose

Certification demonstrates both declarative knowledge and operational competence. Completion alone does not imply competence, because progress bars have already received far too much authority in education.

## Weighting and gates

| Component | Weight |
|---|---:|
| Practical simulation | 60% |
| Multiple-choice / multi-select | 25% |
| Fill-in-the-blank | 15% |

Pass requirements:

- overall score at least 80%;
- combined knowledge score at least 70%;
- every critical practical competency passed;
- no disqualifying integrity violation configured by the assessment;
- maximum two attempts in the built-in policy.

## Question types

### Single choice

Exactly one explicit option ID is correct. Option order may be deterministically shuffled by assessment seed.

### Multi-select

Several option IDs are correct. Partial credit rewards correct selections and subtracts a bounded explicit penalty for incorrect selections. The algorithm is versioned with the assessment.

### Fill in the blank

Each blank defines explicit accepted literals or numeric answers/tolerances. Matching can normalize case, trim, and whitespace. There is no fuzzy LLM judge deciding that an answer has good vibes.

### Practical task

A scenario defines required end states and evidence, for example:

- request exactly one H200 GPU;
- inspect the resulting environment;
- distinguish Resources from Priority;
- diagnose host OOM;
- resume from the latest valid checkpoint;
- preserve an artifact before timeout.

Alternative valid command sequences receive credit when they produce the required state/evidence.

## Assessment randomization

A deterministic seed selects questions, option order, job IDs, actor names, scenario timing, and safe numerical parameters. The evidence bundle stores seed and revision, permitting exact replay.

## Hints and solutions

Practice supports progressive hints. Certification mode disables hints and solutions by default. Assessment policy may permit a hint with an explicit score consequence, but that is not the built-in v1 certificate.

## Evidence bundle

The `.dgxlab` bundle contains:

- learner-entered display name;
- application/course/exam revisions;
- seed;
- responses and scoring details;
- practical assertion evidence;
- terminal and VFS interaction transcript;
- timestamps in simulated and local display time;
- final result;
- content digest.

## Verification levels

| Level | Claim |
|---|---|
| Standalone | Local score and replayable evidence; identity not independently verified |
| Instructor verified | An instructor reviews and countersigns the evidence |
| Institutionally verified | Deferred trusted identity/signing/proctoring workflow |

A local application under learner control cannot be its own incorruptible examiner. Hashes detect changed evidence under ordinary use; they do not create an external root of trust.

## Certificate outputs

- print-ready standalone HTML;
- Markdown competency report;
- JSON evidence summary;
- complete `.dgxlab` replay bundle.

PDF generation is deferred to platform print-to-PDF or a later reviewed renderer.
