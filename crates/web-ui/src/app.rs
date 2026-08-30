use crate::bridge::{SimBridge, TerminalBuffer};
use crate::learning::{
    active_job_count, active_lab, preferred_job_id, readiness_practical, suggested_command,
    summarize_progress,
};
use crate::persist::{self, PersistedUiState};
use assessment::{Answer, Question};
use dgxlab_contracts::{SimRequest, SimResponse, TerminalKind, UiWorldView};
use leptos::prelude::*;
use sim_session::{BUILTIN_LABS, SimSession, cert_bank, learner_step_meta};
use std::collections::{BTreeMap, BTreeSet};

const DEFAULT_SEED: u64 = 42;
const DEFAULT_LAB_ID: &str = "01-cluster-mental-model";
const SCRIPT_PATH: &str = "train.sbatch";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkspaceMode {
    Learn,
    Practice,
    Assess,
}

#[derive(Clone)]
struct AppRuntime {
    bridge: ArcRwSignal<SimBridge>,
    terminal: RwSignal<TerminalBuffer>,
    world: RwSignal<UiWorldView>,
    selected_job: RwSignal<Option<u64>>,
    announcement: RwSignal<String>,
    mode: RwSignal<WorkspaceMode>,
    script: RwSignal<String>,
    script_status: RwSignal<String>,
}

impl AppRuntime {
    fn persist(&self) {
        if let Ok(session_json) = self.bridge.get_untracked().export_json() {
            persist::save_local(&PersistedUiState {
                session_json,
                terminal_lines: self.terminal.get_untracked().lines.clone(),
                saved_at_ms: self.world.get_untracked().now_ms,
            });
        }
    }

    fn apply(&self, response: SimResponse) {
        self.announcement.set(response_announcement(&response));
        apply_response(response, self.terminal, self.world, self.selected_job);
        self.persist();
    }

    fn request(&self, request: SimRequest) {
        if let Some(response) = self.bridge.try_update(|sim| sim.handle(request)) {
            self.apply(response);
        }
    }

    fn clear_terminal(&self) {
        self.terminal.update(|buffer| buffer.lines.clear());
        self.announcement.set("Terminal transcript cleared from this view.".into());
        self.persist();
    }

    fn reload_script(&self) {
        load_script_into_editor(&self.bridge, self.script, self.script_status);
    }

    fn restart_lab(&self) {
        let lab_id = self.world.get_untracked().lab_id;
        if let Ok(session) = SimSession::open_lab(&lab_id, DEFAULT_SEED) {
            let view = session.view();
            self.bridge.set(SimBridge::from_session(session));
            self.world.set(view);
            self.selected_job.set(None);
            self.terminal.update(|buffer| buffer.lines.clear());
            self.reload_script();
            self.announcement.set("Lab restarted. Your first action is ready.".into());
            self.persist();
        }
    }

    fn open_lab(&self, lab_id: &'static str, scenario: &'static str, target_mode: WorkspaceMode) {
        persist::clear_local();
        if let Ok(session) = SimSession::open_lab(lab_id, DEFAULT_SEED) {
            let view = session.view();
            self.bridge.set(SimBridge::from_session(session));
            self.world.set(view);
        } else {
            self.request(SimRequest::Reset { scenario_id: scenario.into(), seed: DEFAULT_SEED });
        }
        self.selected_job.set(None);
        self.terminal.update(|buffer| buffer.lines.clear());
        self.mode.set(target_mode);
        self.reload_script();
        self.announcement.set(format!("Opened {lab_id}. Follow the recommended next action."));
        self.persist();
    }
}

#[component]
pub fn App() -> impl IntoView {
    let (mut bridge, terminal, world) = restore_or_new();
    let (initial_script, initial_script_status) = read_script_from_bridge(&mut bridge);
    let bridge = ArcRwSignal::new(bridge);
    let terminal = RwSignal::new(terminal);
    let world = RwSignal::new(world);
    let command = RwSignal::new(String::new());
    let learn_command_ref = NodeRef::<leptos::html::Input>::new();
    let practice_command_ref = NodeRef::<leptos::html::Input>::new();
    let mode = RwSignal::new(WorkspaceMode::Learn);
    let script = RwSignal::new(initial_script);
    let script_status = RwSignal::new(initial_script_status);
    let selected_job = RwSignal::new(Option::<u64>::None);
    let light_theme = RwSignal::new(true);
    let announcement =
        RwSignal::new(String::from("DGX Lab is ready. Start with the recommended action."));
    let cert_result = RwSignal::new(Option::<String>::None);
    let cert_answers = RwSignal::new(BTreeMap::<String, String>::new());
    let cert_multi = RwSignal::new(BTreeMap::<String, BTreeSet<String>>::new());
    let runtime = AppRuntime {
        bridge,
        terminal,
        world,
        selected_job,
        announcement,
        mode,
        script,
        script_status,
    };

    let submit = {
        let runtime = runtime.clone();
        move |_| {
            let trimmed = command.get().trim().to_string();
            if trimmed.is_empty() {
                announcement.set("Type or stage a simulated command first.".into());
                return;
            }
            command.set(String::new());
            runtime.request(SimRequest::ExecuteCommand { command: trimmed });
        }
    };

    let clear_terminal = {
        let runtime = runtime.clone();
        move |_| runtime.clear_terminal()
    };

    let reset = {
        let runtime = runtime.clone();
        move |_| runtime.restart_lab()
    };

    let open_lab = {
        let runtime = runtime.clone();
        move |lab_id: &'static str, scenario: &'static str, target_mode: WorkspaceMode| {
            let runtime = runtime.clone();
            move |_| runtime.open_lab(lab_id, scenario, target_mode)
        }
    };

    let set_speed = {
        let runtime = runtime.clone();
        move |multiplier: u32| {
            let runtime = runtime.clone();
            move |_| runtime.request(SimRequest::SetClockSpeed { multiplier })
        }
    };
    let toggle_pause = {
        let runtime = runtime.clone();
        move |_| {
            let request = if runtime.world.get_untracked().paused {
                SimRequest::Resume
            } else {
                SimRequest::Pause
            };
            runtime.request(request);
        }
    };
    let advance = {
        let runtime = runtime.clone();
        move |delta_ms: u64| {
            let runtime = runtime.clone();
            move |_| runtime.request(SimRequest::AdvanceClock { delta_ms })
        }
    };

    let save_script = {
        let runtime = runtime.clone();
        move |_| {
            let content = script.get();
            runtime.request(SimRequest::WriteVfs { path: SCRIPT_PATH.into(), content });
            script_status.set("Saved to virtual /home/learner/train.sbatch".into());
            announcement.set("Batch script saved in the virtual filesystem.".into());
        }
    };
    let submit_script = {
        let runtime = runtime.clone();
        move |_| {
            let content = script.get();
            runtime.request(SimRequest::WriteVfs { path: SCRIPT_PATH.into(), content });
            runtime
                .request(SimRequest::ExecuteCommand { command: format!("sbatch {SCRIPT_PATH}") });
            script_status.set("Submitted sbatch train.sbatch".into());
        }
    };

    let grade_runtime = runtime.clone();
    let grade_cert = move |_| {
        let snapshot = grade_runtime.bridge.with_untracked(SimBridge::view);
        let capstone_open = snapshot.lab_id == "12-capstone";
        let critical_ok = grade_runtime.bridge.with_untracked(SimBridge::critical_practical_passed);
        let (practical, critical_ok) =
            readiness_practical(&snapshot.lab_id, snapshot.practical_percent, critical_ok);
        let singles = cert_answers.get();
        let multis = cert_multi.get();
        let questions = cert_bank::certification_questions();
        let mut answers = Vec::new();
        let mut answered = 0_usize;

        for question in &questions {
            let answer = match question {
                Question::SingleChoice { id, .. } => {
                    let value = singles.get(id).cloned().unwrap_or_default();
                    answered += usize::from(!value.is_empty());
                    Answer::SingleChoice { option_id: value }
                }
                Question::MultiSelect { id, .. } => {
                    let values = multis.get(id).cloned().unwrap_or_default();
                    answered += usize::from(!values.is_empty());
                    Answer::MultiSelect { option_ids: values }
                }
                Question::FillBlank { id, blanks, .. } => {
                    let values = blanks
                        .iter()
                        .map(|blank| {
                            let key = format!("{id}:{}", blank.id);
                            (blank.id.clone(), singles.get(&key).cloned().unwrap_or_default())
                        })
                        .collect::<BTreeMap<_, _>>();
                    answered += usize::from(values.values().any(|value| !value.trim().is_empty()));
                    Answer::FillBlank { values }
                }
            };
            answers.push((question.id().to_string(), answer));
        }

        match cert_bank::score_certification(&answers, practical, critical_ok) {
            Ok((scores, result)) => {
                let correct = scores.iter().filter(|score| score.correct).count();
                let review = scores
                    .iter()
                    .map(|score| {
                        let explanation = questions
                            .iter()
                            .find(|question| question.id() == score.question_id)
                            .map(question_explanation)
                            .unwrap_or("Review this concept before trying again.");
                        format!(
                            "{} {}: {}",
                            if score.correct { "Correct" } else { "Review" },
                            score.question_id,
                            explanation
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let remediation = if result.passed {
                    "You met the local capstone readiness gates.".to_string()
                } else {
                    let mut actions = Vec::new();
                    if !capstone_open {
                        actions.push("open Module 12 and complete the capstone practical");
                    } else if result.practical_percent < 80 || !result.critical_practical_passed {
                        actions
                            .push("return to Learn and complete the capstone's critical actions");
                    }
                    if result.knowledge_percent < 70 {
                        actions
                            .push("review the explanations below, then retry the knowledge check");
                    }
                    format!("Next: {}.", actions.join("; "))
                };
                let summary = format!(
                    "{}\nOverall {}% / Knowledge {}% / Practical {}%\nAnswered {}/{} / {}/{} correct\n{}\n\n{}",
                    if result.passed { "PASSED" } else { "NOT YET" },
                    result.overall_percent,
                    result.knowledge_percent,
                    result.practical_percent,
                    answered,
                    questions.len(),
                    correct,
                    scores.len(),
                    remediation,
                    review
                );
                announcement.set(format!(
                    "Assessment scored. Overall {} percent. {}",
                    result.overall_percent,
                    if result.passed { "Passed." } else { "Not yet passed." }
                ));
                cert_result.set(Some(summary));
            }
            Err(error) => {
                announcement.set("Assessment could not be scored.".into());
                cert_result.set(Some(format!("Scoring error: {error}")));
            }
        }
    };
    let hint_runtime = runtime.clone();
    let reload_runtime = runtime.clone();

    view! {
        <div class=move || if light_theme.get() { "app-shell theme-light" } else { "app-shell" }>
            <a class="skip-link" href="#main-content">"Skip to learning workspace"</a>
            <div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
                {move || announcement.get()}
            </div>

            <header class="topbar">
                <div class="brand-lockup">
                    <span class="brand-mark" aria-hidden="true">"DL"</span>
                    <div>
                        <strong class="brand">"DGX Lab"</strong>
                        <span class="subtitle">"Shared GPU systems, safely simulated"</span>
                    </div>
                </div>
                <div class="topbar-status" aria-label="Application status">
                    <span class="pill success">"Runs locally"</span>
                    <span class="pill accent">"No real cluster"</span>
                    <button type="button" class="icon-button"
                        aria-label=move || if light_theme.get() { "Use dark theme" } else { "Use light theme" }
                        title=move || if light_theme.get() { "Use dark theme" } else { "Use light theme" }
                        on:click=move |_| light_theme.update(|value| *value = !*value)>
                        <span aria-hidden="true">{move || if light_theme.get() { "◐" } else { "○" }}</span>
                        <span class="theme-label">{move || if light_theme.get() { "Dark view" } else { "Light view" }}</span>
                    </button>
                </div>
            </header>

            <nav class="journey-bar" aria-label="Learning journey">
                <div class="journey-tabs" role="group" aria-label="Choose a learning workspace">
                    <button type="button" class=move || tab_class(mode.get() == WorkspaceMode::Learn)
                        aria-pressed=move || bool_text(mode.get() == WorkspaceMode::Learn)
                        on:click=move |_| mode.set(WorkspaceMode::Learn)>
                        <span class="tab-number">"01"</span><span><strong>"Learn"</strong><small>"Step-by-step lab"</small></span>
                    </button>
                    <button type="button" class=move || tab_class(mode.get() == WorkspaceMode::Practice)
                        aria-pressed=move || bool_text(mode.get() == WorkspaceMode::Practice)
                        on:click=move |_| mode.set(WorkspaceMode::Practice)>
                        <span class="tab-number">"02"</span><span><strong>"Practice"</strong><small>"Open sandbox"</small></span>
                    </button>
                    <button type="button" class=move || tab_class(mode.get() == WorkspaceMode::Assess)
                        aria-pressed=move || bool_text(mode.get() == WorkspaceMode::Assess)
                        on:click=move |_| mode.set(WorkspaceMode::Assess)>
                        <span class="tab-number">"03"</span><span><strong>"Assess"</strong><small>"Readiness review"</small></span>
                    </button>
                </div>

                <div class="clock-controls" hidden=move || mode.get() == WorkspaceMode::Assess
                    role="group" aria-label="Simulation clock">
                    <span class="sim-time">{move || format!("Lab time {}", format_sim_time(world.get().now_ms))}</span>
                    <div class="speed-group" role="group" aria-label="Clock speed">
                        <button type="button" class=move || speed_class(world.get().clock_multiplier == 1)
                            aria-pressed=move || bool_text(world.get().clock_multiplier == 1) on:click=set_speed(1)>"1×"</button>
                        <button type="button" class=move || speed_class(world.get().clock_multiplier == 10)
                            aria-pressed=move || bool_text(world.get().clock_multiplier == 10) on:click=set_speed(10)>"10×"</button>
                        <button type="button" class=move || speed_class(world.get().clock_multiplier == 60)
                            aria-pressed=move || bool_text(world.get().clock_multiplier == 60) on:click=set_speed(60)>"60×"</button>
                    </div>
                    <button type="button" class="quiet-button" on:click=toggle_pause>
                        {move || if world.get().paused { "Resume" } else { "Pause" }}
                    </button>
                    <details class="time-menu">
                        <summary aria-label="More simulation time controls">"Advance time"</summary>
                        <div class="time-menu-items">
                            <button type="button" on:click=advance(60_000)>"+1 minute"</button>
                            <button type="button" on:click=advance(600_000)>"+10 minutes"</button>
                        </div>
                    </details>
                </div>
            </nav>

            <main id="main-content">
                <section id="learn-workspace" class="workspace learn-layout"
                    aria-label="Guided learning workspace" hidden=move || mode.get() != WorkspaceMode::Learn>
                    <aside class="card mission-card" aria-labelledby="mission-title">
                        {move || {
                            let snapshot = world.get();
                            let progress = summarize_progress(&snapshot.lab_steps);
                            let (lab_index, lab) = active_lab(&snapshot.lab_id).unwrap_or((0, &BUILTIN_LABS[0]));
                            let next_id = progress.next_step.as_ref().map(|step| step.id.clone());
                            let next_action = progress.next_step.clone().map(|step| {
                                let command_text = suggested_command(
                                    lab.id,
                                    &step.id,
                                    &snapshot.jobs,
                                    &snapshot.checkpoint_paths,
                                );
                                let evidence = learner_step_meta(lab.id, &step.id)
                                    .map(|meta| meta.evidence)
                                    .unwrap_or("Observe how the terminal and cluster state agree.");
                                let is_terminal_command = command_text
                                    .as_deref()
                                    .is_some_and(|value| !value.starts_with("Use the "));
                                view! {
                                    <div class="next-action" aria-labelledby="next-action-title">
                                        <span class="eyebrow">"DO THIS NEXT"</span>
                                        <h2 id="next-action-title">{step.label}</h2>
                                        <p>{evidence}</p>
                                        {command_text.clone().map(|value| view! { <code class="command-preview">{value}</code> })}
                                        {is_terminal_command.then(|| {
                                            let value = command_text.clone().unwrap_or_default();
                                            view! {
                                                <button type="button" class="primary" on:click=move |_| {
                                                    command.set(value.to_string());
                                                    announcement.set("Suggested command placed in the terminal. Review it, then run it.".into());
                                                    if let Some(input) = learn_command_ref.get() {
                                                        let _ = input.focus();
                                                    }
                                                }>"Place in terminal"</button>
                                            }
                                        })}
                                    </div>
                                }
                            });

                            view! {
                                <div class="mission-heading">
                                    <div><p class="eyebrow">{format!("{} / LAB {:02} OF {}", lab.track.to_uppercase(), lab_index + 1, BUILTIN_LABS.len())}</p>
                                        <h1 id="mission-title">{lab.title}</h1></div>
                                    <span class="duration">{format!("{} min", lab.estimated_minutes)}</span>
                                </div>
                                <p class="mission-summary">{lab.summary}</p>
                                <div class="progress-heading"><span>{format!("{} of {} actions complete", progress.completed, progress.total)}</span>
                                    <strong>{format!("{}% practical evidence", snapshot.practical_percent)}</strong></div>
                                <div class="progress-track" role="progressbar" aria-label="Lab actions complete"
                                    aria-valuemin="0" aria-valuemax="100" aria-valuenow=progress.percent.to_string()>
                                    <span class="progress-fill" style=format!("width: {}%", progress.percent)></span>
                                </div>
                                {if snapshot.lab_complete {
                                    view! { <div class="completion-card" role="status"><span aria-hidden="true">"✓"</span>
                                    <div><strong>"Simulator practice complete"</strong><p>"You captured the required evidence. Before moving on, explain what changed and how you would apply it on a real cluster."</p></div></div> }.into_any()
                                } else {
                                    next_action.map(|item| item.into_any()).unwrap_or_else(|| view! {
                                        <div class="next-action"><h2>"Explore the simulator"</h2><p>"Use the terminal and compare every command with the visual cluster state."</p></div>
                                    }.into_any())
                                }}
                                <div class="section-heading"><h2>"Lab path"</h2><span>"Updates from simulator evidence"</span></div>
                                <ol class="progress-list">
                                    {snapshot.lab_steps.into_iter().enumerate().map(|(index, step)| {
                                        let is_current = !step.complete && next_id.as_deref() == Some(step.id.as_str());
                                        let status = if step.complete { "Complete" } else if is_current { "Now" } else { "Later" };
                                        view! {
                                            <li class=step_class(step.complete, is_current) aria-current=if is_current { "step" } else { "false" }>
                                                <span class="step-mark" aria-hidden="true">{if step.complete { "✓".into() } else { format!("{}", index + 1) }}</span>
                                                <span class="step-copy"><strong>{step.label}</strong>{step.critical.then(|| view!{<small>"Required evidence"</small>})}</span>
                                                <span class="step-status">{status}</span>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ol>
                            }
                        }}

                        <div class="coaching-actions">
                            <button type="button" class="secondary" disabled=move || { world.get().hint_level >= 3 } on:click=move |_| {
                                hint_runtime.request(SimRequest::UseHint);
                                announcement.set("A progressive hint was revealed.".into());
                            }>
                                {move || if world.get().hint_level >= 3 { "All hints revealed".into() } else { format!("Reveal hint {}", world.get().hint_level + 1) }}
                            </button>
                            <span class="coaching-note">"Hints are recorded separately from correctness."</span>
                        </div>
                        {move || world.get().hint_text.map(|text| view! {
                            <div class="notice hint" role="note"><strong>"Hint"</strong><p>{text}</p></div>
                        })}

                        <details class="course-picker">
                            <summary>"Browse all 12 labs"</summary>
                            <div class="course-list">
                                {BUILTIN_LABS.iter().enumerate().map(|(index, lab)| {
                                    let id = lab.id;
                                    let scenario = lab.scenario;
                                    view! {
                                        <button type="button" class=move || lab_button_class(world.get().lab_id == id)
                                            aria-current=move || if world.get().lab_id == id { "page" } else { "false" }
                                            on:click=open_lab(id, scenario, WorkspaceMode::Learn)>
                                            <span class="lab-number">{format!("{:02}", index + 1)}</span>
                                            <span><strong>{lab.title}</strong><small>{format!("{} / {} min", lab.track, lab.estimated_minutes)}</small></span>
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </details>
                        <details class="restart-menu">
                            <summary>"Restart options"</summary>
                            <p>"Restarting clears this lab's transcript and practical progress."</p>
                            <button type="button" class="danger-button" on:click=reset>"Restart this lab"</button>
                        </details>
                    </aside>

                    <section class="center-column" aria-label="Guided practice">
                        <section class="card terminal-card" aria-labelledby="learn-terminal-title">
                            <div class="card-header"><div><p class="eyebrow">"SAFE SIMULATOR"</p><h2 id="learn-terminal-title">"Practice terminal"</h2></div>
                                <button type="button" class="quiet-button" on:click=clear_terminal.clone()>"Clear transcript"</button></div>
                            <TerminalView terminal=terminal world=world />
                            <CommandComposer input_id="learn-command" input_ref=learn_command_ref command=command submit=submit.clone() />
                            <details class="command-shelf">
                                <summary>"Need a safe starting point?"</summary>
                                <p>"These read-only commands help you observe before changing state."</p>
                                <div class="command-chips">
                                    {[("Cluster", "sinfo"), ("Queue", "squeue"), ("Accounting", "sacct"), ("Help", "help")]
                                        .into_iter().map(|(label, value)| view! {
                                            <button type="button" on:click=move |_| command.set(value.into())>{format!("{label}: {value}")}</button>
                                        }).collect_view()}
                                </div>
                            </details>
                            <p class="simulation-note"><span aria-hidden="true">"◇"</span> " Everything here is simulated. No command can reach a real cluster."</p>
                        </section>
                    </section>
                    <ClusterPanel world=world selected_job=selected_job />
                </section>

                <section id="practice-workspace" class="workspace practice-layout"
                    aria-label="Free practice workspace" hidden=move || mode.get() != WorkspaceMode::Practice>
                    <aside class="card practice-brief" aria-labelledby="practice-title">
                        <p class="eyebrow">"OPEN PRACTICE"</p>
                        <h1 id="practice-title">"Turn a command into a reliable job"</h1>
                        <p>"Edit a virtual batch script, submit it, then explain what the scheduler did. Your current course lab stays connected to the same simulator state."</p>
                        <div class="practice-loop" aria-label="Recommended practice loop">
                            <div><span>"1"</span><p><strong>"Predict"</strong>" what will happen before you submit."</p></div>
                            <div><span>"2"</span><p><strong>"Observe"</strong>" queue, GPUs, and terminal evidence."</p></div>
                            <div><span>"3"</span><p><strong>"Explain"</strong>" any wait or failure before changing the script."</p></div>
                        </div>
                        <div class="section-heading"><h2>"Focused drills"</h2></div>
                        <button type="button" class="drill-button" on:click=open_lab("06-batch-jobs", "dgx-h200-8", WorkspaceMode::Practice)>
                            <strong>"Batch basics"</strong><span>"Submit, inspect logs, review accounting"</span></button>
                        <button type="button" class="drill-button" on:click=open_lab("07-pending-reasons", "dgx-contended", WorkspaceMode::Practice)>
                            <strong>"Queue contention"</strong><span>"Read pending reasons and wait productively"</span></button>
                        <button type="button" class="drill-button" on:click=open_lab("09-failure-resume", "dgx-degraded", WorkspaceMode::Practice)>
                            <strong>"Failure recovery"</strong><span>"Use logs and checkpoints as evidence"</span></button>
                        <div class="notice"><strong>"Current context"</strong><p>{move || {
                            let snapshot = world.get();
                            active_lab(&snapshot.lab_id).map(|(_, lab)| format!("{} / {}", lab.title, snapshot.scenario_id)).unwrap_or(snapshot.scenario_id)
                        }}</p></div>
                    </aside>

                    <section class="center-column practice-center" aria-label="Script and terminal">
                        <section class="card editor-card" aria-labelledby="editor-title">
                            <div class="card-header"><div><p class="eyebrow">"VIRTUAL FILE"</p><h2 id="editor-title">"train.sbatch"</h2></div><span class="save-state">"Local draft"</span></div>
                            <label class="sr-only" for="script-editor">"Virtual batch script"</label>
                            <textarea id="script-editor" class="script-editor" prop:value=move || script.get()
                                on:input=move |event| script.set(event_target_value(&event)) spellcheck="false"></textarea>
                            <div class="button-row">
                                <button type="button" class="secondary" on:click=move |_| reload_runtime.reload_script()>"Reload virtual file"</button>
                                <button type="button" class="secondary" on:click=save_script>"Save draft"</button>
                                <button type="button" class="primary" on:click=submit_script>"Submit with sbatch"</button>
                            </div>
                            <p class="form-status" role="status">{move || script_status.get()}</p>
                        </section>
                        <section class="card terminal-card compact-terminal" aria-labelledby="practice-terminal-title">
                            <div class="card-header"><div><p class="eyebrow">"OBSERVE & EXPLAIN"</p><h2 id="practice-terminal-title">"Terminal"</h2></div>
                                <button type="button" class="quiet-button" on:click=clear_terminal>"Clear transcript"</button></div>
                            <TerminalView terminal=terminal world=world />
                            <CommandComposer input_id="practice-command" input_ref=practice_command_ref command=command submit=submit />
                        </section>
                    </section>
                    <ClusterPanel world=world selected_job=selected_job />
                </section>

                <section id="assess-workspace" class="workspace assess-layout"
                    aria-label="Assessment workspace" hidden=move || mode.get() != WorkspaceMode::Assess>
                    <aside class="card assessment-overview" aria-labelledby="assessment-overview-title">
                        <p class="eyebrow">"LOCAL READINESS REVIEW"</p>
                        <h1 id="assessment-overview-title">"Show what you can do"</h1>
                        <p>"This local assessment combines capstone practical evidence with an offline knowledge check."</p>
                        <div class="readiness-score">
                            <span>{move || {
                                let snapshot = world.get();
                                format!("{}%", readiness_practical(&snapshot.lab_id, snapshot.practical_percent, snapshot.lab_complete).0)
                            }}</span>
                            <div><strong>"Capstone evidence"</strong><small>{move || {
                                let snapshot = world.get();
                                if snapshot.lab_id != "12-capstone" {
                                    "Open Module 12 to earn practical credit"
                                } else if snapshot.lab_complete {
                                    "Capstone actions complete"
                                } else {
                                    "Capstone actions still needed"
                                }
                            }}</small></div>
                        </div>
                        <div class="readiness-list">
                            <div><span aria-hidden="true">"60"</span><p><strong>"Practical"</strong>"Module 12 capstone evidence"</p></div>
                            <div><span aria-hidden="true">"25"</span><p><strong>"Concepts"</strong>"Choice and interpretation"</p></div>
                            <div><span aria-hidden="true">"15"</span><p><strong>"Commands"</strong>"Fill-in-the-blank knowledge"</p></div>
                        </div>
                        <div class="notice"><strong>"Before scoring"</strong><p>"Answer every item. Unanswered items receive no credit, and the review will tell you what to revisit."</p></div>
                        <button type="button" class="primary wide-button"
                            hidden=move || world.get().lab_id == "12-capstone"
                            on:click=open_lab("12-capstone", "dgx-contended", WorkspaceMode::Learn)>
                            "Open Module 12 capstone"
                        </button>
                        <button type="button" class="secondary wide-button" on:click=move |_| mode.set(WorkspaceMode::Learn)>"Return to guided practice"</button>
                    </aside>

                    <section class="card cert-panel" aria-labelledby="knowledge-check-title">
                        <div class="card-header"><div><p class="eyebrow">"KNOWLEDGE REVIEW"</p><h2 id="knowledge-check-title">"Explain the scheduler, not just the syntax"</h2></div><span class="pill neutral">"8 items"</span></div>
                        <p class="lede">"Take your time. Your answers stay on this device and no identity is independently verified."</p>
                        <div class="cert-questions">
                            {cert_bank::certification_questions().into_iter().enumerate().map(|(index, question)| {
                                cert_question_view(index + 1, question, cert_answers, cert_multi)
                            }).collect_view()}
                        </div>
                        <div class="assessment-submit"><p>"Scoring uses the approved 60 / 25 / 15 weighting and your authoritative practical state."</p>
                            <button type="button" class="primary" on:click=grade_cert>"Score my readiness"</button></div>
                        {move || cert_result.get().map(|text| view! { <pre class="cert-result" role="status" tabindex="-1">{text}</pre> })}
                    </section>

                    <aside class="card assessment-guide" aria-labelledby="assessment-guide-title">
                        <p class="eyebrow">"HOW TO USE YOUR RESULT"</p>
                        <h2 id="assessment-guide-title">"A score is a route, not a verdict"</h2>
                        <ol>
                            <li><strong>"Read every explanation."</strong><span>"Correct guesses still deserve a mental model."</span></li>
                            <li><strong>"Return to one focused lab."</strong><span>"Practice the weakest concept in the simulator."</span></li>
                            <li><strong>"Retry from evidence."</strong><span>"Explain what changed before you rescore."</span></li>
                        </ol>
                        <div class="trust-card"><span aria-hidden="true">"◇"</span><div><strong>"Standalone local evidence"</strong><p>"No account, telemetry, network, or identity verification."</p></div></div>
                        <details class="scoring-details"><summary>"Pass gates"</summary><ul><li>"80% overall"</li><li>"70% knowledge"</li><li>"All critical practical actions"</li></ul></details>
                    </aside>
                </section>
            </main>

            <footer class="app-footer"><span>"Progress is stored in this browser."</span>
                <span>{move || format!("Scenario {} / seed {}", world.get().scenario_id, world.get().seed)}</span></footer>
        </div>
    }
}

#[component]
fn CommandComposer(
    input_id: &'static str,
    input_ref: NodeRef<leptos::html::Input>,
    command: RwSignal<String>,
    submit: impl Fn(()) + 'static + Clone,
) -> impl IntoView {
    view! {
        <form class="terminal-input-row" on:submit=move |event| { event.prevent_default(); submit(()); }>
            <label class="sr-only" for=input_id>"Simulated command"</label>
            <span class="prompt-prefix" aria-hidden="true">"$"</span>
            <input id=input_id node_ref=input_ref type="text" autocomplete="off" spellcheck="false"
                placeholder="Type a command or use the recommended action" prop:value=move || command.get()
                on:input=move |event| command.set(event_target_value(&event)) />
            <button type="submit" class="primary">"Run command"</button>
        </form>
    }
}

#[component]
fn ClusterPanel(
    world: RwSignal<UiWorldView>,
    selected_job: RwSignal<Option<u64>>,
) -> impl IntoView {
    view! {
        <aside class="card cluster-card" aria-label="Virtual cluster">
            <div class="card-header"><div><p class="eyebrow">"CLUSTER EVIDENCE"</p><h2>"Virtual cluster"</h2></div>
                <span class="node-state">{move || world.get().node_status}</span></div>
            <p class="cluster-summary">{move || {
                let snapshot = world.get();
                format!("8 simulated H200 GPUs / {} active job(s)", active_job_count(&snapshot.jobs))
            }}</p>
            <ClusterView world=world />
            <details class="evidence-drawer" open>
                <summary>"Jobs and explanations"</summary>
                <JobList world=world selected_job=selected_job />
                <PendingPanel world=world selected_job=selected_job />
                <DiagnosePanel world=world selected_job=selected_job />
            </details>
        </aside>
    }
}

fn tab_class(active: bool) -> &'static str {
    if active { "journey-tab active" } else { "journey-tab" }
}
fn speed_class(active: bool) -> &'static str {
    if active { "speed-button active" } else { "speed-button" }
}
fn lab_button_class(active: bool) -> &'static str {
    if active { "lab-button active" } else { "lab-button" }
}
fn step_class(complete: bool, current: bool) -> &'static str {
    if complete {
        "step done"
    } else if current {
        "step current"
    } else {
        "step"
    }
}
fn bool_text(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn response_announcement(response: &SimResponse) -> String {
    match response {
        SimResponse::CommandResult { lines, state, .. } => {
            let outcome = lines
                .iter()
                .rev()
                .find(|line| !line.text.trim().is_empty())
                .map(|line| line.text.replace('\n', " "))
                .unwrap_or_else(|| "Command completed".into());
            format!("{outcome}. Practical evidence {} percent.", state.practical_percent)
        }
        SimResponse::State { state, .. } | SimResponse::Ready { state, .. } => format!(
            "Simulation updated. Time {}. Practical evidence {} percent.",
            format_sim_time(state.now_ms),
            state.practical_percent
        ),
        SimResponse::FileContent { path, .. } => format!("Loaded virtual file {path}."),
        SimResponse::Error { message, .. } => format!("Simulator message: {message}"),
    }
}

fn question_explanation(question: &Question) -> &str {
    match question {
        Question::SingleChoice { explanation, .. }
        | Question::MultiSelect { explanation, .. }
        | Question::FillBlank { explanation, .. } => explanation,
    }
}

fn cert_question_view(
    number: usize,
    question: Question,
    answers: RwSignal<BTreeMap<String, String>>,
    multi: RwSignal<BTreeMap<String, BTreeSet<String>>>,
) -> AnyView {
    match question {
        Question::SingleChoice { id, prompt, options, points, .. } => view! {
            <fieldset class="cert-q">
                <legend><span>{format!("{:02}", number)}</span><strong>{prompt}</strong><small>{format!("{points} points")}</small></legend>
                <div class="cert-options">{options.into_iter().map(|option| {
                    let question_id = id.clone(); let option_id = option.id.clone();
                    view! { <label class="cert-option"><input type="radio" name=id.clone() value=option.id.clone()
                        on:change=move |_| answers.update(|map| { map.insert(question_id.clone(), option_id.clone()); }) />
                        <span><small>{option.id.to_uppercase()}</small>{option.text}</span></label> }
                }).collect_view()}</div>
            </fieldset>
        }.into_any(),
        Question::MultiSelect { id, prompt, options, points, .. } => view! {
            <fieldset class="cert-q">
                <legend><span>{format!("{:02}", number)}</span><strong>{prompt}</strong><small>{format!("{points} points, choose all that apply")}</small></legend>
                <div class="cert-options">{options.into_iter().map(|option| {
                    let question_id = id.clone(); let option_id = option.id.clone();
                    view! { <label class="cert-option"><input type="checkbox" value=option.id.clone() on:change=move |event| {
                        let checked = event_target_checked(&event); multi.update(|map| { let set = map.entry(question_id.clone()).or_default();
                            if checked { set.insert(option_id.clone()); } else { set.remove(&option_id); } }); } />
                        <span><small>{option.id.to_uppercase()}</small>{option.text}</span></label> }
                }).collect_view()}</div>
            </fieldset>
        }.into_any(),
        Question::FillBlank { id, prompt, blanks, points, .. } => view! {
            <fieldset class="cert-q">
                <legend><span>{format!("{:02}", number)}</span><strong>{prompt}</strong><small>{format!("{points} points")}</small></legend>
                <div class="blank-grid">{blanks.into_iter().map(|blank| {
                    let key = format!("{id}:{}", blank.id); let input_id = format!("answer-{id}-{}", blank.id);
                    let label_for = input_id.clone();
                    view! { <label for=label_for><span>{format!("Answer for {}", blank.id)}</span>
                        <input id=input_id type="text" autocomplete="off" on:input=move |event| {
                            answers.update(|map| { map.insert(key.clone(), event_target_value(&event)); }); } /></label> }
                }).collect_view()}</div>
            </fieldset>
        }.into_any(),
    }
}

fn load_script_into_editor(
    bridge: &ArcRwSignal<SimBridge>,
    script: RwSignal<String>,
    script_status: RwSignal<String>,
) {
    bridge.update(|sim| match sim.handle(SimRequest::ReadVfs { path: SCRIPT_PATH.into() }) {
        SimResponse::FileContent { content, path, .. } => {
            script.set(content);
            script_status.set(format!("Loaded {path}"));
        }
        SimResponse::Error { message, .. } => script_status.set(message),
        _ => {}
    });
}

fn read_script_from_bridge(bridge: &mut SimBridge) -> (String, String) {
    match bridge.handle(SimRequest::ReadVfs { path: SCRIPT_PATH.into() }) {
        SimResponse::FileContent { content, path, .. } => (content, format!("Loaded {path}")),
        SimResponse::Error { message, .. } => (String::new(), message),
        _ => (String::new(), "Your editor uses only the simulator's virtual filesystem.".into()),
    }
}

fn restore_or_new() -> (SimBridge, TerminalBuffer, UiWorldView) {
    if let Some(saved) = persist::load_local()
        && let Ok(session) = SimSession::import_json(&saved.session_json)
    {
        let bridge = SimBridge::from_session(session);
        let view = bridge.view();
        return (bridge, TerminalBuffer { lines: saved.terminal_lines }, view);
    }
    let session = SimSession::open_lab(DEFAULT_LAB_ID, DEFAULT_SEED).expect("built-in lab");
    let bridge = SimBridge::from_session(session);
    let view = bridge.view();
    (bridge, TerminalBuffer::default(), view)
}

fn apply_response(
    response: SimResponse,
    terminal: RwSignal<TerminalBuffer>,
    world: RwSignal<UiWorldView>,
    selected_job: RwSignal<Option<u64>>,
) {
    match response {
        SimResponse::CommandResult { state, lines, .. } => {
            terminal.update(|buffer| buffer.lines.extend(lines));
            if selected_job.get_untracked().is_none()
                && let Some(job_id) = preferred_job_id(&state.jobs)
            {
                selected_job.set(Some(job_id));
            }
            world.set(state);
        }
        SimResponse::State { state, .. } | SimResponse::Ready { state, .. } => world.set(state),
        SimResponse::FileContent { .. } => {}
        SimResponse::Error { message, .. } => terminal.update(|buffer| {
            buffer.lines.push(dgxlab_contracts::TerminalLine::stderr(message));
        }),
    }
}

fn format_sim_time(ms: u64) -> String {
    let seconds = ms / 1_000;
    format!("{:02}:{:02}:{:02}", seconds / 3_600, (seconds % 3_600) / 60, seconds % 60)
}

#[component]
fn TerminalView(terminal: RwSignal<TerminalBuffer>, world: RwSignal<UiWorldView>) -> impl IntoView {
    view! { <pre class="terminal-body" role="log" aria-label="Simulated terminal transcript" aria-live="off" tabindex="0">
        {move || { let buffer = terminal.get(); if buffer.lines.is_empty() {
            format!("{}\n\nWelcome. Start with the recommended action, or run `help`.\n", world.get().prompt)
        } else { buffer.lines.iter().map(|line| match line.kind {
            TerminalKind::Input => format!("$ {}", line.text), TerminalKind::Stderr => format!("! {}", line.text), _ => line.text.clone(),
        }).collect::<Vec<_>>().join("\n") } }}
    </pre> }
}

#[component]
fn ClusterView(world: RwSignal<UiWorldView>) -> impl IntoView {
    view! { <div class="gpu-grid" role="list" aria-label="GPU allocation state">
        {move || world.get().gpus.into_iter().map(|gpu| {
            let owner = gpu.owner_job_id.map(|id| format!("Job {id}")).unwrap_or_else(|| "Available".into());
            let class = gpu_class(&gpu.status, gpu.owner_job_id.is_some());
            let accessible = format!("GPU {}, {}, {}, {}", gpu.index, gpu.model, gpu.status, owner);
            view! { <div class=class role="listitem" aria-label=accessible><div><strong>{format!("GPU {}", gpu.index)}</strong>
                <span class="gpu-status">{gpu.status}</span></div><span>{gpu.model}</span><small>{owner}</small></div> }
        }).collect_view()}
    </div> }
}

fn gpu_class(status: &str, allocated: bool) -> &'static str {
    if status.eq_ignore_ascii_case("failed") || status.eq_ignore_ascii_case("warning") {
        "gpu-tile warning"
    } else if allocated {
        "gpu-tile allocated"
    } else {
        "gpu-tile"
    }
}

#[component]
fn JobList(world: RwSignal<UiWorldView>, selected_job: RwSignal<Option<u64>>) -> impl IntoView {
    view! { <section class="job-list" aria-label="Jobs">
        <div class="subsection-heading"><h3>"Jobs"</h3><span>{move || format!("{} total", world.get().jobs.len())}</span></div>
        {move || { let jobs = world.get().jobs; if jobs.is_empty() {
            view! { <p class="empty-state">"No jobs yet. Submit work and its scheduler state will appear here."</p> }.into_any()
        } else { view! { <ul class="job-ul">{jobs.into_iter().map(|job| {
            let id = job.id; let selected = selected_job.get() == Some(id);
            let reason = job.pending_reason.clone().unwrap_or_else(|| "No wait reason".into());
            let state_class = status_class(&job.status);
            let status = job.status.clone();
            view! { <li><button type="button" class=if selected { "job-button selected" } else { "job-button" }
                aria-pressed=bool_text(selected) on:click=move |_| selected_job.set(Some(id))>
                <span><strong>{format!("Job {}", job.id)}</strong><small>{job.user}</small></span>
                <span class=state_class>{status}</span>
                <span><strong>{format!("{} GPU", job.gpus)}</strong><small>{reason}</small></span>
            </button></li> }
        }).collect_view()}</ul> }.into_any() } }}
    </section> }
}

fn status_class(status: &str) -> &'static str {
    match status {
        "RUNNING" | "COMPLETED" => "job-state success-text",
        "PENDING" => "job-state pending-text",
        "FAILED" | "OUTOFMEMORY" | "TIMEOUT" | "NODEFAIL" => "job-state error-text",
        _ => "job-state",
    }
}

#[component]
fn PendingPanel(
    world: RwSignal<UiWorldView>,
    selected_job: RwSignal<Option<u64>>,
) -> impl IntoView {
    view! { <section class="context-panel" aria-label="Why is it waiting?"><h3>"Why is it waiting?"</h3>
        {move || { let jobs = world.get().jobs; let selected = selected_job.get()
            .and_then(|id| jobs.iter().find(|job| job.id == id && job.status == "PENDING").cloned())
            .or_else(|| jobs.iter().find(|job| job.status == "PENDING").cloned());
            match selected { Some(job) if job.status == "PENDING" => view! {
                <div class="notice pending-notice"><strong>{format!("Job {} / {}", job.id, job.pending_reason.unwrap_or_default())}</strong>
                    <p>{job.pending_explanation.unwrap_or_default()}</p><small>"A pending job can be healthy. Inspect before resubmitting."</small></div>
            }.into_any(), _ => view! { <p class="empty-state">"Select a pending job to see a plain-language scheduler explanation."</p> }.into_any() }
        }}
    </section> }
}

#[component]
fn DiagnosePanel(
    world: RwSignal<UiWorldView>,
    selected_job: RwSignal<Option<u64>>,
) -> impl IntoView {
    view! { <section class="context-panel" aria-label="What should I inspect?"><h3>"What should I inspect?"</h3>
        {move || { let jobs = world.get().jobs; let selected = selected_job.get()
            .and_then(|id| jobs.iter().find(|job| job.id == id && matches!(job.status.as_str(), "OUTOFMEMORY" | "FAILED" | "TIMEOUT" | "NODEFAIL" | "CANCELLED")).cloned())
            .or_else(|| jobs.iter().find(|job| matches!(job.status.as_str(), "OUTOFMEMORY" | "FAILED" | "TIMEOUT" | "NODEFAIL" | "CANCELLED")).cloned());
            match selected { Some(job) => { let tip = match job.status.as_str() {
                "OUTOFMEMORY" => "Compare the memory request with logs, then lower batch size or plan a larger request.",
                "FAILED" => "Inspect logs and virtual input paths before resubmitting.",
                "TIMEOUT" => "Compare elapsed time with the limit and checkpoint interval.",
                "NODEFAIL" => "Treat node health as an external event; look for a usable checkpoint.",
                "PENDING" => "Read the pending explanation and current GPU ownership before changing the request.",
                "RUNNING" => "Verify resources and isolation from inside the allocation.",
                _ => "Use accounting, logs, and resulting state together as evidence.", };
                view! { <div class="job-evidence"><div><strong>{format!("Job {} / {}", job.id, job.status)}</strong><span>{job.name}</span></div>
                    <dl><div><dt>"CPU"</dt><dd>{job.cpus}</dd></div><div><dt>"Memory"</dt><dd>{format!("{} GiB", job.memory_mib / 1_024)}</dd></div>
                        <div><dt>"GPU"</dt><dd>{job.gpus}</dd></div></dl><p>{tip}</p></div> }.into_any()
            }, None => view! { <p class="empty-state">"Select a job to connect its resource request, state, and next diagnostic step."</p> }.into_any() }
        }}
    </section> }
}
