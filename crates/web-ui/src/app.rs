use crate::bridge::{SimBridge, TerminalBuffer};
use crate::persist::{self, PersistedUiState};
use assessment::{Answer, Question};
use dgxlab_contracts::{SimRequest, SimResponse, TerminalKind, UiWorldView};
use leptos::prelude::*;
use sim_session::{cert_bank, SimSession, BUILTIN_LABS};
use std::collections::{BTreeMap, BTreeSet};

const DEFAULT_SEED: u64 = 42;
const SCRIPT_PATH: &str = "train.sbatch";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkspaceMode {
    Learn,
    Sandbox,
    Certification,
}

#[component]
pub fn App() -> impl IntoView {
    let (bridge, terminal, world) = restore_or_new();
    let bridge = RwSignal::new(bridge);
    let terminal = RwSignal::new(terminal);
    let world = RwSignal::new(world);
    let command = RwSignal::new(String::new());
    let mode = RwSignal::new(WorkspaceMode::Learn);
    let script = RwSignal::new(String::new());
    let script_status = RwSignal::new(String::from("Sandbox editor loads train.sbatch from the VFS."));
    let selected_job = RwSignal::new(Option::<u64>::None);
    let light_theme = RwSignal::new(false);
    let cert_result = RwSignal::new(Option::<String>::None);
    let cert_answers = RwSignal::new(BTreeMap::<String, String>::new());
    let cert_multi = RwSignal::new(BTreeMap::<String, BTreeSet<String>>::new());

    Effect::new(move |_| {
        if script.get_untracked().is_empty() {
            load_script_into_editor(bridge, script, script_status);
        }
    });

    let persist_now = move || {
        if let Ok(session_json) = bridge.get_untracked().export_json() {
            persist::save_local(&PersistedUiState {
                session_json,
                terminal_lines: terminal.get_untracked().lines.clone(),
                saved_at_ms: world.get_untracked().now_ms,
            });
        }
    };

    let apply = move |response: SimResponse| {
        apply_response(response, terminal, world, selected_job);
        persist_now();
    };

    let submit = move |_| {
        let trimmed = command.get().trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        command.set(String::new());
        bridge.update(|sim| {
            let response = sim.handle(SimRequest::ExecuteCommand { command: trimmed });
            apply(response);
        });
    };

    let reset = move |_| {
        persist::clear_local();
        let scenario = world.get_untracked().scenario_id.clone();
        bridge.update(|sim| {
            let response = sim.handle(SimRequest::Reset {
                scenario_id: scenario,
                seed: DEFAULT_SEED,
            });
            terminal.update(|buf| buf.lines.clear());
            apply(response);
        });
        load_script_into_editor(bridge, script, script_status);
    };

    let open_lab = move |lab_id: &'static str, scenario: &'static str| {
        move |_| {
            persist::clear_local();
            if let Ok(session) = SimSession::open_lab(lab_id, DEFAULT_SEED) {
                let view = session.view();
                bridge.set(SimBridge::from_session(session));
                world.set(view);
                terminal.update(|buf| buf.lines.clear());
            } else {
                bridge.update(|sim| {
                    let response = sim.handle(SimRequest::Reset {
                        scenario_id: scenario.into(),
                        seed: DEFAULT_SEED,
                    });
                    terminal.update(|buf| buf.lines.clear());
                    apply(response);
                });
            }
            mode.set(WorkspaceMode::Learn);
            load_script_into_editor(bridge, script, script_status);
        }
    };

    let hint = move |_| {
        bridge.update(|sim| apply(sim.handle(SimRequest::UseHint)));
    };
    let set_speed = move |multiplier: u32| {
        move |_| bridge.update(|sim| apply(sim.handle(SimRequest::SetClockSpeed { multiplier })))
    };
    let toggle_pause = move |_| {
        bridge.update(|sim| {
            let response = if sim.view().paused {
                sim.handle(SimRequest::Resume)
            } else {
                sim.handle(SimRequest::Pause)
            };
            apply(response);
        });
    };
    let advance = move |delta_ms: u64| {
        move |_| bridge.update(|sim| apply(sim.handle(SimRequest::AdvanceClock { delta_ms })))
    };

    let save_script = move |_| {
        let content = script.get();
        bridge.update(|sim| {
            apply(sim.handle(SimRequest::WriteVfs {
                path: SCRIPT_PATH.into(),
                content,
            }));
        });
        script_status.set("Saved to virtual /home/learner/train.sbatch".into());
    };
    let submit_script = move |_| {
        let content = script.get();
        bridge.update(|sim| {
            let _ = sim.handle(SimRequest::WriteVfs {
                path: SCRIPT_PATH.into(),
                content,
            });
            apply(sim.handle(SimRequest::ExecuteCommand {
                command: format!("sbatch {SCRIPT_PATH}"),
            }));
        });
        script_status.set("Submitted sbatch train.sbatch".into());
    };

    let grade_cert = move |_| {
        let practical = bridge.with_untracked(|b| b.view().practical_percent);
        let critical_ok = bridge.with_untracked(|b| {
            // Approximate: lab complete implies critical practical OK for standalone cert.
            b.view().lab_complete || b.view().practical_percent >= 80
        });
        let singles = cert_answers.get();
        let multis = cert_multi.get();
        let mut answers = Vec::new();
        for question in cert_bank::certification_questions() {
            let answer = match &question {
                Question::SingleChoice { id, .. } => Answer::SingleChoice {
                    option_id: singles.get(id).cloned().unwrap_or_default(),
                },
                Question::MultiSelect { id, .. } => Answer::MultiSelect {
                    option_ids: multis.get(id).cloned().unwrap_or_default(),
                },
                Question::FillBlank { id, blanks, .. } => Answer::FillBlank {
                    values: blanks
                        .iter()
                        .map(|blank| {
                            let key = format!("{id}:{}", blank.id);
                            (blank.id.clone(), singles.get(&key).cloned().unwrap_or_default())
                        })
                        .collect(),
                },
            };
            answers.push((question.id().to_string(), answer));
        }
        match cert_bank::score_certification(&answers, practical, critical_ok) {
            Ok((scores, result)) => {
                let correct = scores.iter().filter(|s| s.correct).count();
                let trust = "Trust level: standalone local evidence (not identity-proctored).";
                let disclaimer = "DGX Lab is independent educational software and is not affiliated with NVIDIA or SchedMD.";
                cert_result.set(Some(format!(
                    "{} · overall {}% · knowledge {}% · practical {}% · {}/{} knowledge items correct.\n{trust}\n{disclaimer}",
                    if result.passed { "PASSED" } else { "FAILED" },
                    result.overall_percent,
                    result.knowledge_percent,
                    result.practical_percent,
                    correct,
                    scores.len()
                )));
            }
            Err(error) => cert_result.set(Some(format!("Scoring error: {error}"))),
        }
    };

    view! {
        <div class=move || if light_theme.get() { "app-shell theme-light" } else { "app-shell" }>
            <header class="topbar">
                <div>
                    <strong class="brand">"DGX Lab"</strong>
                    <span class="subtitle">"Interactive SLURM Training Simulator · v0.1"</span>
                </div>
                <div class="topbar-status">
                    <span class="pill success">"Offline · Local"</span>
                    <span class="pill accent">"Simulation"</span>
                    <div class="segmented">
                        <button type="button" on:click=set_speed(1)>"1×"</button>
                        <button type="button" on:click=set_speed(10)>"10×"</button>
                        <button type="button" on:click=set_speed(60)>"60×"</button>
                    </div>
                    <button type="button" class="secondary" on:click=toggle_pause>
                        {move || if world.get().paused { "Resume" } else { "Pause" }}
                    </button>
                    <button type="button" class="secondary" on:click=advance(60_000)>"+1m"</button>
                    <button type="button" class="secondary" on:click=advance(600_000)>"+10m"</button>
                    <button type="button" class="secondary" on:click=move |_| light_theme.update(|v| *v = !*v)>
                        {move || if light_theme.get() { "Dark" } else { "Light" }}
                    </button>
                </div>
            </header>

            <nav class="mode-tabs" aria-label="Workspace mode">
                <button type="button" class=move || tab_class(mode.get() == WorkspaceMode::Learn) on:click=move |_| mode.set(WorkspaceMode::Learn)>"Learn"</button>
                <button type="button" class=move || tab_class(mode.get() == WorkspaceMode::Sandbox) on:click=move |_| mode.set(WorkspaceMode::Sandbox)>"Sandbox"</button>
                <button type="button" class=move || tab_class(mode.get() == WorkspaceMode::Certification) on:click=move |_| mode.set(WorkspaceMode::Certification)>"Certification"</button>
            </nav>

            <main class="workspace">
                <aside class="card instructions">
                    <p class="eyebrow">"COURSE · 12 LABS"</p>
                    <div class="lab-list">
                        {BUILTIN_LABS.iter().map(|lab| {
                            let id = lab.id;
                            let scenario = lab.scenario;
                            let title = lab.title;
                            view! {
                                <button type="button" class="lab-chip" on:click=open_lab(id, scenario)>
                                    {format!("{id} · {title}")}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                    <p class="eyebrow" style="margin-top:16px">"ACTIVE"</p>
                    <h1>{move || format!("{} / {}", world.get().scenario_id, format_sim_time(world.get().now_ms))}</h1>
                    <div class="eyebrow">"PROGRESS"</div>
                    <ol class="progress-list">
                        {move || world.get().lab_steps.into_iter().enumerate().map(|(i, step)| {
                            let class = if step.complete { "done" } else { "pending" };
                            view! {
                                <li class=class>
                                    <span class="step-mark">{if step.complete { "✓".into() } else { format!("{}", i+1) }}</span>
                                    <span>{step.label}</span>
                                </li>
                            }
                        }).collect_view()}
                    </ol>
                    <div class="button-row">
                        <button type="button" class="secondary" on:click=hint>"Hint"</button>
                        <button type="button" class="secondary" on:click=reset>"Reset"</button>
                    </div>
                    {move || world.get().hint_text.map(|t| view!{ <div class="notice hint">{t}</div> })}
                    {move || {
                        let w = world.get();
                        view!{ <div class=if w.lab_complete {"notice success-banner"} else {"notice"}>{format!("Practical {}%", w.practical_percent)}</div> }
                    }}
                    <div class="digest">
                        <div class="eyebrow">"STATE DIGEST"</div>
                        <code>{move || truncate_digest(&world.get().state_digest)}</code>
                    </div>
                </aside>

                <section class="center-column">
                    <Show when=move || mode.get() != WorkspaceMode::Certification fallback=move || {
                        view! {
                            <section class="card cert-panel">
                                <div class="card-title">"Certification · standalone local grading"</div>
                                <p class="muted">"Knowledge items are scored offline. Practical % uses the current lab session. This is not identity-proctored."</p>
                                <div class="cert-questions">
                                    {cert_bank::certification_questions().into_iter().map(|q| {
                                        cert_question_view(q, cert_answers, cert_multi)
                                    }).collect_view()}
                                </div>
                                <div class="button-row">
                                    <button type="button" class="primary" on:click=grade_cert>"Score attempt"</button>
                                </div>
                                {move || cert_result.get().map(|text| view!{ <pre class="cert-result">{text}</pre> })}
                            </section>
                        }.into_any()
                    }>
                        <section class="card terminal">
                            <div class="card-title">"Terminal · Simulated"</div>
                            <TerminalView terminal=terminal world=world />
                            <form class="terminal-input-row" on:submit=move |ev| { ev.prevent_default(); submit(()); }>
                                <span class="prompt-prefix">{move || world.get().prompt}</span>
                                <input id="cmd" type="text" autocomplete="off" spellcheck="false"
                                    prop:value=move || command.get()
                                    on:input=move |ev| command.set(event_target_value(&ev)) />
                                <button type="submit" class="primary">"Run"</button>
                            </form>
                        </section>
                        <Show when=move || mode.get() == WorkspaceMode::Sandbox fallback=|| view!{<div></div>}>
                            <section class="card editor">
                                <div class="card-title">"Script editor · VFS only"</div>
                                <textarea class="script-editor" prop:value=move || script.get()
                                    on:input=move |ev| script.set(event_target_value(&ev)) spellcheck="false" />
                                <div class="button-row">
                                    <button type="button" class="secondary" on:click=move |_| load_script_into_editor(bridge, script, script_status)>"Reload"</button>
                                    <button type="button" class="secondary" on:click=save_script>"Save"</button>
                                    <button type="button" class="primary" on:click=submit_script>"sbatch"</button>
                                </div>
                                <p class="muted">{move || script_status.get()}</p>
                            </section>
                        </Show>
                    </Show>
                </section>

                <aside class="card cluster">
                    <div class="card-title">"Cluster · diagnose"</div>
                    <p class="muted">{move || format!("Node {} · ×{}", world.get().node_status, world.get().clock_multiplier)}</p>
                    <ClusterView world=world />
                    <JobList world=world selected_job=selected_job />
                    <DiagnosePanel world=world selected_job=selected_job />
                    <PendingPanel world=world selected_job=selected_job />
                </aside>
            </main>
            <footer>
                "DGX Lab is independent educational software and is not affiliated with, sponsored by, or endorsed by NVIDIA Corporation or SchedMD LLC. · Protocol "
                {dgxlab_contracts::WORKER_PROTOCOL_VERSION}
            </footer>
        </div>
    }
}

fn tab_class(active: bool) -> &'static str {
    if active {
        "mode-tab active"
    } else {
        "mode-tab"
    }
}

fn truncate_digest(digest: &str) -> String {
    if digest.len() > 16 {
        format!("{}…", &digest[..16])
    } else {
        digest.to_string()
    }
}

fn cert_question_view(
    question: Question,
    answers: RwSignal<BTreeMap<String, String>>,
    multi: RwSignal<BTreeMap<String, BTreeSet<String>>>,
) -> AnyView {
    match question {
        Question::SingleChoice {
            id, prompt, options, ..
        } => view! {
            <fieldset class="cert-q">
                <legend>{prompt}</legend>
                {options.into_iter().map(|opt| {
                    let qid = id.clone();
                    let oid = opt.id.clone();
                    view! {
                        <label class="cert-option">
                            <input type="radio" name=id.clone()
                                on:change=move |_| {
                                    answers.update(|map| { map.insert(qid.clone(), oid.clone()); });
                                }
                            />
                            {format!(" ({}) {}", opt.id, opt.text)}
                        </label>
                    }
                }).collect_view()}
            </fieldset>
        }
        .into_any(),
        Question::MultiSelect {
            id, prompt, options, ..
        } => view! {
            <fieldset class="cert-q">
                <legend>{prompt}</legend>
                {options.into_iter().map(|opt| {
                    let qid = id.clone();
                    let oid = opt.id.clone();
                    view! {
                        <label class="cert-option">
                            <input type="checkbox"
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    multi.update(|map| {
                                        let set = map.entry(qid.clone()).or_default();
                                        if checked { set.insert(oid.clone()); } else { set.remove(&oid); }
                                    });
                                }
                            />
                            {format!(" ({}) {}", opt.id, opt.text)}
                        </label>
                    }
                }).collect_view()}
            </fieldset>
        }
        .into_any(),
        Question::FillBlank {
            id, prompt, blanks, ..
        } => view! {
            <fieldset class="cert-q">
                <legend>{prompt}</legend>
                {blanks.into_iter().map(|blank| {
                    let key = format!("{id}:{}", blank.id);
                    view! {
                        <label class="cert-option">
                            {blank.id.clone()}
                            <input type="text"
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    answers.update(|map| { map.insert(key.clone(), value); });
                                }
                            />
                        </label>
                    }
                }).collect_view()}
            </fieldset>
        }
        .into_any(),
    }
}

fn load_script_into_editor(
    bridge: RwSignal<SimBridge>,
    script: RwSignal<String>,
    script_status: RwSignal<String>,
) {
    bridge.update(|sim| match sim.handle(SimRequest::ReadVfs {
        path: SCRIPT_PATH.into(),
    }) {
        SimResponse::FileContent { content, path, .. } => {
            script.set(content);
            script_status.set(format!("Loaded {path}"));
        }
        SimResponse::Error { message, .. } => script_status.set(message),
        _ => {}
    });
}

fn restore_or_new() -> (SimBridge, TerminalBuffer, UiWorldView) {
    if let Some(saved) = persist::load_local() {
        if let Ok(session) = SimSession::import_json(&saved.session_json) {
            let bridge = SimBridge::from_session(session);
            let view = bridge.view();
            return (
                bridge,
                TerminalBuffer {
                    lines: saved.terminal_lines,
                },
                view,
            );
        }
    }
    let bridge = SimBridge::connect("guided-one-gpu", DEFAULT_SEED).expect("scenario");
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
            terminal.update(|buf| buf.lines.extend(lines));
            if selected_job.get_untracked().is_none() {
                if let Some(job) = state.jobs.iter().find(|j| j.user == "learner") {
                    selected_job.set(Some(job.id));
                }
            }
            world.set(state);
        }
        SimResponse::State { state, .. } | SimResponse::Ready { state, .. } => world.set(state),
        SimResponse::FileContent { .. } => {}
        SimResponse::Error { message, .. } => {
            terminal.update(|buf| {
                buf.lines
                    .push(dgxlab_contracts::TerminalLine::stderr(message))
            });
        }
    }
}

fn format_sim_time(ms: u64) -> String {
    let s = ms / 1000;
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

#[component]
fn TerminalView(
    terminal: RwSignal<TerminalBuffer>,
    world: RwSignal<UiWorldView>,
) -> impl IntoView {
    view! {
        <pre class="terminal-body" role="log" aria-live="polite">
            {move || {
                let buf = terminal.get();
                if buf.lines.is_empty() {
                    format!("{}\nSelect a lab, then run simulated commands.\n", world.get().prompt)
                } else {
                    buf.lines.iter().map(|line| match line.kind {
                        TerminalKind::Input => format!("$ {}", line.text),
                        TerminalKind::Stderr => format!("! {}", line.text),
                        _ => line.text.clone(),
                    }).collect::<Vec<_>>().join("\n")
                }
            }}
        </pre>
    }
}

#[component]
fn ClusterView(world: RwSignal<UiWorldView>) -> impl IntoView {
    view! {
        <div class="gpu-grid">
            {move || world.get().gpus.into_iter().map(|gpu| {
                let owner = gpu.owner_job_id.map(|id| format!("job {id}")).unwrap_or_else(|| gpu.status.clone());
                view! {
                    <div class="gpu-tile" class:allocated=gpu.owner_job_id.is_some()>
                        <strong>{format!("GPU {}", gpu.index)}</strong>
                        <span>{format!("{} · {}", gpu.model, owner)}</span>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

#[component]
fn JobList(world: RwSignal<UiWorldView>, selected_job: RwSignal<Option<u64>>) -> impl IntoView {
    view! {
        <div class="job-list">
            <div class="card-title">"Jobs"</div>
            {move || {
                let jobs = world.get().jobs;
                if jobs.is_empty() {
                    view!{ <p class="muted">"Queue empty"</p> }.into_any()
                } else {
                    view! {
                        <ul class="job-ul">
                            {jobs.into_iter().map(|job| {
                                let id = job.id;
                                let selected = selected_job.get() == Some(id);
                                view! {
                                    <li class=if selected {"job-item selected"} else {"job-item"}
                                        on:click=move |_| selected_job.set(Some(id))>
                                        <strong>{format!("{}", job.id)}</strong>
                                        {format!(" {} · {} · g{} · {}", job.user, job.status, job.gpus, job.pending_reason.unwrap_or_else(|| "—".into()))}
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn PendingPanel(world: RwSignal<UiWorldView>, selected_job: RwSignal<Option<u64>>) -> impl IntoView {
    view! {
        <div class="pending-panel">
            <div class="card-title">"Pending reason"</div>
            {move || {
                let jobs = world.get().jobs;
                let selected = selected_job.get()
                    .and_then(|id| jobs.iter().find(|j| j.id == id).cloned())
                    .or_else(|| jobs.iter().find(|j| j.status == "PENDING").cloned());
                match selected {
                    Some(job) if job.status == "PENDING" => view! {
                        <div class="notice hint">
                            <strong>{format!("Job {} · {}", job.id, job.pending_reason.unwrap_or_default())}</strong>
                            <p>{job.pending_explanation.unwrap_or_default()}</p>
                        </div>
                    }.into_any(),
                    _ => view!{ <p class="muted">"No pending job selected."</p> }.into_any(),
                }
            }}
        </div>
    }
}

#[component]
fn DiagnosePanel(world: RwSignal<UiWorldView>, selected_job: RwSignal<Option<u64>>) -> impl IntoView {
    view! {
        <div class="pending-panel">
            <div class="card-title">"Diagnose"</div>
            {move || {
                let jobs = world.get().jobs;
                let selected = selected_job.get()
                    .and_then(|id| jobs.iter().find(|j| j.id == id).cloned())
                    .or_else(|| jobs.iter().find(|j| matches!(j.status.as_str(), "OUTOFMEMORY"|"FAILED"|"TIMEOUT"|"NODEFAIL"|"CANCELLED")).cloned());
                match selected {
                    Some(job) => {
                        let tip = match job.status.as_str() {
                            "OUTOFMEMORY" => "Likely GPU or host memory oversubscription. Lower --batch-size / raise --mem, resume from checkpoint.",
                            "FAILED" => "Script or missing input failure. Inspect stderr logs under logs/.",
                            "TIMEOUT" => "Walltime exhausted. Raise --time or checkpoint more often.",
                            "NODEFAIL" => "Simulated node health event. Drain/recovery labs apply.",
                            other => other,
                        };
                        view! {
                            <div class="notice">
                                <strong>{format!("Job {} · {}", job.id, job.status)}</strong>
                                <p>{format!("{} · {} CPU · {} MiB · {} GPU", job.name, job.cpus, job.memory_mib, job.gpus)}</p>
                                <p class="muted">{tip}</p>
                            </div>
                        }.into_any()
                    }
                    None => view!{ <p class="muted">"Select a terminal/failed job for diagnosis tips."</p> }.into_any(),
                }
            }}
        </div>
    }
}
