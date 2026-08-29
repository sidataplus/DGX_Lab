# Content Authoring Runbook

## Source workflow

1. Write scenario YAML under `scenario-src/`.
2. Write lab YAML and Markdown under `course-src/<course>/labs/`.
3. Add deterministic questions under `question-src/`.
4. Run `python3 scripts/validate_all.py`.
5. Run the Rust scenario compiler once a build host is available.
6. Execute the lab from a clean session and inspect evidence.
7. Freeze revision and record changes.

## Scenario rules

- generic hostnames and paths only;
- every actor action is declarative;
- no embedded shell, JS, HTML, WASM, or executable code;
- stable ID and revision;
- deterministic seed controls randomization;
- at least one observable objective/check;
- faults have educational purpose and recovery path.

## Lab rules

- one central operational story;
- 20–60 minutes;
- objectives use observable verbs;
- each critical step has state/evidence assertion;
- progressive hints teach inspection rather than reveal final command immediately;
- reflection connects simulator behavior to safe shared-cluster practice;
- no exact-command grading when equivalent safe alternatives exist.

## Question rules

- one defensible answer under the stated context;
- distractors are plausible misconceptions, not word games;
- explanations teach the mental model;
- multi-select penalties are explicit;
- fill blanks list accepted aliases deliberately;
- no reliance on obscure version-specific trivia unless the module teaches it;
- every question maps to one primary competency.

## Revision rules

Changing answers, practical checks, scheduler semantics, or certification weights requires a new content revision. Cosmetic typo fixes may retain revision only before release.
