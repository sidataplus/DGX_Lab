# DGX Lab

## Product Requirements Document

**Document version:** 1.0  
**Status:** Approved for implementation  
**Date:** 5 August 2026  
**Product:** DGX Lab  
**Subtitle:** Interactive SLURM Training Simulator  
**Primary application form:** Standalone desktop application built with Tauri 2  
**Primary execution model:** Leptos client-side UI and deterministic Rust simulation compiled to WebAssembly  
**Primary user model:** One local learner with simulated concurrent users, workloads, and infrastructure events  
**Connectivity:** Fully offline; no connection to a real scheduler, cluster, shell, SSH service, container runtime, or external API  
**Initial platform target:** macOS Apple Silicon  
**Required v1.0 platforms:** macOS Apple Silicon, Windows x86-64, and Linux x86-64  
**Code license:** Apache License 2.0  
**Built-in course-content license:** Creative Commons Attribution 4.0, subject to third-party notices  
**Canonical deliverable:** Markdown

---

# Document control

## Purpose

This Product Requirements Document defines the product behavior, educational design, simulation semantics, architecture, security boundary, certification system, packaging, quality strategy, delivery roadmap, and acceptance criteria for **DGX Lab**.

The document is intended to be implementation-ready. It shall support:

- architecture and dependency decisions;
- repository and crate design;
- curriculum and assessment authoring;
- scenario-pack schemas;
- simulator behavior and compatibility rules;
- backlog generation and milestone planning;
- threat modeling and capability review;
- automated acceptance testing;
- cross-platform release preparation;
- maintenance by one primary human developer assisted by coding agents.

## Decision status

The following decisions are approved:

1. The canonical product name is **DGX Lab**.
2. The canonical subtitle is **Interactive SLURM Training Simulator**.
3. DGX Lab is a standalone, single-user desktop application.
4. The desktop shell is **Tauri 2**.
5. The interface is implemented with **Leptos client-side rendering** and compiled to WebAssembly.
6. The simulation engine is pure Rust and is compiled both natively for tests and to WebAssembly for production execution.
7. The production simulation runs in a dedicated browser Web Worker inside the Tauri WebView.
8. The Tauri native process is a narrow packaging and file-dialog shell, not an application backend.
9. DGX Lab has no localhost server in its primary desktop form.
10. DGX Lab contains no real SLURM adapter, no SSH client, no process execution, no arbitrary shell, no external HTTP client, and no network dependency.
11. The application supports one real learner and deterministic simulated concurrent users.
12. The initial default profile represents a generic single-node, eight-GPU H200-class DGX environment without institutional names, paths, IP addresses, or operational secrets.
13. The curriculum teaches user-facing SLURM and shared GPU-computing workflows before administrator topics.
14. Built-in lessons include guided labs, deterministic hints, free-play sandbox scenarios, knowledge checks, and formal certification assessments.
15. Certification combines practical simulator tasks, multiple-choice questions, and fill-in-the-blank questions.
16. Certification is locally graded and clearly distinguishes standalone, instructor-verified, and future institutionally verified evidence.
17. Sessions, progress, virtual files, assessment evidence, and scenario state persist locally in IndexedDB.
18. Full sessions export to a portable `.dgxlab` bundle.
19. Course and scenario packs import through a non-executable `.dgxlabpack` format.
20. The application ships with no telemetry, cloud account, cloud sync, external fonts, content-delivery network, or online updater in v1.
21. English is the initial user-interface and course language; internationalization support is P0 and a Thai course pack is P1.
22. The project is architected for public open-source distribution, subject to a trademark and naming review before public release.
23. DGX Lab is independent educational software and shall not imply sponsorship, affiliation, or endorsement by NVIDIA or SchedMD.
24. A secondary static web build remains architecturally possible but is not required for the first desktop MVP.

## Source basis and generalization policy

The default eight-GPU profile is informed by a commissioned single-node DGX H200 environment with eight H200 GPUs, 224 logical CPUs, approximately 1.86 TB of scheduler-visible memory, Slurm 25.05, accounting, cgroup isolation, Singularity execution, and monitoring of GPU and scheduler state [@orca2026UAT; @orca2026Installation; @orca2026Monitoring].

DGX Lab deliberately generalizes those operational characteristics:

- `orca-*` hostnames become `dgx-*` hostnames;
- institutional partitions and accounts become generic teaching profiles;
- `/lustrefs` and `/cm/shared` become `/shared`, `/datasets`, `/containers`, and `/checkpoints`;
- site IP addresses, BMC addresses, network VLANs, credentials, and support paths are excluded;
- exact production errors may inspire scenarios but are rewritten as clearly simulated output;
- no source document is bundled with the public application unless separately licensed.

The application models **pedagogically faithful behavior**, not a complete implementation of Slurm. Supported command semantics are versioned as DGX Lab behavior and are tested against the documented Slurm concepts they teach [@slurmJobStates; @slurmSbatch; @slurmSqueue; @slurmGRES].

---

# 1. Executive summary

## 1.1 Product statement

> **DGX Lab is a standalone desktop simulation environment for learning SLURM and shared GPU-computing workflows through realistic DGX-scale scenarios, virtual users, synthetic workloads, guided exercises, safe failure experimentation, and locally verifiable assessments.**

The application gives a learner a realistic terminal, a virtual filesystem, a scheduler, GPU nodes, job accounting, containers, logs, dashboards, competing users, and failures. Every object is simulated. A learner may submit jobs, exhaust memory, wait on resources, diagnose pending reasons, lose a node, resume a checkpoint, and inspect accounting evidence without consuming a single real GPU or acquiring credentials to a production cluster.

DGX Lab is not a collection of canned command-output screenshots. It is a deterministic discrete-event world:

```text
learner command
      │
      ▼
typed parser and validator
      │
      ▼
virtual shell / SLURM command model
      │
      ▼
discrete-event scheduler simulation
      │
      ├── resource allocation
      ├── job and step state
      ├── simulated users
      ├── synthetic workload telemetry
      ├── failures and recovery
      └── accounting history
      │
      ▼
terminal output + visual cluster state + grading evidence
```

The same simulated state drives the terminal, job table, GPU map, logs, charts, grading, and session replay. Humanity has already produced enough training software where each panel invents its own version of reality.

## 1.2 Core value

DGX Lab provides four forms of value:

1. **Safety:** learners cannot harm, congest, or misconfigure production infrastructure.
2. **Repetition:** expensive and rare failures can be practiced repeatedly and reset instantly.
3. **Visibility:** learners can see hidden scheduler state, resource allocation, pending causes, and time evolution.
4. **Evidence:** competency is demonstrated through commands, resulting state, practical outcomes, and knowledge assessments rather than passive attendance.

## 1.3 Product modes

```text
DGX Lab
├── Learn
│   ├── concept cards
│   ├── demonstrations
│   └── guided labs
├── Practice
│   ├── free-play sandbox
│   ├── targeted failure scenarios
│   └── scenario controls
├── Assess
│   ├── knowledge checks
│   ├── certification exam
│   └── practical competency challenges
├── Review
│   ├── command transcript
│   ├── job timeline
│   ├── mistakes and hints
│   └── competency report
└── Build later
    ├── imported course packs
    └── external scenario compiler
```

## 1.4 Technical summary

```text
                         DGX Lab desktop application

┌────────────────────────────────────────────────────────────────────┐
│                         Tauri 2 native shell                       │
│                                                                    │
│  window lifecycle · packaged assets · native open/save dialogs     │
│  narrowly scoped import/export of .dgxlab and .dgxlabpack files    │
│                                                                    │
│  NO shell · NO process spawn · NO SSH · NO HTTP client             │
│  NO real filesystem browsing outside user-selected files           │
└───────────────────────────────┬────────────────────────────────────┘
                                │ minimal capability bridge
                                ▼
┌────────────────────────────────────────────────────────────────────┐
│                         System WebView                              │
│                                                                    │
│  Leptos UI WASM                                                    │
│  ├── guided learning                                               │
│  ├── custom simulated terminal                                    │
│  ├── virtual file editor                                          │
│  ├── cluster and GPU visualization                                │
│  ├── assessment interface                                         │
│  └── reports and session controls                                 │
│                                                                    │
│                typed messages                                     │
│                      │                                             │
│                      ▼                                             │
│  Simulation Web Worker WASM                                       │
│  ├── virtual shell and filesystem                                 │
│  ├── SLURM command model                                          │
│  ├── resource scheduler                                           │
│  ├── discrete-event clock                                         │
│  ├── concurrent virtual actors                                    │
│  ├── synthetic workloads and telemetry                            │
│  ├── scenario/fault engine                                        │
│  ├── grading and certification evidence                           │
│  └── deterministic event log and replay                           │
│                                                                    │
│  IndexedDB                                                         │
│  ├── progress                                                     │
│  ├── events and snapshots                                         │
│  ├── virtual files                                                │
│  ├── imported packs                                               │
│  └── assessment evidence                                          │
└────────────────────────────────────────────────────────────────────┘
```

Tauri 2's capability model permits frontend access to be limited by window, permission, and scope; this is used to keep the native bridge deliberately narrow [@tauri2026Capabilities]. Leptos client-side rendering compiles the application to WebAssembly and runs it inside the system WebView [@leptos2026CSR; @tauri2026Leptos].

---

# 2. Problem statement

## 2.1 Learner problem

New users of shared GPU systems routinely struggle with concepts that ordinary local development hides:

- a login node is not a compute allocation;
- resource requests affect scheduling and isolation;
- CPU, memory, GPU, task, and node requests are related but not interchangeable;
- a submitted job may remain pending for valid reasons;
- a job allocation and a job step are different objects;
- GPU visibility may be remapped inside a job or container;
- batch jobs require explicit environment and filesystem assumptions;
- host-memory OOM and GPU-memory OOM are different failure classes;
- wall-time limits, checkpointing, cancellation, and resumption are operational requirements;
- a fast model training script is not yet a reliable shared-cluster workload;
- accounting and telemetry matter after the job exits.

Documentation alone explains syntax but gives limited intuition about time, contention, failure, and scheduler state. Real clusters offer authentic behavior but are scarce, expensive, security-sensitive, and poorly suited to destructive exercises.

## 2.2 Institutional problem

Training users directly on a production accelerator cluster creates predictable costs:

1. instructors must reserve real resources;
2. mistakes may disrupt other work;
3. every learner needs an account and network access;
4. failure exercises are constrained by operational safety;
5. classes are vulnerable to maintenance, queued jobs, and infrastructure incidents;
6. monitoring and accounting views may expose other users or internal topology;
7. learners receive uneven experiences because production state changes;
8. repeating a lab consumes additional GPU-hours.

## 2.3 Existing-tool gap

Typical alternatives each omit an important part of the learning problem:

| Approach | Strength | Limitation |
|---|---|---|
| Written tutorial | inexpensive | no scheduler behavior or consequences |
| Shell transcript | looks authentic | output is canned and panels can disagree |
| Containerized mock CLI | easy distribution | often lacks coherent scheduling and time |
| Tiny real Slurm cluster | authentic scheduler | still requires infrastructure and safe isolation |
| Production training allocation | fully authentic | costly, risky, inconsistent, administratively burdensome |
| Generic HPC course | broad concepts | rarely models an eight-GPU AI training workflow in depth |

DGX Lab fills the gap with a coherent simulated system that is local, deterministic, visually inspectable, and safe.

## 2.4 Product opportunity

The product can serve:

- institutional onboarding before granting real cluster access;
- self-study for researchers and students;
- pre-course preparation for hands-on workshops;
- competency assessment for shared GPU access;
- failure-diagnosis practice for experienced users;
- reproducible demonstrations in classrooms or conference workshops;
- public open-source training without publishing institutional topology.

---

# 3. Product vision and principles

## 3.1 Vision

DGX Lab becomes the preferred safe first environment for learning how to transform an interactive ML experiment into a reproducible, observable, checkpointed, resource-efficient SLURM workload.

## 3.2 Product principles

### P1. Simulate state, not screenshots

Every command changes or queries a shared simulation world. Terminal output, visual panels, grading, and reports derive from the same state.

### P2. Teach transferable mental models

The curriculum emphasizes jobs, allocations, steps, resources, states, pending reasons, files, containers, accounting, and recovery. Exact site aliases and house rules remain profile data rather than universal truths.

### P3. Failure is a first-class lesson

OOM, timeouts, invalid requests, pending jobs, drained nodes, quota errors, broken checkpoints, and storage incidents are intentional learning material.

### P4. Nothing reaches real infrastructure

The absence of real scheduler access is structural, tested, and non-configurable. It is not a checkbox named `simulation=true` sitting beside a real backend.

### P5. Determinism before theatrical realism

The same scenario revision, seed, and learner actions must reproduce the same scheduler decisions and grading result. Reproducibility is more valuable than decorative randomness.

### P6. Consequences remain visible

Learners should see why a job is pending, what resources it holds, what it consumed, why it failed, and what changed after remediation.

### P7. Grade competency, not memorized whitespace

Practical assessment evaluates resulting state and evidence. Equivalent commands and valid alternative workflows receive credit.

### P8. Offline is the default, not a degraded mode

Every runtime asset ships locally. The app remains complete with the network disabled.

### P9. Desktop packaging must not become a native backend

Tauri provides a window and constrained import/export. Simulation, grading, storage, and application logic remain in the WebView/WASM boundary.

### P10. One learner can experience a crowded cluster

Concurrent users are deterministic actors inside the simulation. The application does not need real accounts, threads, or networked classmates to teach contention.

### P11. Explanations remain inspectable

Hints, grading rules, accepted answers, scenario transitions, and workload models are deterministic data or code. No online LLM is required to decide whether a learner was somehow correct in spirit.

### P12. Public branding must remain honest

DGX Lab uses original visual identity, no NVIDIA logo, and a clear independent-simulator disclaimer. Public naming remains subject to trademark review.

---

# 4. Goals and non-goals

## 4.1 Product goals

| ID | Goal |
|---|---|
| G-01 | Provide a one-click standalone desktop environment for learning SLURM and shared GPU computing. |
| G-02 | Simulate coherent scheduling, resource allocation, job states, steps, accounting, and failure behavior. |
| G-03 | Model realistic single-node eight-GPU AI workloads and contention. |
| G-04 | Allow repeated practice without real cluster credentials or resource consumption. |
| G-05 | Teach progression from interactive debugging to reliable batch execution. |
| G-06 | Support guided learning, free-play, failure drills, and formal assessment. |
| G-07 | Provide deterministic virtual users and background workload scenarios. |
| G-08 | Provide visual explanations of queue, GPU, CPU, RAM, time, and job history. |
| G-09 | Persist progress and permit complete session export/import. |
| G-10 | Generate locally graded certification evidence from practical and knowledge assessments. |
| G-11 | Run fully offline with no external runtime dependency. |
| G-12 | Be structurally incapable of invoking real SLURM, shell, SSH, containers, or cluster APIs. |
| G-13 | Remain maintainable by one primary developer through pure domain logic, typed schemas, tests, and compiled course packs. |
| G-14 | Support later static-web distribution from the same WASM application. |
| G-15 | Support localization and a future Thai course pack without forking the simulator. |

## 4.2 Non-goals

| ID | Non-goal |
|---|---|
| NG-01 | DGX Lab is not a real SLURM client, scheduler proxy, or cluster-management tool. |
| NG-02 | It does not execute learner shell commands, Python, containers, CUDA, NCCL, or native binaries. |
| NG-03 | It does not implement every Slurm command, option, plugin, or edge case. |
| NG-04 | It does not provide a general Unix emulator. |
| NG-05 | It does not teach production cluster administration in MVP. |
| NG-06 | It does not provide a networked multiplayer classroom in v1. |
| NG-07 | It does not verify legal identity or provide tamper-proof remote proctoring. |
| NG-08 | It does not require or offer cloud accounts, telemetry, hosted progress, or online content. |
| NG-09 | It does not bundle NVIDIA logos, copied brand styling, proprietary screenshots, or production configuration. |
| NG-10 | It does not claim exact output compatibility with every Slurm version or site configuration. |
| NG-11 | It does not include an AI tutor in MVP. |
| NG-12 | It does not permit executable third-party plugins or course-pack code. |
| NG-13 | It does not expose arbitrary native filesystem or process access through Tauri. |
| NG-14 | It does not create an abstraction for a future `RealSlurmBackend`. |
| NG-15 | It does not model MIG, GPU sharding, or MPS in the initial curriculum. |

---

# 5. Users and jobs to be done

## 5.1 Primary learner

The primary learner is a researcher, data scientist, ML engineer, graduate student, or technical staff member who:

- understands basic files, directories, and command-line concepts;
- may have run Python locally;
- has limited or no experience with SLURM;
- needs to use shared GPU infrastructure safely and efficiently.

### Jobs to be done

1. Understand the roles of login node, compute node, partition, job, allocation, step, and scheduler.
2. Request CPU, RAM, time, and GPUs correctly.
3. obtain an interactive allocation and inspect its environment.
4. convert a working command into a batch script.
5. understand why a job is pending.
6. diagnose host OOM, GPU OOM, timeout, cancellation, script failure, and missing inputs.
7. use arrays and dependencies for experiment campaigns.
8. checkpoint and resume long work.
9. run a simulated two- or four-GPU training workload.
10. interpret `sacct`, `sstat`, resource utilization, and efficiency evidence.
11. demonstrate competency before receiving access to a real cluster.

## 5.2 Experienced learner

An experienced user uses failure scenarios and sandbox profiles to rehearse:

- QOS and reservation behavior;
- fair-share and priority;
- node drain and maintenance;
- storage and quota failures;
- multi-GPU scaling and communication symptoms;
- failure recovery and accounting interpretation.

## 5.3 Instructor or course author

There is no separate login role. A local **Scenario Control** mode lets the same user:

- inspect hidden scenario state;
- control the simulated clock;
- inject faults;
- reset or branch a scenario;
- view grading rules;
- validate imported packs;
- run demonstrations.

Entering Scenario Control during a certification attempt invalidates the attempt.

## 5.4 Institutional reviewer

A training coordinator or cluster administrator may review exported evidence:

- certificate summary;
- practical competency matrix;
- question responses;
- command transcript;
- scenario and application versions;
- assessment seed;
- integrity digest.

The reviewer does not need DGX Lab to issue a local certificate, but DGX Lab should make replay and verification convenient.

## 5.5 Coding agent

An approved coding agent assists with implementation, tests, documentation, and course generation. It has no runtime role and no in-product privileged interface.

---

# 6. Product modes and top-level navigation

## 6.1 Navigation

```text
Home
Learn
Practice
Assess
Review
Library
Settings
About
```

## 6.2 Home

The home screen prioritizes:

1. resume current lesson;
2. next recommended competency;
3. recent practical result;
4. certification readiness;
5. imported pack warnings;
6. recovery of an interrupted session.

It shall not resemble an executive analytics dashboard where eight decorative charts compete to report that nothing has happened.

## 6.3 Learn mode

Learn mode contains:

- concise concept cards;
- annotated command examples;
- interactive demonstrations;
- guided labs;
- deterministic progressive hints;
- ungraded knowledge checks;
- end-of-lab reflection and evidence.

## 6.4 Practice mode

Practice mode contains:

- free-play sandbox;
- targeted drills;
- cluster profiles;
- fault scenarios;
- clock controls;
- optional Scenario Control;
- session rewind and branching in P1.

## 6.5 Assess mode

Assess mode contains:

- module quizzes;
- certification readiness checks;
- certification attempt creation;
- multiple-choice and multi-select questions;
- fill-in-the-blank questions;
- command-output interpretation;
- practical challenges;
- results and certificate generation.

## 6.6 Review mode

Review mode contains:

- chronological command transcript;
- virtual job history;
- timeline and resource charts;
- hints used;
- failed attempts;
- corrected misconceptions;
- competency matrix;
- export controls.

## 6.7 Library

Library contains:

- built-in courses;
- built-in cluster profiles;
- imported `.dgxlabpack` files;
- signature/trust state;
- version and compatibility status;
- pack removal controls.

## 6.8 Settings

Settings contains:

- language;
- theme;
- text size;
- reduced motion;
- keyboard preferences;
- default clock speed;
- storage usage;
- data export and reset;
- developer diagnostics;
- no network or telemetry settings, because neither feature exists.

---

# 7. End-to-end learning workflow

```text
Choose course
    ↓
Read concise concept card
    ↓
Observe or manipulate a demonstration
    ↓
Enter guided scenario
    ↓
Use terminal, editor, and visual cluster view
    ↓
Simulator records state-based evidence
    ↓
Receive deterministic feedback and optional hint
    ↓
Complete knowledge check
    ↓
Review competency evidence
    ↓
Proceed to next lab
    ↓
Take certification readiness check
    ↓
Start certification attempt
    ├── knowledge assessment
    └── practical scenarios
    ↓
Generate local certificate and evidence bundle
```

## 7.1 Example guided task

> Request an interactive allocation with one H200 GPU, eight CPUs, 64 GiB of RAM, and a 30-minute wall-time. Confirm that only the allocated GPU is visible.

The learner may use:

```bash
srun --partition=gpu \
  --gres=gpu:h200:1 \
  --cpus-per-task=8 \
  --mem=64G \
  --time=00:30:00 \
  --pty bash
```

or an equivalent supported form.

The simulator then creates an allocation, assigns a physical virtual GPU, remaps it inside the step, sets simulated environment variables, and updates all views. Slurm's GPU GRES behavior uses `CUDA_VISIBLE_DEVICES` to identify devices available to job steps, and cgroup-constrained environments may remap the allocated device to index zero inside the job [@slurmGRES; @slurmPrologEpilog].

Hidden evidence checks include:

- one GPU requested;
- CPU and RAM within policy;
- job reached `RUNNING`;
- learner inspected `CUDA_VISIBLE_DEVICES` or `nvidia-smi -L`;
- visible GPU count equals one;
- learner did not mistake the physical simulator GPU ID for the job-local ID.

---

# 8. Learning and certification design

## 8.1 Competency model

DGX Lab organizes learning around observable competencies rather than course completion alone.

```text
C1  Cluster mental model
C2  Interactive allocations
C3  CPU and RAM requests
C4  GPU requests and visibility
C5  Environment modules and containers
C6  Batch scripts
C7  Queue and pending diagnosis
C8  Arrays and dependencies
C9  Failure diagnosis
C10 Checkpoint and resume
C11 Multi-GPU execution
C12 Accounting and efficiency
```

Each competency has:

- learning objectives;
- prerequisite competencies;
- guided evidence;
- knowledge items;
- practical assessment rules;
- critical and non-critical criteria;
- remediation recommendations.

## 8.2 Built-in v1 course sequence

| Lab | Title | Primary competency |
|---|---|---|
| 01 | Meet the Cluster | login node, compute node, partition, job, step |
| 02 | Your First Interactive Job | `srun`, allocation lifecycle, environment |
| 03 | Ask for What You Need | CPU, memory, time, validation |
| 04 | One GPU Means One GPU | GRES, visibility, device isolation |
| 05 | Reproducible Environments | modules and simulated Singularity |
| 06 | From Command to Batch Script | `#SBATCH`, logs, submission |
| 07 | Why Is My Job Pending? | resources, priority, limits, dependency |
| 08 | Run a Campaign | arrays, task IDs, output naming |
| 09 | Failure Is Data | GPU OOM, host OOM, timeout, script error |
| 10 | Survive Interruption | checkpoints, cancellation, resume |
| 11 | Scale Across GPUs | `torchrun`, ranks, multi-GPU allocation |
| 12 | Prove What Happened | `sacct`, `sstat`, efficiency and evidence |

## 8.3 Practice versus certification

| Behavior | Practice | Certification |
|---|---|---|
| Hints | available | disabled by default; use marks attempt assisted |
| Scenario reset | unrestricted | controlled; restart counts as attempt restart |
| Scenario Control | available | prohibited; opening invalidates attempt |
| Explanation after question | immediate or deferred | after section submission |
| Command reference | available | limited to approved reference sheet |
| Clock control | learner-controlled | scenario-defined |
| Seed | visible | recorded, hidden until completion |
| Transcript editing | impossible | impossible |
| Pass/fail | formative | summative |

## 8.4 Certification composition

| Component | Weight |
|---|---:|
| Practical SLURM scenarios | 60% |
| Multiple-choice and multi-select questions | 25% |
| Fill-in-the-blank questions | 15% |

Initial passing policy:

- at least **80% overall**;
- at least **70% across knowledge questions**;
- all critical practical competencies completed;
- no critical safety misconception left unresolved;
- no use of Scenario Control;
- no more than two attempts per generated certification session unless the course policy overrides it.

## 8.5 Question types

### Single-answer multiple choice

One correct option from two or more choices.

### Multi-select

One or more correct options. Partial-credit policy is explicit per item and shall avoid rewarding selection of every option.

### Fill in the blank

Supports:

- exact normalized text;
- case-insensitive answers;
- normalized whitespace;
- accepted aliases;
- equivalent supported flags;
- multiple blanks;
- numeric tolerance;
- optional partial credit.

Example:

```text
Complete the command to request one H200 GPU:

srun --gres=__________:1 --pty bash
```

Accepted answers may include `gpu:h200` and, when the scenario profile permits a generic GPU request, `gpu`.

### Command-output interpretation

The learner inspects simulated `squeue`, `scontrol`, `sacct`, log, or telemetry output and selects or enters the diagnosis.

### Practical scenario

The learner reaches a target state through terminal commands and virtual files. Grading evaluates state and evidence, not exact keystrokes.

## 8.6 Assessment randomization

A certification attempt is determined by:

```text
application version
+ course revision
+ assessment blueprint revision
+ generated seed
+ question-bank revision
+ practical-scenario revisions
```

Randomization may change:

- option order;
- question selection within blueprint constraints;
- numeric resources;
- job names and IDs;
- virtual user names;
- background load;
- practical scenario parameters.

Randomization may not change the intended competency or difficulty band without creating a different blueprint revision.

## 8.7 Certificate and evidence levels

| Level | Meaning |
|---|---|
| Standalone certificate | locally generated result with replayable evidence and integrity digest |
| Instructor-verified certificate | exported bundle reviewed and countersigned outside or through a future local verification workflow |
| Institutionally verified certificate | deferred trusted signing, identity, and/or proctoring workflow |

The standalone application cannot prove the learner's legal identity or prevent a user who controls the device from modifying software or files. It shall state this plainly rather than placing a cryptographic-looking border around wishful thinking.

## 8.8 Certificate contents

- learner-entered display name;
- certificate title;
- application version;
- course and assessment revisions;
- completion timestamp;
- score and pass/fail;
- competency summary;
- assisted/unassisted designation;
- certificate evidence ID;
- SHA-256 evidence digest;
- independent-simulator disclaimer;
- optional instructor countersignature field.

---

# 9. System context and boundaries

## 9.1 In-scope runtime components

```text
Tauri shell
Leptos UI WASM
simulation-worker WASM
IndexedDB
bundled static assets
imported validated data packs
user-selected import/export files
```

## 9.2 Out-of-scope runtime components

```text
real slurmctld or slurmd
Slurm command binaries
SSH or SCP
host shell or PTY
Python or R interpreter
Docker, Podman, Singularity, or Apptainer runtime
CUDA, NCCL, or GPU driver
external database
web service or localhost API
remote telemetry or analytics
cloud authentication
```

## 9.3 Trust boundaries

```text
┌──────────────────────── user-controlled desktop ────────────────────────┐
│                                                                         │
│  imported pack bytes                                                    │
│         │ untrusted                                                     │
│         ▼                                                               │
│  schema, size, path, hash, and signature validation                     │
│         │ validated data only                                           │
│         ▼                                                               │
│  WebView/WASM application                                               │
│         │ narrow Tauri capability calls                                 │
│         ▼                                                               │
│  Tauri native shell                                                     │
│         │ user-selected file only                                       │
│         ▼                                                               │
│  operating-system file dialog and chosen path                           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

The Tauri capability layer reduces the frontend's native authority, but it does not excuse unsafe Rust commands or overly broad scopes. Tauri explicitly notes that capabilities do not protect against malicious native code or incorrect scope checking [@tauri2026Capabilities]. DGX Lab therefore minimizes both permissions and native command surface.

---

# 10. Logical architecture

## 10.1 Component diagram

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                             dgx-lab-ui                                  │
│                     Leptos client-side WASM                             │
│                                                                         │
│ navigation  terminal  editor  job tables  cluster views  assessment     │
│ reports     accessibility  localization  persistence coordinator        │
└──────────────────────────────┬──────────────────────────────────────────┘
                               │ typed request/response messages
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          dgx-lab-sim-worker                             │
│                             WASM Worker                                 │
│                                                                         │
│  sim-core             deterministic clock and event queue               │
│  slurm-model          jobs, steps, nodes, partitions, QOS, accounting   │
│  scheduler            validation, priority, allocation, transitions     │
│  virtual-shell        command parser and environment                    │
│  virtual-fs           files, permissions, quotas, redirection           │
│  workloads            synthetic compute and telemetry models            │
│  actors               scripted and policy-driven virtual users          │
│  scenarios            initial state, faults, objectives                 │
│  grading              state/evidence and question scoring               │
│  replay               event log, snapshots, deterministic restoration   │
└──────────────────────────────┬──────────────────────────────────────────┘
                               │ snapshot/event persistence messages
                               ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                              IndexedDB                                  │
│ sessions · events · snapshots · progress · packs · settings             │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                           Tauri 2 shell                                 │
│ window · bundled resources · open/save dialogs · import/export bytes    │
└─────────────────────────────────────────────────────────────────────────┘
```

## 10.2 Dependency direction

```text
Tauri shell ───────────► no simulation dependencies required
web-ui ────────────────► shared contracts
sim-worker-wasm ───────► sim-core and domain crates
scenario-compiler ─────► shared contracts and validation
native tests ──────────► sim-core and domain crates

sim-core ───────X──────► Tauri
sim-core ───────X──────► browser APIs
sim-core ───────X──────► OS shell/process APIs
sim-core ───────X──────► Slurm libraries or binaries
course packs ───X──────► executable code
```

## 10.3 Why the simulation remains in WASM

Keeping production simulation inside the WebView/WASM boundary provides:

- no path from simulated commands to host command execution;
- static-web parity;
- deterministic worker isolation;
- a small native attack surface;
- shared native and WASM tests from one Rust core;
- easier review of Tauri permissions;
- no temptation to add a real scheduler connector beside the simulator.

---

# 11. Tauri desktop shell

## 11.1 Responsibilities

The Tauri process shall:

- create and manage the main application window;
- load bundled frontend assets;
- expose application version and platform metadata required for reports;
- open native file-selection dialogs;
- read bytes only from a path explicitly selected for import;
- write bytes only to a path explicitly selected for export;
- enforce extension, file-size, and path-type checks;
- package platform-specific application bundles.

## 11.2 Prohibited responsibilities

The Tauri process shall not:

- run simulation logic;
- store authoritative learning state;
- execute shell commands;
- spawn child processes;
- access SSH;
- access network APIs;
- scan directories;
- watch clipboard contents;
- expose environment variables except narrowly required platform metadata;
- load dynamic native plugins;
- update itself automatically;
- open arbitrary URLs from course content.

## 11.3 Capability policy

Tauri capabilities are defined explicitly for the main window. The product shall not rely on broad default plugin permissions. Capability files are versioned and tested. Tauri capabilities can be scoped by window and platform and are intended to constrain native exposure to the frontend [@tauri2026Capabilities].

Permitted capability categories:

```text
core window lifecycle
application metadata required for reports
native open dialog for .dgxlab / .dgxlabpack
native save dialog for .dgxlab / report exports
custom narrowly scoped import/export commands if required
```

Explicitly forbidden capability categories:

```text
shell
process
HTTP client
websocket
localhost server
unrestricted filesystem
command-line sidecars
SQL plugin
updater
clipboard monitoring
global shortcuts
remote content access
```

Tauri's dialog plugin supports native open and save dialogs across desktop platforms [@tauri2026Dialog].

## 11.4 Content Security Policy

The release configuration shall use a restrictive CSP:

- default source limited to bundled application assets;
- no remote scripts, styles, fonts, frames, media, or network connections;
- images limited to bundled, blob, and validated imported data;
- WebAssembly enabled only as required;
- no arbitrary inline script;
- no remote iframe;
- no navigation to untrusted origins.

Tauri recommends a restrictive CSP and warns against remote assets; WebAssembly frontends require the appropriate WASM script allowance [@tauri2026CSP].

## 11.5 Platform packaging

v1.0 target outputs:

```text
DGX-Lab_<version>_aarch64.dmg
DGX-Lab_<version>_x64-setup.exe
DGX-Lab_<version>_amd64.AppImage
```

Alternative Linux packaging may be added after release evidence. Each target is built and smoke-tested on its target operating system.

---

# 12. WebView and Leptos interface

## 12.1 Rendering model

DGX Lab uses Leptos client-side rendering. The frontend is compiled to WebAssembly and loaded as bundled static assets. Leptos documents CSR as a browser-executed WASM application, and Tauri provides a documented Leptos frontend path [@leptos2026CSR; @tauri2026Leptos].

## 12.2 Main UI regions

```text
┌─────────────────────────────────────────────────────────────────────────┐
│ DGX Lab   Course / Scenario   Simulated time   Speed   Session state     │
├───────────────────┬─────────────────────────────────┬───────────────────┤
│                   │                                 │                   │
│ Learning panel    │ Terminal / file editor          │ Cluster view      │
│                   │                                 │                   │
│ objective         │ $ squeue                        │ nodes             │
│ concepts          │ ...                             │ GPUs              │
│ hints             │                                 │ CPU / RAM         │
│ progress          │                                 │ queue             │
│ evidence          │                                 │                   │
├───────────────────┴─────────────────────────────────┼───────────────────┤
│ Timeline / logs / telemetry / assessment            │ Job details       │
└─────────────────────────────────────────────────────┴───────────────────┘
```

## 12.3 Custom terminal

The terminal is a constrained simulator surface, not an operating-system terminal emulator.

Required behavior:

- keyboard-first input;
- command history;
- supported completion;
- editable current command;
- multiline batch-script display;
- selected ANSI colors and text styles;
- copy of terminal text through ordinary selection where the WebView permits it;
- clickable job IDs, node names, and virtual paths;
- clear indication that the terminal is simulated;
- screen-reader transcript mode;
- no PTY, shell process, or `xterm.js` in MVP.

## 12.4 Virtual file editor

The built-in editor supports:

- plain text and shell-script syntax highlighting;
- line numbers;
- search;
- save to virtual filesystem;
- diagnostics for supported `#SBATCH` directives;
- accessible keyboard operation;
- no extensions, code execution, LSP, or host file access.

## 12.5 Visual cluster view

The cluster view displays:

- node states;
- partition membership;
- allocated and idle CPUs;
- allocated and idle RAM;
- per-GPU allocation, utilization, HBM, temperature-like simulated metrics, and fault state;
- job-to-resource relationships;
- pending queue;
- reservations and QOS in advanced profiles;
- simulated time.

Animations are restrained, informative, and disabled under reduced-motion settings.

## 12.6 Job detail view

A job detail includes:

- job and array identifiers;
- user, account, partition, and QOS;
- submit, eligible, start, and end times;
- state and reason;
- request versus allocation;
- node and GPU assignment;
- steps;
- environment;
- dependencies;
- exit code;
- synthetic utilization;
- logs and virtual artifacts;
- accounting summary;
- grading evidence where relevant.

## 12.7 Stale and transitional state

The UI receives state deltas from the simulation worker. It shall:

- preserve ordering by event sequence;
- reject out-of-order deltas;
- request a full state snapshot after a detected gap;
- show an explicit recovery indicator;
- never independently infer authoritative job state from animations.


---

# 13. Simulation engine

## 13.1 Simulation model

DGX Lab uses a deterministic discrete-event simulation. The engine advances from one scheduled event to the next rather than polling simulated processes continuously.

```text
SimulationWorld
├── identity and schema revision
├── deterministic random generator state
├── simulated clock
├── cluster state
├── scheduler configuration
├── users, accounts, associations, and QOS
├── jobs and job steps
├── virtual filesystem
├── workload instances
├── actors
├── fault state
├── ordered event queue
├── learning objectives
└── evidence ledger
```

## 13.2 Determinism contract

Given identical:

- application simulator compatibility version;
- cluster profile revision;
- course/scenario revision;
- seed;
- imported-pack digest;
- learner command/event sequence;

DGX Lab shall reproduce identical:

- job IDs and array IDs;
- actor submissions;
- resource allocations;
- scheduling decisions;
- workload metrics;
- fault occurrence;
- terminal output under the same output profile;
- grading evidence;
- scores.

Timestamps based on simulated time are deterministic. Human wall-clock timestamps are recorded separately and do not affect simulation.

## 13.3 Event ordering

Events are ordered by:

1. simulated timestamp;
2. event priority class;
3. deterministic sequence number.

Initial priority classes:

```text
0  infrastructure state changes required before scheduling
1  job/step termination and resource release
2  learner commands committed at that simulated time
3  actor actions
4  scheduler evaluation and allocation
5  workload progress and metric emission
6  grading/evidence evaluation
7  UI-only notification events
```

The exact priority table is versioned because changing it can alter scenario outcomes.

## 13.4 Clock controls

Supported controls:

- pause;
- advance one event;
- advance to next learner-relevant event;
- real-time progression;
- ×10;
- ×60;
- instant to a selected event or condition in Practice mode.

Certification scenarios control clock behavior and may disable manual speed changes.

## 13.5 Randomness

All scenario randomness uses a deterministic, explicitly selected pseudo-random generator. The generator algorithm and seed derivation are versioned in the replay manifest.

No simulation decision may use:

- operating-system random APIs after session initialization;
- current system time;
- hash-map iteration order;
- platform-dependent floating-point behavior without normalization;
- unordered asynchronous message arrival.

## 13.6 Numeric representation

Resource quantities use integer base units where practical:

- CPU: integer logical CPUs;
- memory and storage: bytes or MiB;
- GPU count: integer devices;
- time: integer milliseconds or microseconds of simulated time;
- utilization: fixed-point basis points;
- scores: integer points or rational components before final display.

This reduces cross-platform drift caused by decorative floating-point precision.

## 13.7 Worker responsiveness

The worker shall process bounded event batches and return to the WebView message loop regularly. At high simulation speeds it emits coalesced state deltas rather than one UI message per metric event. The authoritative event log retains the full logical sequence even when rendering is downsampled.

---

# 14. Core domain model

## 14.1 Identifier model

Identifiers are typed and never interchangeable by raw integer alone.

```rust
pub struct JobId(pub u64);
pub struct StepId(pub u32);
pub struct ArrayTaskId(pub u32);
pub struct NodeId(pub String);
pub struct UserId(pub String);
pub struct ScenarioId(pub String);
pub struct SessionId(pub String);
pub struct EvidenceId(pub String);
```

Job IDs are allocated deterministically from the profile's starting value.

## 14.2 Cluster

```rust
pub struct ClusterState {
    pub cluster_id: String,
    pub display_name: String,
    pub scheduler_profile: SchedulerProfile,
    pub nodes: BTreeMap<NodeId, NodeState>,
    pub partitions: BTreeMap<String, PartitionState>,
    pub users: BTreeMap<UserId, UserState>,
    pub accounts: BTreeMap<String, AccountState>,
    pub qos: BTreeMap<String, QosState>,
    pub reservations: BTreeMap<String, ReservationState>,
    pub jobs: BTreeMap<JobId, JobState>,
    pub clock: SimTime,
}
```

Ordered maps are preferred in deterministic core code.

## 14.3 Node

A node contains:

- identity and display label;
- state: `IDLE`, `MIXED`, `ALLOCATED`, `DRAIN`, `DRAINING`, `DOWN`, `FAIL`, `MAINTENANCE`;
- available and configured CPU;
- available and configured memory;
- GPU devices and topology metadata;
- features and constraints;
- partition membership;
- active jobs and steps;
- simulated health state;
- filesystem reachability;
- reason and reason timestamp for non-normal state.

## 14.4 GPU device

```rust
pub struct GpuDevice {
    pub physical_id: u16,
    pub gpu_type: String,
    pub memory_total_mib: u64,
    pub allocation: Option<GpuAllocation>,
    pub health: GpuHealth,
    pub utilization_bps: u16,
    pub memory_used_mib: u64,
    pub temperature_millic: i32,
    pub power_milliwatts: u64,
}
```

The initial H200 profile may use a simplified HBM capacity value for training exercises. Hardware quantities are profile metadata, not a claim that every H200 system is configured identically.

## 14.5 Job specification

```rust
pub struct JobSpec {
    pub name: String,
    pub owner: UserId,
    pub account: Option<String>,
    pub partition: Option<String>,
    pub qos: Option<String>,
    pub nodes: NodeRequest,
    pub tasks: TaskRequest,
    pub cpus_per_task: u32,
    pub memory: MemoryRequest,
    pub gres: Vec<GresRequest>,
    pub time_limit: SimDuration,
    pub array: Option<ArraySpec>,
    pub dependencies: Vec<JobDependency>,
    pub reservation: Option<String>,
    pub constraints: Vec<String>,
    pub output_path: VirtualPathTemplate,
    pub error_path: VirtualPathTemplate,
    pub command: CommandPlan,
    pub environment: BTreeMap<String, String>,
    pub submit_context: SubmitContext,
}
```

## 14.6 Job state

```rust
pub struct JobState {
    pub job_id: JobId,
    pub spec: JobSpec,
    pub state: JobStatus,
    pub reason: Option<JobReason>,
    pub submit_time: SimTime,
    pub eligible_time: SimTime,
    pub start_time: Option<SimTime>,
    pub end_time: Option<SimTime>,
    pub allocation: Option<ResourceAllocation>,
    pub steps: BTreeMap<StepId, StepState>,
    pub exit_code: Option<ExitCode>,
    pub stdout: VirtualPath,
    pub stderr: VirtualPath,
    pub accounting: AccountingRecord,
    pub workload: Option<WorkloadInstance>,
}
```

## 14.7 Job status

P0 terminal and active states:

```text
PENDING
CONFIGURING
RUNNING
COMPLETING
COMPLETED
FAILED
CANCELLED
TIMEOUT
OUT_OF_MEMORY
NODE_FAIL
PREEMPTED
```

P1 states:

```text
SUSPENDED
REQUEUED
RESIZING
SPECIAL_EXIT
DEADLINE
BOOT_FAIL
```

Slurm documents `PENDING`, `RUNNING`, `COMPLETED`, `FAILED`, `TIMEOUT`, `OUT_OF_MEMORY`, `PREEMPTED`, and related states. DGX Lab supports a curriculum-relevant subset and labels any intentional simplification [@slurmJobStates].

## 14.8 Pending reason

P0 reasons:

```text
Resources
Priority
Dependency
InvalidAccount
InvalidQOS
QOSMaxJobsPerUserLimit
QOSMaxGRESPerUser
Reservation
PartitionDown
PartitionTimeLimit
NodeNotAvail
ReqNodeNotAvail
BeginTime
```

P1 reasons include association and fair-share limits. The UI explains that a real scheduler may have multiple blocking conditions while `squeue` commonly displays the reason encountered by the scheduling path [@slurmSqueue].

## 14.9 Job step

A job step belongs to an allocation and contains:

- step ID (`batch`, `extern`, numeric, or simulated equivalent);
- task count;
- CPU binding;
- allocated GPU subset within the parent allocation;
- environment;
- command plan;
- state and exit code;
- step telemetry.

The model shall distinguish:

```text
salloc        obtains an allocation
sbatch        submits a batch job whose script later runs in an allocation
srun          launches work and may obtain an allocation when invoked outside one
nested srun   creates a job step inside an existing allocation
```

## 14.10 Accounting record

Accounting stores:

- elapsed time;
- requested and allocated TRES;
- CPU time;
- peak host RAM;
- average and peak GPU utilization;
- peak HBM;
- read/write bytes;
- exit code;
- state;
- start/end timestamps;
- checkpoint count;
- wasted allocation estimates;
- simulated energy estimate where the profile supports it.

The simulator's energy and cost values are explicitly labeled estimates.

---

# 15. State machines

## 15.1 Batch job lifecycle

```text
SUBMISSION_REQUESTED
        │ validate syntax and policy
        ├── invalid ───────────────► REJECTED_AT_SUBMISSION
        ▼
PENDING
        │ scheduler grants resources
        ▼
CONFIGURING
        │ setup succeeds
        ▼
RUNNING
        ├── command exits 0 ───────► COMPLETING ─► COMPLETED
        ├── command exits nonzero ─► COMPLETING ─► FAILED
        ├── host OOM ──────────────► COMPLETING ─► OUT_OF_MEMORY
        ├── wall time ─────────────► COMPLETING ─► TIMEOUT
        ├── user cancel ───────────► COMPLETING ─► CANCELLED
        ├── node failure ──────────► COMPLETING ─► NODE_FAIL
        └── preemption ────────────► PREEMPTED / REQUEUED by profile
```

Submission rejection is not recorded as a scheduler job unless the scenario explicitly teaches a site behavior that accepts then holds invalid jobs. Rejected attempts remain in the learner transcript.

## 15.2 Interactive allocation lifecycle

```text
salloc or allocation-form srun
        ↓
PENDING
        ↓
RUNNING allocation
        ├── one or more steps
        ├── interactive shell state
        └── idle allocation still consumes resources
        ↓
learner exits / time limit / cancellation
        ↓
resource release and accounting
```

## 15.3 Node lifecycle

```text
IDLE ──allocate some resources──► MIXED
MIXED ──allocate remaining──────► ALLOCATED
ALLOCATED/MIXED ──jobs end──────► MIXED/IDLE

IDLE/MIXED/ALLOCATED ──drain──► DRAINING
DRAINING ──last job ends──────► DRAIN
DRAIN ──resume───────────────► IDLE

any schedulable state ──fault──► DOWN/FAIL
DOWN/FAIL ──recover+resume────► IDLE
```

## 15.4 Assessment lifecycle

```text
DRAFT ATTEMPT
   ↓ start, lock revisions and seed
IN_PROGRESS
   ├── abandon ───────────────► ABANDONED
   ├── invalidating action ───► INVALIDATED
   └── submit sections
          ↓
SCORING
   ↓
PASSED or FAILED
   ↓
EVIDENCE_FINALIZED
   ↓
certificate/report export
```

Finalized evidence is immutable within the session store. A correction creates a new assessment record rather than editing prior evidence.

---

# 16. Scheduler behavior

## 16.1 P0 scheduling policy

The MVP scheduler uses:

1. eligibility validation;
2. dependency and begin-time checks;
3. partition and QOS checks;
4. deterministic queue ordering;
5. first feasible resource allocation;
6. configurable backfill-lite behavior only where a lesson requires it.

The default basic profile orders eligible jobs by:

```text
scenario-defined priority override
then certification/guided control priority when explicitly enabled
then submission time
then job ID
```

## 16.2 P1 multifactor policy

The advanced profile adds simplified factors:

```text
priority = age + fair_share + job_size + partition + qos + nice
```

Every factor is visible in `sprio` and the learning panel. The simulator shall not pretend to reproduce every production plugin parameter. It teaches factor interaction with profile-defined weights.

## 16.3 Resource matching

A job may start only when one or more eligible nodes satisfy:

- partition membership;
- node state;
- node count;
- feature constraints;
- free CPU;
- free memory;
- free GPU type/count;
- reservation access;
- QOS and association limits;
- dependency completion;
- begin time;
- time limit permitted by partition/QOS.

## 16.4 CPU allocation

Initial semantics:

- `--cpus-per-task` allocates logical CPUs per task;
- `--ntasks` controls task count;
- total CPU request is derived deterministically;
- node capacity is consumable;
- optional socket/NUMA affinity appears in advanced multi-GPU labs;
- oversubscription is disabled in the default profile.

## 16.5 Memory allocation

Supported forms:

- `--mem=<size>` per node;
- `--mem-per-cpu=<size>`;
- profile default when omitted;
- explicit conflict detection.

The learner-facing explanation distinguishes requested memory from workload usage. A workload may:

- fit within the request;
- waste allocation;
- exceed the cgroup limit and terminate as host OOM;
- fail submission because request exceeds node/partition policy.

## 16.6 GPU allocation

Supported GRES forms:

```text
--gres=gpu:1
--gres=gpu:h200:1
--gpus=1
--gpus-per-node=1
```

Exact accepted forms depend on scenario compatibility. The default profile allocates whole GPUs only.

The model tracks:

- physical virtual GPU IDs;
- job allocation GPU set;
- step GPU subset;
- job-local device mapping;
- simulated `CUDA_VISIBLE_DEVICES`;
- container-visible device list;
- HBM and utilization.

## 16.7 Dependencies

P1 supported dependencies:

```text
after
any
 afterok
afternotok
afterany
singleton
```

The parser supports the documented syntax subset and explains unsupported dependency variants rather than silently treating them as `afterok`.

## 16.8 Job arrays

Arrays support:

- range syntax;
- optional step;
- concurrency limit using `%`;
- task-specific environment;
- `%A` and `%a` output substitutions;
- independent task states;
- aggregate array view;
- selected cancellation.

## 16.9 QOS and limits

P1 profiles model QOS effects on priority, preemption, and resource limits, reflecting the major roles described in Slurm documentation [@slurmQOS; @slurmResourceLimits].

Supported limits include:

- maximum running jobs per user;
- maximum submitted jobs per user;
- maximum GPU TRES per user;
- maximum GPU TRES per job;
- maximum wall time;
- group TRES;
- required reservation;
- partition/QOS eligibility.

## 16.10 Reservations

P1 reservations specify:

- start and end simulated time;
- reserved nodes or TRES;
- permitted users/accounts;
- maintenance or course purpose;
- overlap policy;
- flags modeled by the curriculum.

---

# 17. Supported command model

## 17.1 Command-support principle

DGX Lab supports commands and flags needed by its curriculum. It does not claim to parse the complete Slurm CLI. Each command has a support manifest containing:

- command version;
- supported options;
- aliases;
- incompatible options;
- output profiles;
- known simplifications;
- associated competencies.

Unknown options produce a specific simulated error and link to the supported-reference panel.

## 17.2 P0 Slurm commands

| Command | Required behavior |
|---|---|
| `sinfo` | partitions, node state, CPU/memory/GPU summaries, formatting subset |
| `squeue` | active jobs, pending reasons, user/partition filters, formatting subset |
| `sbatch` | parse script, `#SBATCH`, submit, return job ID, output paths |
| `srun` | obtain allocation as needed or launch step inside allocation |
| `salloc` | obtain interactive allocation |
| `scancel` | cancel job, array task, or own simulated job set |
| `scontrol show job` | detailed job state and reason |
| `scontrol show node` | node resources, state, reason, allocations |
| `sacct` | terminal and historical accounting records |

## 17.3 P1 Slurm commands

| Command | Required behavior |
|---|---|
| `sstat` | live step-level CPU, RAM, and I/O metrics |
| `sprio` | simplified priority factors |
| `squeue --start` | estimated start under deterministic scheduler assumptions |
| `scontrol show partition` | partition configuration |
| `scontrol show reservation` | reservation details |
| `scontrol update` | limited Scenario Control operations only |
| `sacctmgr show` | read-only simulated associations/QOS in advanced lessons |

No administrator mutation command is available in ordinary learner mode.

## 17.4 `sbatch` behavior

The simulator follows these teaching-relevant semantics:

- accepts a virtual script path;
- parses `#SBATCH` directives before the first non-comment/non-whitespace command;
- treats directive text literally rather than as expanded shell variables;
- merges CLI options with directives according to versioned precedence;
- validates request and submits a job;
- returns immediately with a simulated job ID after accepted submission;
- creates default output `slurm-%j.out` when none is specified.

These align with the core documented behavior of `sbatch` [@slurmSbatch].

## 17.5 P0 shell commands

```text
pwd
ls
cd
cat
head
tail
grep
mkdir
touch
cp
mv
rm
echo
env
export
which
history
clear
exit
help
man   simulated curated manual
nano  opens built-in virtual editor
```

## 17.6 P0 workload and environment commands

```text
module avail
module list
module load
module unload
singularity exec
python <registered-script>
torchrun <registered-script>
nvidia-smi
nvidia-smi -L
nvidia-smi topo -m   advanced profile
```

Every command maps to typed simulator behavior. None is executed by the host.

## 17.7 Limited shell grammar

P0 grammar supports:

- whitespace and quoting;
- environment-variable expansion from virtual environment;
- line continuation;
- output append/overwrite redirection;
- simple pipelines for explicitly composable commands;
- `&&` and `;` for supported commands;
- comments;
- shebang ignored as metadata;
- assignment and `export`.

Explicitly unsupported in MVP:

- command substitution;
- arbitrary glob semantics beyond virtual-path patterns;
- shell functions;
- process substitution;
- background host processes;
- traps except scenario-specific checkpoint teaching syntax;
- `eval`;
- arbitrary interpreters.

## 17.8 Output profiles

Output is versioned by:

```text
command
+ simulator compatibility version
+ cluster profile
+ locale
+ formatting flags
```

Golden tests validate common outputs. The About screen states that real sites and Slurm releases may format output differently.

---

# 18. Virtual filesystem

## 18.1 Purpose

The virtual filesystem lets learners create scripts, logs, checkpoints, and datasets without host access.

## 18.2 Initial layout

```text
/
├── home/
│   └── learner/
│       ├── labs/
│       ├── scripts/
│       ├── logs/
│       └── checkpoints/
├── shared/
│   ├── courses/
│   ├── examples/
│   └── teams/
├── datasets/
├── containers/
├── checkpoints/
├── scratch/
└── tmp/
```

## 18.3 File model

Supported types:

- directory;
- regular text file;
- generated log;
- checkpoint artifact metadata;
- dataset placeholder;
- container-image placeholder;
- symbolic link in P1 if required by lessons.

Binary assets are represented by metadata and optional small preview bytes rather than multi-gigabyte fiction occupying IndexedDB.

## 18.4 Permissions

P0 permissions are simplified:

- learner-owned home;
- readable shared course content;
- read-only datasets and containers where configured;
- writable scratch;
- scenario-defined group paths;
- permission-denied errors.

P1 adds group ownership, quotas, and selected mode-bit exercises.

## 18.5 Quotas

The simulator may enforce:

- home quota;
- shared project quota;
- scratch capacity;
- file-count quota;
- checkpoint retention.

Quota exhaustion generates realistic but clearly simulated errors and updates storage views.

## 18.6 Path safety

All paths are normalized within the virtual root. `..` cannot escape. Imported pack paths are validated before insertion. No virtual path maps to a host path.

## 18.7 Persistence

Virtual files are stored as content-addressed records plus directory metadata in IndexedDB. Snapshots reference immutable content hashes to avoid duplicating unchanged files.

---

# 19. Synthetic workload system

## 19.1 Workload principle

A learner command launches a declared synthetic workload model, not code. Workload definitions describe expected resource demand, duration, logs, metrics, artifacts, and failure conditions.

## 19.2 P0 workload families

1. **CPU preprocessing**
   - dataset scan;
   - tokenization or transformation;
   - CPU/RAM/I/O curves;
   - output shard creation.

2. **Single-GPU training**
   - epochs and steps;
   - loss curve;
   - HBM and GPU utilization;
   - checkpoints;
   - batch-size-dependent OOM.

3. **Parameter sweep**
   - job arrays;
   - varied parameters;
   - independent output directories;
   - selected task failure.

4. **Checkpointed training**
   - checkpoint interval;
   - wall-time warning;
   - cancellation;
   - resume validation.

5. **Multi-GPU training**
   - 2- or 4-GPU allocation;
   - ranks;
   - synchronization phases;
   - communication overhead;
   - simulated scaling efficiency.

## 19.3 Workload definition

```yaml
id: pytorch-image-train-v1
kind: training
command_match:
  executable: python
  script: train.py
parameters:
  batch_size:
    type: integer
    default: 64
  epochs:
    type: integer
    default: 5
resource_model:
  min_gpus: 1
  cpu_usage_bps: [3000, 6500]
  ram_mib: 42000
  hbm_formula: "base + batch_size * per_example"
  gpu_usage_bps: [7200, 9600]
duration_model:
  base_sim_seconds: 1080
outputs:
  - logs/train.log
  - checkpoints/epoch-{epoch}.pt
failure_rules:
  - when: hbm_required > hbm_available
    state: OUT_OF_MEMORY
  - when: host_ram_required > allocation_mem
    state: OUT_OF_MEMORY
  - when: duration > time_limit
    state: TIMEOUT
```

Human-authored source is YAML or Markdown, but release packs contain compiled validated data.

## 19.4 Metric generation

Metrics are generated from deterministic phases:

```text
initialization
warm-up
steady training
checkpoint write
validation
completion
```

Each phase defines bounded curves for:

- CPU;
- host RAM;
- GPU utilization;
- HBM;
- power-like estimate;
- temperature-like estimate;
- filesystem I/O;
- network/NCCL-like traffic in advanced profiles.

## 19.5 Logs

Logs are generated from templates and structured events. They may include:

- environment summary;
- rank startup;
- epoch/step progress;
- loss and learning rate;
- checkpoint creation;
- warnings;
- simulated stack traces;
- scheduler messages;
- termination reason.

Logs must contain enough evidence for diagnosis without depending on a hidden answer panel.

## 19.6 Failure distinctions

### GPU OOM

Characteristics:

- HBM demand exceeds job-visible GPU capacity;
- host allocation may remain adequate;
- simulated framework error appears;
- job commonly exits `FAILED`, with an OOM diagnosis tag in evidence;
- remedy may include lower batch size, accumulation, checkpointing, or more GPUs depending on scenario.

### Host-memory OOM

Characteristics:

- process memory exceeds allocated job memory;
- cgroup-like kill event;
- job state `OUT_OF_MEMORY`;
- `sacct` and logs show host memory evidence.

### Timeout

Characteristics:

- workload duration exceeds wall-time;
- optional warning before termination;
- final state `TIMEOUT`;
- existing checkpoints may permit resume.

### Script failure

Characteristics:

- unsupported path, missing file, bad argument, nonzero exit;
- final state `FAILED`;
- no implication that additional resources solve it.

## 19.7 Container simulation

Container definitions model:

- image path and availability;
- declared frameworks;
- GPU capability;
- binds;
- environment modules;
- working directory;
- version metadata;
- selected failure scenarios.

`singularity exec --nv` alters simulated environment and device visibility. It never mounts or parses a real SIF file.

---

# 20. Concurrent virtual users and actors

## 20.1 Actor model

All concurrent users are actors inside the one simulation worker.

```rust
pub enum ActorKind {
    Scripted,
    PolicyDriven,
    BackgroundLoad,
    Infrastructure,
}
```

Actors submit typed actions to the same scheduler path used by the learner. They do not bypass resource validation unless the scenario explicitly models administrator behavior.

## 20.2 Scripted actor

Executes fixed actions at simulated times.

```yaml
id: user-alice
kind: scripted
username: alice
actions:
  - at: 00:00:10
    submit: vision-train.sbatch
  - at: 00:25:00
    cancel: 1002
```

## 20.3 Policy-driven actor

Observes selected public cluster state and chooses from deterministic rules.

```yaml
id: impatient-researcher
kind: policy
policy:
  submit_when_idle_gpus_at_least: 4
  requested_gpus: 4
  resubmit_after_failure: true
  remediation_sequence:
    - reduce_batch_size
    - increase_memory
```

## 20.4 Background-load actor

Maintains a target utilization distribution. It is seeded and constrained so certification difficulty remains reproducible.

## 20.5 Infrastructure actor

May:

- drain or resume a node;
- create maintenance reservation;
- inject GPU fault;
- degrade storage;
- exhaust quota;
- recover a service;
- preempt or requeue jobs under profile policy.

## 20.6 Scale target

The engine shall support at least:

- 100 actors;
- 1,000 jobs in one active scenario;
- 10,000 events between snapshots;
- ×60 time progression;

while preserving responsive interaction on an M1-class Mac.

## 20.7 Actor visibility

Ordinary learner mode sees only information available through simulated commands and cluster views. Hidden scripts, future actions, and grading intent remain concealed. Scenario Control may reveal them outside assessment.

---

# 21. Fault and recovery system

## 21.1 P0 faults

- unsatisfiable resource request;
- invalid partition/account/QOS;
- GPU OOM;
- host-memory OOM;
- timeout;
- learner cancellation;
- script failure;
- missing input;
- invalid `#SBATCH` directive;
- container image missing;
- write permission denied.

## 21.2 P1 faults

- node drain and maintenance;
- node failure;
- GPU XID-like error;
- GPU or fabric degradation;
- shared-storage outage;
- storage quota exceeded;
- checkpoint corruption;
- stale checkpoint incompatibility;
- container mount-format failure;
- network-like collective timeout in multi-node profile;
- preemption and requeue.

## 21.3 Fault declaration

```yaml
faults:
  - id: gpu2-xid
    at: 00:18:00
    target: dgx-h200-01/gpu/2
    type: gpu_fault
    effect:
      fail_allocated_job: true
      node_state: drain
    recover:
      at: 00:35:00
      requires_operator_resume: true
```

## 21.4 Diagnosis evidence

A fault scenario defines acceptable diagnostic evidence, for example:

- learner inspected `squeue` reason;
- learner used `scontrol show job`;
- learner inspected job log;
- learner compared requested and peak memory;
- learner found checkpoint;
- learner resubmitted with valid correction;
- learner avoided an irrelevant resource increase.

## 21.5 Recovery

Recovery may require:

- change request;
- fix script;
- load correct module;
- select valid image;
- wait for reservation;
- resume/requeue;
- use checkpoint;
- ask for appropriate resources;
- acknowledge that user action cannot fix an infrastructure incident.

That last lesson is useful. Not every red panel is a personal invitation to become root.

---

# 22. Cluster profiles

## 22.1 Default `DGX-H200-8`

```yaml
schema: dgxlab.cluster-profile/v1
id: dgx-h200-8
display_name: DGX H200 — Single Node
scheduler:
  family: slurm
  teaching_version: "25.05"
login_nodes:
  - id: dgx-login-01
compute_nodes:
  - id: dgx-h200-01
    cpus: 224
    memory_mib: 1857528
    gpus:
      type: h200
      count: 8
partitions:
  - id: gpu
    default: true
    nodes: [dgx-h200-01]
filesystems:
  home: /home
  shared: /shared
  datasets: /datasets
  containers: /containers
  checkpoints: /checkpoints
isolation:
  cores: true
  memory: true
  devices: true
```

The profile generalizes a validated eight-H200, 224-CPU, cgroup-isolated Slurm system while removing site identifiers and production paths [@orca2026UAT; @orca2026Installation].

## 22.2 `DGX-Contended`

Adds:

- 6–12 background users;
- 60–90% target GPU occupancy;
- mixed 1-, 2-, and 4-GPU jobs;
- pending-resource exercises;
- jobs ending at known future times.

## 22.3 `DGX-Degraded`

Adds:

- drained GPU;
- storage incident;
- one faulted job;
- recovery sequence;
- monitoring and diagnosis exercises.

## 22.4 `DGX-Shared`

Adds:

- accounts;
- QOS;
- reservations;
- user limits;
- simplified multifactor priority;
- fair-share history.

## 22.5 `DGX-MultiNode` P2

A fictional profile with multiple GPU nodes for distributed-training education. It does not imply the source institutional system has this topology.

---

# 23. Scenario and course-pack model

## 23.1 Source and runtime formats

Human-authored source:

```text
YAML
Markdown
localization catalogs
small original images or SVG
```

Release/runtime pack:

```text
manifest
compiled typed scenario data
compiled course content
question bank
localization resources
asset hashes
optional signature
```

No JavaScript, WASM, Rust dynamic library, native binary, shell script execution, or arbitrary HTML is permitted in imported packs.

## 23.2 `.dgxlabpack` container

Logical contents:

```text
manifest.cbor
content.pack
assets/
signatures/
LICENSES/
```

The physical format is a versioned archive with:

- magic bytes;
- schema version;
- compressed-size and expanded-size limits;
- entry-count limits;
- normalized paths;
- per-entry hashes;
- whole-pack digest;
- optional Ed25519 signature for official packs.

The exact archive and serialization choices require an ADR after a WASM size/performance benchmark.

## 23.3 Pack trust states

```text
BUILT_IN
OFFICIAL_SIGNED
LOCAL_UNSIGNED
INVALID_SIGNATURE
QUARANTINED
INCOMPATIBLE
```

Unsigned packs may be imported after a clear warning because they remain non-executable data. Invalidly signed packs are not silently downgraded to unsigned.

## 23.4 Validation

Validation checks:

- magic and schema;
- supported minimum/maximum app version;
- compressed and expanded size;
- file count;
- duplicate and traversal paths;
- hash integrity;
- signature;
- all references resolved;
- no executable content type;
- valid resource bounds;
- deterministic IDs;
- question correctness constraints;
- accepted-answer ambiguity;
- scenario reachability where static analysis is possible;
- localization completeness policy.

## 23.5 Scenario definition

A scenario contains:

- identity and revision;
- cluster profile;
- seed policy;
- learner identity and association;
- initial virtual filesystem;
- actors;
- initial and scheduled jobs;
- faults;
- learning objectives;
- hints;
- grading rules;
- completion conditions;
- certification policy;
- content references.

## 23.6 Course definition

A course contains:

- title, description, and intended audience;
- competencies;
- prerequisites;
- modules and lessons;
- scenario references;
- concept cards;
- practice questions;
- certification blueprint;
- localization;
- license and attribution.

---

# 24. Grading and evidence engine

## 24.1 State-based grading

A practical rule may inspect:

- submitted job specification;
- state transitions;
- commands used;
- virtual files created;
- diagnostic evidence;
- final job outcome;
- resource efficiency;
- prohibited or unsafe actions;
- elapsed simulated and human time;
- hints used.

Example:

```yaml
checks:
  - id: one-gpu
    assert:
      learner_job.request.gpus: 1
    points: 10
    critical: true
  - id: diagnosed-timeout
    any_of:
      - command_used: "sacct"
      - command_used: "scontrol show job"
    and:
      diagnosis: "time_limit"
    points: 10
  - id: checkpoint-survived
    assert:
      virtual_file_exists: "/home/learner/checkpoints/epoch-002.pt"
    points: 15
    critical: true
```

## 24.2 Evidence ledger

Evidence events are append-only:

```rust
pub struct EvidenceEvent {
    pub id: EvidenceId,
    pub sim_time: SimTime,
    pub human_time: Option<Timestamp>,
    pub source_event_seq: u64,
    pub competency: String,
    pub evidence_kind: EvidenceKind,
    pub payload: EvidencePayload,
    pub scenario_revision: String,
}
```

## 24.3 Hint engine

Hints are deterministic and progressive:

1. conceptual nudge;
2. view/command category;
3. specific command suggestion;
4. worked explanation.

Hints may be triggered by:

- learner request;
- repeated invalid command;
- no progress for a scenario-defined interval;
- observed misconception;
- terminal state.

Hint use is recorded separately from competency. Practice reports distinguish **completed with assistance** from **completed independently**.

## 24.4 Knowledge scoring

### Multiple choice

- exact selection match by default;
- option order randomized;
- explanation after allowed submission point.

### Multi-select

Configurable policies:

- all-or-nothing;
- bounded partial credit;
- penalty for incorrect selection;
- minimum correct selections.

No item may award positive score for selecting all options unless all options are actually correct.

### Fill in the blank

Normalization pipeline:

1. Unicode normalization;
2. trim outer whitespace;
3. normalize internal whitespace if item permits;
4. case folding if permitted;
5. command alias canonicalization if permitted;
6. unit normalization if numeric;
7. exact accepted-pattern match;
8. numeric tolerance where defined.

Regex matching is compiled from constrained authoring patterns and protected against pathological expressions.

## 24.5 Critical misconceptions

Examples that may block certification until remediated:

- believing work should run directly on the login node;
- requesting no GPU for a required GPU workload and interpreting failure as hardware fault;
- treating GPU OOM as host-memory OOM without examining evidence;
- assuming a pending job is broken merely because it is pending;
- losing required work because no checkpoint exists in an interruption scenario;
- attempting to bypass scheduler allocation in a simulated command path;
- misidentifying another user's job as safe to cancel.

## 24.6 Alternative valid paths

Grading rules must permit documented equivalent solutions. Course authors are expected to test at least:

- canonical solution;
- one valid alternative;
- one near-miss;
- one false-positive risk;
- one adversarial command sequence.

---

# 25. Certification evidence and reports

## 25.1 Assessment record

```rust
pub struct AssessmentRecord {
    pub assessment_id: String,
    pub learner_display_name: String,
    pub app_version: String,
    pub course_id: String,
    pub course_revision: String,
    pub blueprint_revision: String,
    pub seed: u64,
    pub started_at: Timestamp,
    pub completed_at: Option<Timestamp>,
    pub status: AssessmentStatus,
    pub section_scores: BTreeMap<String, Score>,
    pub competency_results: BTreeMap<String, CompetencyResult>,
    pub assisted: bool,
    pub evidence_digest: Option<String>,
}
```

## 25.2 Finalization

Finalization:

1. validates event-log completeness;
2. computes scores from pinned rules;
3. checks critical competencies;
4. records application/course/scenario revisions;
5. serializes canonical evidence;
6. computes SHA-256 digest;
7. creates certificate metadata;
8. prevents mutation of finalized record.

## 25.3 Export formats

- `.dgxlab` complete session/evidence bundle;
- PDF certificate where platform rendering is qualified;
- self-contained HTML certificate/report;
- Markdown report;
- JSON evidence summary;
- CSV competency table.

HTML is the required portable report baseline. PDF is P0 for v1.0 but may be produced through a deterministic print/export pipeline rather than a native PDF engine if cross-platform fidelity is better.

## 25.4 Verification

A verifier can:

- import the `.dgxlab` bundle;
- validate hashes;
- inspect application/course/scenario versions;
- replay practical state transitions where compatible;
- recalculate scoring;
- compare the certificate digest.

This detects accidental or casual tampering. It does not defeat a determined user who modifies the open-source application or forges an entire self-consistent local bundle.

## 25.5 Privacy

Reports contain only locally entered display name and learning evidence. They do not contain host usernames, machine serials, network identifiers, or hidden device fingerprints.

---

# 26. Persistence, event sourcing, and recovery

## 26.1 IndexedDB stores

```text
settings
courses
packs
progress
sessions
events
snapshots
virtual_file_blobs
assessments
reports
migrations
```

## 26.2 Event log

Each event includes:

- session ID;
- monotonic sequence;
- simulated time;
- human timestamp when relevant;
- actor;
- event type;
- versioned payload;
- prior-event hash in finalized assessment mode where enabled.

## 26.3 Snapshot policy

Create a snapshot:

- every 250 logical events by default;
- before and after a lab checkpoint;
- before assessment section transition;
- on app background/close request where time permits;
- on explicit save;
- before importing or migrating a session.

## 26.4 Restore

```text
load latest valid snapshot
        ↓
validate snapshot schema and digest
        ↓
replay subsequent events
        ↓
verify resulting sequence and state digest
        ↓
resume UI
```

If the latest snapshot is corrupt, fall back to the previous valid snapshot. If replay fails, open a read-only recovery report and preserve raw data for export.

## 26.5 Migration

The application supports read/migrate behavior for at least the previous two major session schema versions.

Migration rules:

- never mutate the only copy without backup;
- produce a migration record;
- preserve original bundle digest;
- fail read-only when semantic migration is unsafe;
- never silently reinterpret assessment answers under new scoring rules.

## 26.6 Storage management

Settings displays:

- total local storage estimate;
- sessions and reports;
- imported packs;
- virtual-file blobs;
- removable caches;
- protected assessment evidence.

Deletion is explicit and reversible only where a local trash policy is implemented. Built-in course assets are bundled and not duplicated per session.

---

# 27. Session export and import

## 27.1 `.dgxlab` bundle

Logical structure:

```text
manifest.cbor
session.cbor
progress.cbor
events.cbor.zst
snapshots/
virtual-files/
assessment/
reports/
hashes.cbor
```

## 27.2 Manifest

Required fields:

- file magic and format version;
- originating application version;
- minimum compatible application version;
- session ID;
- scenario/course references and digests;
- seed and simulator compatibility version;
- contents and sizes;
- compression algorithms;
- hash algorithm;
- assessment finalization state;
- optional external countersignature metadata.

## 27.3 Import validation

- user-selected file only;
- extension and magic check;
- maximum compressed size;
- maximum expanded size;
- entry-count limit;
- no path traversal;
- hash validation;
- schema validation;
- version compatibility;
- duplicate-session handling;
- assessment immutability;
- quarantine on failure.

## 27.4 Duplicate handling

Options:

- open read-only;
- import as copy with new local session ID;
- replace local copy only after explicit confirmation;
- retain both evidence digests.

---

# 28. Accessibility and localization

## 28.1 Accessibility target

DGX Lab targets WCAG 2.2 AA principles where applicable to a desktop WebView application.

Required:

- full keyboard navigation;
- visible focus;
- semantic headings and landmarks;
- screen-reader labels;
- table alternatives for visual diagrams;
- terminal transcript mode;
- scalable text without broken layout;
- sufficient contrast;
- reduced motion;
- no color-only state communication;
- accessible question controls;
- time controls that do not disadvantage assistive-technology users;
- certification accommodations recorded without lowering competency requirements.

## 28.2 Terminal accessibility

The terminal provides:

- ARIA live-region policy that avoids reading every metric update;
- command and output grouping;
- optional plain transcript view;
- shortcuts to latest prompt and latest error;
- text alternatives for colored state;
- no essential mouse-only interaction.

## 28.3 Internationalization

All UI strings use stable localization keys. Course content and simulator output are separated:

- UI locale;
- course-content locale;
- simulated command-output locale.

Initial v1:

- English UI;
- English built-in course;
- command output in English to match common Slurm environments.

P1:

- Thai UI;
- Thai concept cards and explanations;
- commands and canonical scheduler output remain English, with Thai annotations.

## 28.4 Locale-neutral grading

Practical grading uses state, not translated text. Knowledge questions have locale-specific accepted answers and shared competency IDs.

---

# 29. Security and structural isolation

## 29.1 Security invariant

> **DGX Lab shall be unable to execute or forward a learner command to the host operating system or a real cluster.**

## 29.2 Runtime attack surface

| Surface | Policy |
|---|---|
| Tauri commands | minimal, enumerated, capability-scoped |
| Imported packs | data only, validated, size-limited |
| Web content | bundled only, restrictive CSP |
| Network | no runtime client capability; connections denied |
| Filesystem | user-selected import/export only |
| Processes | no spawn/shell/sidecar |
| Dynamic code | prohibited |
| HTML in course packs | sanitized or compiled restricted markup only |
| JavaScript in packs | prohibited |
| WASM in packs | prohibited |

## 29.3 Forbidden dependencies and APIs

Release CI shall flag or deny:

- `std::process::Command` in application/runtime crates;
- shell/process Tauri plugins;
- SSH libraries;
- Slurm client libraries;
- arbitrary HTTP/WebSocket clients;
- dynamic library loading;
- embedded interpreters;
- runtime `eval` or imported JavaScript;
- unrestricted filesystem plugins;
- external runtime URLs.

Build tooling may use network and processes outside release runtime code, but the boundary must be explicit.

## 29.4 Unsafe Rust

Application-owned runtime crates use:

```rust
#![forbid(unsafe_code)]
```

A dependency containing unsafe code requires ordinary supply-chain review but does not invalidate the policy for owned crates. Any unavoidable application-owned exception requires an ADR, isolated crate, tests, and review.

## 29.5 Threat model

| Threat | Mitigation |
|---|---|
| Simulated command reaches host shell | no process API, typed parser, CI deny rules |
| Course pack includes executable payload | compiled data schema, content-type allowlist, no dynamic loading |
| Path traversal during import | normalized archive paths, user-selected file, expansion limits |
| Decompression bomb | compressed/expanded size and entry-count limits |
| XSS through course content | restricted markup compiler, escaping, CSP, no arbitrary HTML |
| Frontend compromise invokes broad native API | minimal Tauri capabilities and commands |
| Native import command reads arbitrary path | file-dialog token/path binding, extension and size checks |
| External network exfiltration | no HTTP capability, restrictive CSP, offline tests |
| Assessment answer leakage through Scenario Control | certification invalidation and mode lock |
| Local certificate presented as proctored identity proof | explicit trust-level wording and evidence metadata |
| Pack signature confusion | invalid signature never treated as unsigned trusted pack |
| Nondeterministic cross-platform scoring | integer/fixed-point core, deterministic tests and golden replay |
| Data loss on crash | event log, snapshots, atomic IndexedDB transactions, recovery fallback |
| Trademark confusion | original branding, disclaimer, no NVIDIA logo, public-release review |

## 29.6 Network denial testing

Release tests run the complete built-in course with:

- network disabled;
- DNS unavailable;
- proxy unavailable;
- CSP reporting any attempted connection;
- no external assets in the bundle.

Any runtime connection attempt fails the release gate.

## 29.7 No real-backend architecture

There shall be no public trait such as:

```rust
trait SchedulerBackend {
    fn submit(...);
}
```

implemented by both simulator and hypothetical real scheduler. Interfaces are named for simulation semantics, such as `SimulationWorld` and `SchedulerModel`. A future real-cluster product would be a separate repository and security boundary.

---

# 30. Privacy and telemetry

## 30.1 Data collected

Local-only data may include:

- learner-entered display name;
- progress;
- commands typed in the simulator;
- virtual files;
- question responses;
- scores;
- hints;
- timestamps;
- imported packs;
- reports.

## 30.2 Data not collected

DGX Lab does not collect or transmit:

- account email;
- real system username unless manually entered into a report;
- IP address;
- machine serial;
- hardware fingerprint;
- crash telemetry;
- usage analytics;
- clipboard history;
- external browsing behavior.

## 30.3 Telemetry policy

No telemetry endpoint exists in v1. Debug logs remain local and exclude host paths unless required for import/export diagnostics. A future opt-in telemetry feature would require a new PRD amendment and is not presumed.

## 30.4 User control

The user can:

- inspect stored sessions;
- export all learning data;
- delete individual sessions;
- delete imported packs;
- reset application data;
- preserve finalized certificates separately before reset.

---

# 31. Branding, licensing, and public-release boundary

## 31.1 Name

The approved working product name is **DGX Lab**.

## 31.2 Independent-product disclaimer

Every public release, About page, repository README, and certificate footer shall include a concise statement such as:

> DGX Lab is an independent educational simulator. It is not affiliated with, sponsored by, or endorsed by NVIDIA Corporation or SchedMD LLC. NVIDIA, DGX, H200, CUDA, and related marks are trademarks or registered trademarks of their respective owners. Slurm is a trademark of SchedMD LLC.

Final wording requires legal/trademark review.

## 31.3 Visual identity

Prohibited:

- NVIDIA logo;
- imitation of NVIDIA brand style;
- NVIDIA green as a deliberate brand imitation;
- copied DGX product imagery;
- claims of certification by NVIDIA or SchedMD.

Required:

- original DGX Lab logo and visual system;
- neutral hardware diagrams;
- clear “simulated” labels;
- independent-project disclaimer.

NVIDIA's current brand guidance states that logo and branded-element use requires authorization and that unapproved branding must not imply affiliation or endorsement [@nvidia2026Brand]. Because `DGX` is itself a trademark, public release is gated on a focused naming review [@nvidiaTrademarks]. If review determines that the product name creates unacceptable confusion, the codebase shall preserve a rename path through centralized product strings and identifiers.

## 31.4 Licenses

- code: Apache-2.0;
- built-in original course content: CC BY 4.0;
- external dependencies: included in third-party notices;
- imported packs: carry their own license metadata;
- generated certificates: user-owned evidence, with template attribution where required.

## 31.5 Repository policy

The repository shall include:

- `LICENSE`;
- `CONTENT-LICENSE`;
- `NOTICE`;
- `TRADEMARKS.md`;
- `THIRD_PARTY_NOTICES`;
- contributor guidelines;
- security policy;
- pack-authoring license guidance.


---

# 32. Functional requirements

Priorities:

- **P0:** required for the smallest useful MVP or foundational v1 behavior.
- **P1:** required for the complete v1.0 course/certification release unless otherwise noted.
- **P2:** deferred advanced capability.

## 32.1 Application shell and lifecycle

| ID | Requirement | Priority |
|---|---|---|
| APP-001 | DGX Lab shall launch as a standalone Tauri 2 desktop application without starting a localhost server. | P0 |
| APP-002 | The primary macOS build shall support Apple Silicon. | P0 |
| APP-003 | v1.0 shall provide qualified builds for macOS Apple Silicon, Windows x86-64, and Linux x86-64. | P1 |
| APP-004 | The application shall run without internet access after installation. | P0 |
| APP-005 | All runtime assets, fonts, icons, courses, and default profiles shall be bundled locally. | P0 |
| APP-006 | The application shall restore the most recent valid session after an unclean shutdown. | P0 |
| APP-007 | The application shall expose no automatic updater in v1. | P0 |
| APP-008 | The Tauri process shall expose only approved window, metadata, and import/export capabilities. | P0 |
| APP-009 | The application shall not include shell, process, HTTP, WebSocket, sidecar, unrestricted filesystem, or SQL Tauri plugins. | P0 |
| APP-010 | A release build shall fail CI when forbidden Tauri capabilities are present. | P0 |
| APP-011 | Native import/export commands shall validate file type, size, and user selection before reading or writing. | P0 |
| APP-012 | The application shall expose version, build ID, simulator compatibility version, and course-pack compatibility in About. | P0 |
| APP-013 | Application-owned runtime Rust crates shall forbid unsafe code unless an isolated approved exception exists. | P0 |
| APP-014 | The application shall support a future static web build without moving simulation into native-only services. | P1 |

## 32.2 User interface

| ID | Requirement | Priority |
|---|---|---|
| UI-001 | The UI shall be implemented with Leptos client-side rendering compiled to WASM. | P0 |
| UI-002 | The main interface shall contain learning, terminal/editor, cluster, and detail/timeline regions. | P0 |
| UI-003 | Users shall be able to resize or collapse major panels. | P0 |
| UI-004 | Panel layout shall persist locally per device. | P0 |
| UI-005 | Job IDs, node IDs, and virtual paths shall be clickable where they resolve to a detail view. | P0 |
| UI-006 | All authoritative state shall come from the simulation worker, not duplicated UI state machines. | P0 |
| UI-007 | The UI shall detect missing/out-of-order worker deltas and request a full state snapshot. | P0 |
| UI-008 | The cluster view shall show node, CPU, RAM, GPU, queue, and simulated-time state. | P0 |
| UI-009 | The job detail view shall show requests, allocation, state, reason, steps, logs, telemetry, and accounting. | P0 |
| UI-010 | The terminal shall be clearly labeled as simulated. | P0 |
| UI-011 | The terminal shall support history, completion, clickable references, and transcript view. | P0 |
| UI-012 | The application shall provide an integrated virtual text editor for batch scripts. | P0 |
| UI-013 | The editor shall never open or modify host files directly. | P0 |
| UI-014 | The application shall provide calm light and dark themes. | P0 |
| UI-015 | The application shall honor reduced-motion preference. | P0 |
| UI-016 | Animations shall not be required to understand state. | P0 |
| UI-017 | The home screen shall prioritize resume, next competency, readiness, and recovery actions. | P0 |
| UI-018 | Scenario Control shall be visibly distinct from learner mode. | P0 |
| UI-019 | Entering Scenario Control during certification shall invalidate the attempt. | P0 |
| UI-020 | The UI shall remain usable at 125%, 150%, and 200% text scaling. | P1 |

## 32.3 Simulation core

| ID | Requirement | Priority |
|---|---|---|
| SIM-001 | The simulation shall use a deterministic discrete-event model. | P0 |
| SIM-002 | Equal compatibility version, profile, scenario, seed, and learner event sequence shall reproduce equal logical outcomes. | P0 |
| SIM-003 | The simulator shall use an explicitly versioned pseudo-random generator. | P0 |
| SIM-004 | Simulation decisions shall not depend on system time, operating-system randomness, or unordered map iteration. | P0 |
| SIM-005 | The simulator shall execute in a dedicated Web Worker in production. | P0 |
| SIM-006 | The pure simulation core shall compile natively for tests and benchmarks. | P0 |
| SIM-007 | The simulation shall support pause, single-event step, real time, ×10, and ×60. | P0 |
| SIM-008 | Practice mode shall support advance-to-next-relevant-event. | P0 |
| SIM-009 | Certification scenarios shall be able to restrict clock controls. | P0 |
| SIM-010 | The worker shall process bounded batches and yield to the message loop. | P0 |
| SIM-011 | UI metric deltas may be coalesced without removing logical events from replay. | P0 |
| SIM-012 | Resource and scoring arithmetic shall use deterministic integer/fixed-point representations where practical. | P0 |
| SIM-013 | The simulator shall support at least 100 actors and 1,000 jobs in one scenario. | P1 |
| SIM-014 | The simulator shall replay at least 10,000 events deterministically. | P1 |
| SIM-015 | The simulator shall provide state digests for snapshots and finalized assessments. | P0 |

## 32.4 Cluster and scheduler model

| ID | Requirement | Priority |
|---|---|---|
| SCH-001 | The default profile shall contain one generic login node and one eight-GPU H200-class compute node. | P0 |
| SCH-002 | The default compute node shall expose 224 logical CPUs and approximately 1.86 TB allocatable memory. | P0 |
| SCH-003 | The default profile shall use generic names and paths, with no institutional identifiers. | P0 |
| SCH-004 | The scheduler shall model jobs, allocations, job steps, nodes, partitions, users, accounts, and resources. | P0 |
| SCH-005 | The scheduler shall model whole-GPU GRES allocation. | P0 |
| SCH-006 | The scheduler shall track physical virtual GPU allocation and job-local visibility mapping. | P0 |
| SCH-007 | The scheduler shall model consumable CPU and memory. | P0 |
| SCH-008 | The scheduler shall reject unsatisfiable requests according to profile policy. | P0 |
| SCH-009 | The scheduler shall support `PENDING`, `CONFIGURING`, `RUNNING`, `COMPLETING`, `COMPLETED`, `FAILED`, `CANCELLED`, `TIMEOUT`, `OUT_OF_MEMORY`, `NODE_FAIL`, and `PREEMPTED`. | P0 |
| SCH-010 | Pending jobs shall retain a typed reason code. | P0 |
| SCH-011 | The P0 scheduler shall implement deterministic FIFO/resource behavior with explicit overrides. | P0 |
| SCH-012 | P1 shall add simplified multifactor priority and fair-share. | P1 |
| SCH-013 | The scheduler shall support job arrays and task concurrency limits. | P1 |
| SCH-014 | The scheduler shall support core dependency types used by the curriculum. | P1 |
| SCH-015 | The scheduler shall support QOS limits in advanced profiles. | P1 |
| SCH-016 | The scheduler shall support time-bounded reservations in advanced profiles. | P1 |
| SCH-017 | The scheduler shall support node drain, draining, down, and resume behavior. | P1 |
| SCH-018 | The scheduler shall distinguish submission rejection from accepted pending jobs. | P0 |
| SCH-019 | Resource release shall occur deterministically at job/step termination. | P0 |
| SCH-020 | The scheduler shall expose request, allocation, usage, and accounting as separate concepts. | P0 |

## 32.5 Command model

| ID | Requirement | Priority |
|---|---|---|
| CMD-001 | Every learner command shall pass through a typed simulator parser. | P0 |
| CMD-002 | No learner command shall be forwarded to a host shell, interpreter, or native process. | P0 |
| CMD-003 | P0 shall implement `sinfo`, `squeue`, `sbatch`, `srun`, `salloc`, `scancel`, `scontrol show job`, `scontrol show node`, and `sacct`. | P0 |
| CMD-004 | P1 shall implement `sstat`, `sprio`, `squeue --start`, partition/reservation views, and read-only accounting views. | P1 |
| CMD-005 | `sbatch` shall parse supported `#SBATCH` directives from virtual scripts. | P0 |
| CMD-006 | The parser shall stop recognizing `#SBATCH` directives after the first non-comment/non-whitespace command. | P0 |
| CMD-007 | Unsupported commands and options shall fail explicitly with curriculum-safe guidance. | P0 |
| CMD-008 | The parser shall support quoting, environment variables, line continuation, selected redirection, and selected pipelines. | P0 |
| CMD-009 | P0 shall support curated shell/file commands required by lessons. | P0 |
| CMD-010 | `module`, `singularity`, `python`, `torchrun`, and `nvidia-smi` shall map only to registered simulation behavior. | P0 |
| CMD-011 | Command support shall be versioned and exposed in an in-app reference. | P0 |
| CMD-012 | Output for common command/flag combinations shall be golden-tested. | P0 |
| CMD-013 | Command output shall be labeled as DGX Lab behavior, not universal Slurm output. | P0 |
| CMD-014 | Learner mode shall not expose simulated administrator mutation commands. | P0 |

## 32.6 Virtual filesystem and editor

| ID | Requirement | Priority |
|---|---|---|
| VFS-001 | Every session shall have an isolated virtual filesystem. | P0 |
| VFS-002 | Virtual paths shall never map to host paths. | P0 |
| VFS-003 | The virtual root shall include generic home, shared, dataset, container, checkpoint, scratch, and temporary paths. | P0 |
| VFS-004 | The filesystem shall support directories and regular text files. | P0 |
| VFS-005 | Logs and checkpoint metadata shall appear as virtual artifacts. | P0 |
| VFS-006 | Basic ownership and permission errors shall be supported. | P0 |
| VFS-007 | P1 shall support quota and capacity scenarios. | P1 |
| VFS-008 | All virtual path resolution shall normalize `.` and `..` without permitting escape. | P0 |
| VFS-009 | Large simulated binary artifacts shall be represented by metadata rather than full payloads. | P0 |
| VFS-010 | The integrated editor shall save only to the virtual filesystem. | P0 |
| VFS-011 | Content blobs shall be deduplicated by hash where practical. | P1 |

## 32.7 Workloads and telemetry

| ID | Requirement | Priority |
|---|---|---|
| WRK-001 | Workloads shall be declarative synthetic models, not executable code. | P0 |
| WRK-002 | P0 shall include CPU preprocessing and single-GPU training workloads. | P0 |
| WRK-003 | P1 shall include parameter sweeps, checkpointed training, and multi-GPU workloads. | P1 |
| WRK-004 | Workloads shall produce deterministic logs and artifacts. | P0 |
| WRK-005 | Workloads shall model time-varying CPU, RAM, GPU, HBM, and I/O. | P0 |
| WRK-006 | Multi-GPU workloads shall model rank startup and communication phases. | P1 |
| WRK-007 | Workloads shall support profile/scenario parameterization. | P0 |
| WRK-008 | Workloads shall declare failure rules. | P0 |
| WRK-009 | The simulator shall distinguish GPU OOM, host-memory OOM, timeout, and script failure. | P0 |
| WRK-010 | Telemetry views shall derive from the same workload state used for logs and accounting. | P0 |
| WRK-011 | Simulated energy or cost estimates shall be labeled estimates. | P1 |
| WRK-012 | Imported packs shall not define executable workload plugins. | P0 |

## 32.8 Actors and scenarios

| ID | Requirement | Priority |
|---|---|---|
| ACT-001 | The simulator shall support scripted virtual users. | P0 |
| ACT-002 | P1 shall support policy-driven and background-load actors. | P1 |
| ACT-003 | The simulator shall support infrastructure actors for fault events. | P1 |
| ACT-004 | Actor actions shall use the same validation/scheduling path as learner actions unless explicitly marked administrator behavior. | P0 |
| ACT-005 | Actor IDs, names, accounts, and actions shall be deterministic from scenario data and seed. | P0 |
| ACT-006 | Hidden future actor actions shall not be visible in learner mode. | P0 |
| ACT-007 | Scenario Control shall expose actor scripts and future events outside assessment. | P1 |
| ACT-008 | Ordinary lessons shall support at least 12 visible concurrent users. | P0 |
| ACT-009 | The engine shall support at least 100 actors for stress scenarios. | P1 |

## 32.9 Faults and recovery

| ID | Requirement | Priority |
|---|---|---|
| FLT-001 | P0 scenarios shall include invalid request, GPU OOM, host OOM, timeout, cancellation, script error, missing input, and permission error. | P0 |
| FLT-002 | P1 scenarios shall include node drain/down, GPU fault, storage outage, quota exhaustion, checkpoint corruption, and container failure. | P1 |
| FLT-003 | Faults shall alter scheduler, workload, logs, telemetry, and accounting consistently. | P0 |
| FLT-004 | Fault recovery shall be scenario-defined and deterministic. | P0 |
| FLT-005 | Practical grading shall distinguish diagnosis from remediation. | P0 |
| FLT-006 | Scenarios shall allow a correct conclusion that the learner cannot directly remediate an infrastructure fault. | P1 |
| FLT-007 | Fault output shall be realistic but clearly simulated and independently authored. | P0 |

## 32.10 Learning system

| ID | Requirement | Priority |
|---|---|---|
| LRN-001 | DGX Lab shall provide guided lessons and free-play practice. | P0 |
| LRN-002 | MVP shall ship with at least four complete guided labs. | P0 |
| LRN-003 | v1.0 shall ship with twelve complete labs covering competencies C1–C12. | P1 |
| LRN-004 | Lessons shall include concise concept cards and command references. | P0 |
| LRN-005 | Hints shall be deterministic, progressive, and recorded. | P0 |
| LRN-006 | Practice completion shall distinguish independent and assisted completion. | P0 |
| LRN-007 | Learning objectives shall map to stable competency IDs. | P0 |
| LRN-008 | Practical grading shall be state/evidence-based rather than exact command-string matching. | P0 |
| LRN-009 | Equivalent valid solution paths shall receive credit when covered by grading rules. | P0 |
| LRN-010 | A course shall declare prerequisites and completion policy. | P0 |
| LRN-011 | The UI shall recommend remediation after failed evidence checks. | P1 |
| LRN-012 | No online LLM shall be required for instruction, hints, or scoring. | P0 |

## 32.11 Knowledge assessment

| ID | Requirement | Priority |
|---|---|---|
| QST-001 | The question engine shall support single-answer multiple choice. | P0 |
| QST-002 | The question engine shall support multi-select. | P1 |
| QST-003 | The question engine shall support fill-in-the-blank. | P0 |
| QST-004 | Fill-in-the-blank shall support accepted aliases, normalized whitespace, case policy, and numeric tolerance. | P0 |
| QST-005 | Questions shall map to competencies and difficulty bands. | P0 |
| QST-006 | Option order shall be randomizable deterministically. | P0 |
| QST-007 | Question selection shall follow a versioned assessment blueprint. | P1 |
| QST-008 | Explanations shall be shown according to practice/certification policy. | P0 |
| QST-009 | Multi-select partial-credit policy shall be explicit and bounded. | P1 |
| QST-010 | Question authoring validation shall detect missing correct answers and duplicate options. | P0 |
| QST-011 | Runtime answer matching shall not use an LLM. | P0 |
| QST-012 | Regex-like accepted patterns shall be restricted and tested for pathological behavior. | P1 |

## 32.12 Certification

| ID | Requirement | Priority |
|---|---|---|
| CERT-001 | v1.0 shall provide a certification workflow combining knowledge and practical assessment. | P1 |
| CERT-002 | Default weights shall be 60% practical, 25% multiple-choice/multi-select, and 15% fill-in-the-blank. | P1 |
| CERT-003 | Default pass policy shall require 80% overall and 70% knowledge score. | P1 |
| CERT-004 | Critical practical competencies shall be mandatory regardless of aggregate score. | P1 |
| CERT-005 | Certification attempts shall pin app, course, blueprint, scenario, question-bank, and seed revisions. | P1 |
| CERT-006 | Scenario Control shall be disabled or invalidate a certification attempt. | P1 |
| CERT-007 | Hints shall be disabled by default in certification; any permitted use shall mark the attempt assisted. | P1 |
| CERT-008 | The default certification session shall allow up to two attempts. | P1 |
| CERT-009 | The application shall generate a locally verifiable evidence digest. | P1 |
| CERT-010 | The application shall generate a certificate and detailed competency report. | P1 |
| CERT-011 | The certificate shall state its standalone/local trust level. | P1 |
| CERT-012 | The system shall support later instructor countersignature metadata without claiming institutional verification in v1. | P2 |
| CERT-013 | Finalized assessment evidence shall be immutable in local storage. | P1 |
| CERT-014 | A finalized assessment shall be replayable or rescored under its pinned compatible rules. | P1 |

## 32.13 Persistence and recovery

| ID | Requirement | Priority |
|---|---|---|
| PER-001 | Sessions, progress, virtual files, and assessment evidence shall persist in IndexedDB. | P0 |
| PER-002 | The application shall autosave after learner commands and significant state transitions. | P0 |
| PER-003 | The application shall create periodic snapshots. | P0 |
| PER-004 | Restore shall load the latest valid snapshot and replay subsequent events. | P0 |
| PER-005 | Restore shall fall back to an earlier valid snapshot after corruption. | P1 |
| PER-006 | Reset-to-scenario-start shall be available in Practice mode. | P0 |
| PER-007 | Full rewind and branching shall be available in P1. | P1 |
| PER-008 | Sessions shall export to `.dgxlab`. | P0 |
| PER-009 | `.dgxlab` import shall validate size, paths, schemas, and hashes. | P0 |
| PER-010 | The app shall support at least the previous two major session schema versions through read or migration. | P1 |
| PER-011 | Migration shall preserve original evidence and never silently rescore a finalized attempt. | P1 |
| PER-012 | Storage management shall show local usage and deletion controls. | P0 |

## 32.14 Course and scenario packs

| ID | Requirement | Priority |
|---|---|---|
| PACK-001 | Built-in content shall use compiled, validated pack data. | P0 |
| PACK-002 | v1.0 shall import `.dgxlabpack` files. | P1 |
| PACK-003 | Imported packs shall contain data only and no executable code. | P0 |
| PACK-004 | Pack import shall validate magic, schema, compatibility, size, entry count, paths, hashes, and references. | P0 |
| PACK-005 | Official packs may use an embedded-public-key signature scheme. | P1 |
| PACK-006 | Invalid signatures shall not be treated as unsigned trusted packs. | P1 |
| PACK-007 | The UI shall display trust and compatibility state. | P1 |
| PACK-008 | Unsigned local packs may be imported after explicit warning. | P1 |
| PACK-009 | Course-pack source authoring shall remain external in v1. | P0 |
| PACK-010 | A scenario compiler/validator CLI shall be delivered in P1. | P1 |
| PACK-011 | Pack licenses and attribution shall be displayed. | P1 |
| PACK-012 | The pack format shall be versioned independently from the session format. | P0 |

## 32.15 Reports and export

| ID | Requirement | Priority |
|---|---|---|
| RPT-001 | The app shall provide a command transcript and job timeline. | P0 |
| RPT-002 | The app shall provide a competency matrix. | P0 |
| RPT-003 | Reports shall distinguish practice, assessment, assisted, and independent evidence. | P0 |
| RPT-004 | The app shall export a human-readable HTML or Markdown learning report. | P0 |
| RPT-005 | v1.0 shall export a certificate as PDF or deterministic print-ready HTML. | P1 |
| RPT-006 | The app shall export JSON evidence and CSV competency data. | P1 |
| RPT-007 | Reports shall not include machine identifiers or host paths by default. | P0 |
| RPT-008 | A report shall include app/course/scenario versions and evidence digest. | P1 |

## 32.16 Accessibility and localization

| ID | Requirement | Priority |
|---|---|---|
| A11Y-001 | The application shall be usable through keyboard-only navigation. | P0 |
| A11Y-002 | Interactive controls shall have semantic labels and visible focus. | P0 |
| A11Y-003 | Visual cluster views shall have table/text alternatives. | P0 |
| A11Y-004 | The terminal shall provide a screen-reader-friendly transcript mode. | P0 |
| A11Y-005 | Reduced-motion mode shall disable nonessential animation. | P0 |
| A11Y-006 | State shall not be communicated by color alone. | P0 |
| A11Y-007 | Certification timing policy shall support declared accommodations. | P1 |
| I18N-001 | All UI strings shall use localization keys. | P0 |
| I18N-002 | UI, course, and simulated-output locale shall be separately represented. | P0 |
| I18N-003 | English UI and course content shall ship in v1. | P0 |
| I18N-004 | Thai UI/course pack shall be supported as P1 content. | P1 |
| I18N-005 | Practical scoring shall remain locale-neutral. | P0 |

## 32.17 Security and privacy

| ID | Requirement | Priority |
|---|---|---|
| SEC-001 | The runtime shall contain no code path that invokes real Slurm or SSH. | P0 |
| SEC-002 | The runtime shall contain no host process-spawn API in application-owned crates. | P0 |
| SEC-003 | The runtime shall contain no arbitrary HTTP/WebSocket client capability. | P0 |
| SEC-004 | The release CSP shall deny external runtime resources and connections. | P0 |
| SEC-005 | Complete built-in-course execution shall pass with network disabled. | P0 |
| SEC-006 | Course content shall not include arbitrary HTML or script. | P0 |
| SEC-007 | Imported archives shall enforce expansion, count, and path limits. | P0 |
| SEC-008 | CI shall scan for forbidden dependencies, capabilities, APIs, and external URLs. | P0 |
| SEC-009 | Developer/Scenario Control shall not enable host execution. | P0 |
| SEC-010 | No `RealSlurmBackend` or equivalent interface shall exist in the repository. | P0 |
| PRIV-001 | No telemetry shall be transmitted in v1. | P0 |
| PRIV-002 | No cloud account or progress synchronization shall be required. | P0 |
| PRIV-003 | Learner data shall remain local unless explicitly exported. | P0 |
| PRIV-004 | Users shall be able to delete local sessions and imported packs. | P0 |
| PRIV-005 | Reports shall minimize personal data. | P0 |

---

# 33. Non-functional requirements

| ID | Requirement |
|---|---|
| NFR-001 | Cold start to usable Home screen shall be under 3 seconds on an M1-class Mac after installation, excluding first OS security verification. |
| NFR-002 | Resume of an ordinary session shall complete in under 1 second after IndexedDB open on target hardware. |
| NFR-003 | Replaying 10,000 events from a valid snapshot shall complete in under 2 seconds on target hardware. |
| NFR-004 | Terminal command acknowledgement shall appear within 50 ms for non-advancing commands and within 100 ms for normal simulated scheduling actions. |
| NFR-005 | UI frame interaction shall remain responsive while 100 actors and 1,000 jobs are simulated at ×60. |
| NFR-006 | The simulation worker shall not block the UI main thread for more than 50 ms due to simulation processing. |
| NFR-007 | Cross-platform deterministic golden scenarios shall produce identical canonical state/evidence digests. |
| NFR-008 | Runtime network operation shall not be required or attempted. |
| NFR-009 | An unclean application close shall lose no more than the most recent uncommitted UI-only change; committed commands shall be recoverable. |
| NFR-010 | Imported pack validation shall fail safely without partially activating content. |
| NFR-011 | A malformed learner command shall not crash the simulation worker. |
| NFR-012 | A worker crash shall preserve the last valid persisted state and provide a recovery path. |
| NFR-013 | All P0 features shall have automated tests and documented failure behavior. |
| NFR-014 | Application-owned domain crates shall have no dependency on Tauri or browser APIs. |
| NFR-015 | The public API surface among crates shall use typed versioned contracts. |
| NFR-016 | Release builds shall be reproducible to the degree practical and record compiler, dependency, and build metadata. |
| NFR-017 | No external font, script, stylesheet, image, or course asset shall be fetched at runtime. |
| NFR-018 | The application shall meet keyboard-first and screen-reader acceptance criteria for core learning and certification flows. |
| NFR-019 | Storage use for the built-in application, excluding platform WebView/runtime, should remain below 250 MB unless documented course assets justify more. |
| NFR-020 | An ordinary session with 10,000 events should remain below 50 MB before optional detailed report artifacts. |
| NFR-021 | The simulator shall expose clear compatibility errors rather than silently altering old scenario semantics. |
| NFR-022 | The system shall be maintainable and testable by one primary developer; unnecessary services and native plugins are prohibited. |

---

# 34. Testing and quality strategy

## 34.1 Test layers

| Layer | Purpose |
|---|---|
| Unit | parsers, state transitions, allocation, scoring, normalization |
| Property-based | invariants across commands, jobs, resources, and packs |
| Model/state machine | valid and invalid transition exploration |
| Golden output | supported command and log rendering |
| Deterministic replay | equal seeds/events produce equal state digests |
| Scenario contract | objectives reachable, grading rules valid, references resolved |
| Question-bank validation | answer correctness, ambiguity, scoring bounds |
| Native integration | pure Rust scenario execution and persistence codecs |
| WASM integration | worker messages, IndexedDB, UI state deltas |
| Tauri capability | only approved commands/permissions available |
| Security | no process/network paths, import attacks, CSP, archive limits |
| Accessibility | keyboard, focus, labels, transcript, reduced motion |
| Cross-platform | WebView behavior, file dialogs, persistence, reports |
| Recovery | crash, corrupted snapshot, migration, partial import |
| Performance | actors, jobs, event replay, large transcripts |
| Release smoke | install, launch, complete sample lab, export/import |

## 34.2 Required properties

1. Allocated CPU, memory, and whole GPUs never exceed configured capacity.
2. A GPU cannot be allocated to two whole-GPU jobs simultaneously.
3. Resources are released exactly once on terminal transition.
4. A terminal job never returns to `RUNNING` except through an explicit requeue model.
5. A pending job holds no compute resources unless the modeled feature explicitly says otherwise.
6. Job-local visible GPUs are a deterministic mapping of the allocated set.
7. `sacct` terminal state matches the authoritative job record.
8. Deleting a virtual file cannot affect host files.
9. Imported paths cannot escape the archive or virtual root.
10. Replaying canonical events from a snapshot reproduces the same state digest.
11. Scoring never exceeds item/section maximum and never becomes negative unless the blueprint explicitly permits bounded negative marking.
12. Selecting all options in a multi-select item does not receive accidental full credit.
13. An invalid pack signature cannot be presented as official.
14. Finalized assessment evidence cannot be edited through ordinary application operations.
15. An assessment invalidated by Scenario Control cannot produce a passing certificate.
16. No runtime command path reaches a native process-spawn or network API.

## 34.3 Golden scenarios

Minimum golden scenarios:

```text
basic interactive allocation
one-GPU visibility mapping
CPU/memory unsatisfiable request
resource contention and pending reason
batch output naming
array task output substitution
GPU OOM
host OOM
timeout with checkpoint
cancel and resource release
multi-GPU rank startup
node drain and recovery
QOS limit
assessment pass
assessment fail on critical competency
crash and restore
pack import rejection
```

## 34.4 Scenario validation

The scenario compiler shall check:

- unique IDs;
- valid times and references;
- profile compatibility;
- actor action validity;
- no impossible initial allocations;
- objective references;
- grading maximums;
- hint progression;
- certification critical criteria;
- localization keys;
- asset hashes;
- deterministic compilation.

Where feasible, a bounded solver or simulation sweep should demonstrate at least one successful canonical solution path.

## 34.5 Question validation

Every question requires:

- stable ID and revision;
- competency;
- difficulty;
- prompt;
- answer policy;
- explanation;
- source/author provenance;
- locale;
- no duplicate choices;
- at least one correct answer;
- bounded score;
- test cases for accepted and rejected fill answers.

## 34.6 Security tests

- inspect Tauri capability manifest;
- ensure no shell/process/http plugins;
- static scan application-owned source for process APIs;
- dependency allow/deny list;
- run with network disabled;
- CSP violation capture;
- malicious archive traversal;
- decompression bomb simulation;
- oversized question/asset input;
- malformed CBOR/Postcard fuzzing;
- arbitrary HTML/script injection;
- forged/invalid official signature;
- certification mode bypass attempts.

## 34.7 Fuzzing

Fuzz targets:

- shell lexer/parser;
- `#SBATCH` directive parser;
- size/unit parser;
- time parser;
- job-format parser;
- pack/session decoder;
- virtual path normalizer;
- fill-answer normalizer;
- event replay decoder.

## 34.8 Cross-platform matrix

| Platform | MVP | v1.0 |
|---|---:|---:|
| macOS Apple Silicon latest two major OS releases | required | required |
| Windows 11 x86-64 | smoke | required |
| Ubuntu LTS x86-64 AppImage | smoke | required |
| Intel macOS | deferred | optional |
| Other Linux distributions | deferred | best effort |

## 34.9 Definition of a test-passing release

A release candidate must:

- pass all unit/property/golden/replay tests;
- produce equal canonical digests on target platforms for core scenarios;
- complete all built-in labs offline;
- complete a certification pass and fail scenario;
- survive forced close and restore;
- export/import a session;
- reject malicious packs;
- expose no forbidden Tauri capability;
- generate a complete third-party notice bundle.

---

# 35. Repository and code architecture

## 35.1 Monorepo

```text
dgx-lab/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── README.md
├── LICENSE
├── CONTENT-LICENSE
├── NOTICE
├── TRADEMARKS.md
├── SECURITY.md
├── crates/
│   ├── dgxlab-contracts/
│   ├── sim-core/
│   ├── slurm-model/
│   ├── scheduler/
│   ├── virtual-shell/
│   ├── virtual-fs/
│   ├── workloads/
│   ├── actors/
│   ├── scenarios/
│   ├── grading/
│   ├── assessment/
│   ├── persistence-codec/
│   ├── sim-worker-wasm/
│   ├── web-ui/
│   ├── scenario-compiler/
│   └── report-renderer/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── main.json
│   ├── src/
│   │   ├── main.rs
│   │   ├── file_dialog.rs
│   │   ├── import_export.rs
│   │   └── app_metadata.rs
│   └── icons/
├── course-src/
│   ├── slurm-fundamentals/
│   └── shared/
├── scenario-src/
│   ├── profiles/
│   ├── labs/
│   ├── practice/
│   └── assessments/
├── question-src/
├── localization/
├── schemas/
├── fixtures/
├── tests/
│   ├── golden/
│   ├── replay/
│   ├── security/
│   ├── recovery/
│   ├── accessibility/
│   └── release/
├── docs/
│   ├── architecture/
│   ├── adr/
│   ├── authoring/
│   ├── runbooks/
│   └── prd/
└── .github/workflows/
```

## 35.2 Crate responsibilities

### `dgxlab-contracts`

Versioned shared types, message schemas, IDs, manifests, and compatibility rules.

### `sim-core`

Clock, event queue, deterministic RNG facade, world, replay, and state digest.

### `slurm-model`

Jobs, steps, states, reasons, nodes, partitions, QOS, reservations, TRES, and accounting.

### `scheduler`

Validation, eligibility, priority, resource matching, allocation, transitions, and start estimates.

### `virtual-shell`

Lexer, parser, command AST, environment, redirection, supported command dispatch.

### `virtual-fs`

Virtual paths, file nodes, permissions, quota, content hashes, and snapshots.

### `workloads`

Declarative workload interpretation, telemetry, logs, failure rules, and artifacts.

### `actors`

Scripted/policy/infrastructure actor behavior.

### `scenarios`

Scenario initialization, fault schedules, objectives, hints, and content links.

### `grading`

Evidence ledger and state-based practical grading.

### `assessment`

Question model, answer normalization, blueprint selection, scoring, and finalization.

### `persistence-codec`

Canonical serialization, compression facade, pack/session manifests, migrations, and hashes.

### `sim-worker-wasm`

Web Worker adapter only. It shall not contain domain rules that belong in core crates.

### `web-ui`

Leptos views, accessibility, localization, terminal/editor rendering, IndexedDB coordination.

### `scenario-compiler`

Native authoring tool that validates YAML/Markdown and emits runtime packs.

### `report-renderer`

Deterministic HTML/Markdown/JSON report generation shared where practical.

## 35.3 Dependency policy

- pin Rust toolchain per release branch;
- commit `Cargo.lock`;
- minimize dependencies in runtime crates;
- deny duplicate high-risk codecs where avoidable;
- run license/advisory checks;
- document WebView platform prerequisites;
- no Node runtime required for end users;
- build tooling may use Trunk and supporting tools.

## 35.4 Development standards

| Concern | Standard |
|---|---|
| Formatting | `rustfmt` |
| Linting | Clippy with warnings denied in CI |
| Testing | Rust tests, proptest, wasm-bindgen-test, Tauri/WebDriver where qualified |
| Schemas | Serde + generated JSON Schema where useful |
| Documentation | Markdown and rustdoc |
| ADRs | Markdown |
| Security | cargo-deny/advisory scan plus custom forbidden-API checks |
| Fuzzing | cargo-fuzz or equivalent |
| Build | Tauri CLI + Trunk for Leptos assets |
| Release | per-platform signed pipeline when credentials are available |

---

# 36. Build, packaging, and release

## 36.1 Development build

```text
scenario compiler validates built-in content
        ↓
Trunk builds Leptos UI and WASM worker assets
        ↓
Tauri bundles assets and native shell
        ↓
platform-specific test installation
```

## 36.2 Release channels

- `dev`: unsigned local development;
- `preview`: milestone builds, may be unsigned and clearly labeled;
- `stable`: signed/notarized where applicable, full release gate;
- no automatic update channel in v1.

## 36.3 Signing

Public polished distribution should use:

- Apple code signing and notarization for macOS;
- Windows code signing where feasible;
- checksums and detached signatures for Linux artifacts;
- release provenance/attestation where practical.

Signing affects distribution trust, not certification identity.

## 36.4 Static web build

P1 or later:

- same Leptos UI;
- same worker WASM;
- browser import/export;
- IndexedDB;
- no Tauri APIs;
- static assets only;
- same simulator compatibility tests.

The static build is not allowed to introduce a server dependency merely because it now lives in a browser without a desktop coat.

## 36.5 Release manifest

Every release records:

- semantic version;
- Git commit;
- Rust toolchain;
- Tauri/Leptos versions;
- dependency lock digest;
- simulator compatibility version;
- built-in pack digests;
- session/pack schema versions;
- platform artifact hashes;
- test summary;
- known limitations;
- trademark disclaimer revision.

---

# 37. Observability and diagnostics

DGX Lab has no remote telemetry, but local diagnosability remains necessary.

## 37.1 Local diagnostic categories

- application startup;
- IndexedDB migration;
- worker start/restart;
- scenario validation;
- import/export;
- snapshot/replay;
- rendering performance;
- report generation;
- capability-denial event.

## 37.2 Logging policy

- local only;
- structured where practical;
- no host path beyond user-selected import/export diagnostics unless redacted;
- no virtual learner file contents by default;
- ring-buffer retention;
- exportable diagnostic bundle with explicit user action;
- certification evidence and debug logs remain separate.

## 37.3 Developer diagnostics

Developer diagnostics may show:

- worker message latency;
- event queue length;
- state digest;
- IndexedDB sizes;
- snapshot sequence;
- frame/render timing;
- current profile/pack revisions;
- CSP and capability status.

It shall not expose host shell access.

---

# 38. Success metrics

## 38.1 Product metrics, measured locally or in formal studies

| Metric | MVP target | v1.0 target |
|---|---:|---:|
| Built-in labs | 4 | 12 |
| Core practical competencies | 4 | 12 |
| Certification workflow | prototype | complete |
| Offline course completion | 100% | 100% |
| Real infrastructure calls | 0 | 0 |
| Deterministic golden replay pass | 100% | 100% |
| Critical scenario false-positive grading | 0 in test set | 0 in release test set |
| Crash restore success | ≥ 99% test runs | ≥ 99.9% test runs |
| Session export/import success | 100% fixtures | 100% fixtures |
| Keyboard-only core-flow completion | 100% | 100% |
| Cross-platform canonical state digest parity | macOS native/WASM | all release platforms |

## 38.2 Educational metrics

A pilot should measure:

- pre/post knowledge score;
- practical task completion;
- time to diagnose pending job;
- correct differentiation of GPU versus host OOM;
- correct batch-script construction;
- checkpoint/resume success;
- transfer to a supervised real Slurm environment;
- learner confidence calibrated against actual performance;
- instructor time per learner;
- number of real-cluster support incidents after training.

## 38.3 Recommended validation study

Compare:

```text
written tutorial only
versus
written tutorial + DGX Lab
```

Primary outcome:

- performance on a held-out practical Slurm assessment.

Secondary outcomes:

- time to complete first supervised real job;
- number/severity of resource-request errors;
- diagnostic accuracy;
- retention after 4–8 weeks;
- learner cognitive load and usability.

---

# 39. Roadmap

The roadmap is capability-gated. Calendar ranges assume one primary human developer assisted by coding agents.

## Phase 0: architecture and safety skeleton, weeks 0–3

Deliverables:

- monorepo and Rust toolchain;
- core contracts;
- pure Rust event queue and deterministic RNG;
- Tauri shell with minimal capability manifest;
- Leptos CSR shell;
- simulation worker proof;
- no-network/no-process CI checks;
- initial ADRs;
- basic branded window and disclaimer.

Exit criteria:

- worker responds to typed ping/state request;
- native and WASM deterministic test agree;
- Tauri exposes no forbidden capability;
- offline launch succeeds.

## Phase 1: scheduler walking skeleton, weeks 3–8

Deliverables:

- cluster/node/resource model;
- jobs and steps;
- P0 state machine;
- `sinfo`, `squeue`, `srun`, `salloc`, and `scancel` subset;
- custom terminal;
- cluster visualization;
- event persistence and basic restore;
- `DGX-H200-8` profile.

Exit criteria:

- learner requests an interactive allocation;
- one GPU is allocated and visualized;
- two virtual users contend coherently;
- reset and replay are deterministic.

## Phase 2: files, batch, and workloads, weeks 8–14

Deliverables:

- virtual filesystem;
- integrated editor;
- `sbatch` and `#SBATCH` parsing;
- logs and output paths;
- single-GPU synthetic training;
- GPU OOM, host OOM, timeout, script failure;
- `scontrol` and `sacct` subset;
- first four guided labs;
- hints and evidence engine.

Exit criteria: **MVP**

- four labs complete end to end;
- crash/restore works;
- no real execution/network capability;
- session export/import works;
- learner can diagnose at least three failure classes.

## Phase 3: campaign and recovery, weeks 14–20

Deliverables:

- arrays;
- dependencies;
- checkpoint/resume;
- multi-GPU synthetic workload;
- `sstat` subset;
- richer timeline/telemetry;
- Labs 5–10;
- policy-driven actors;
- degraded scenarios.

Exit criteria:

- array campaign runs coherently;
- checkpoint survives timeout/cancel scenario;
- two-/four-GPU ranks and accounting are consistent;
- ten labs pass content QA.

## Phase 4: certification and v1 content, weeks 20–28

Deliverables:

- question engine;
- assessment blueprints;
- certification lifecycle;
- practical scoring finalization;
- certificate/evidence reports;
- Labs 11–12;
- complete English question bank;
- imported pack format;
- scenario compiler CLI;
- accessibility pass;
- Windows and Linux qualification.

Exit criteria: **v1.0 candidate**

- twelve labs;
- full certification pass/fail paths;
- offline, cross-platform, export/import, recovery, security gates pass;
- public naming/trademark decision documented.

## Phase 5: post-v1 hardening, months 7–10

Deliverables:

- Thai UI/course pack;
- QOS/reservation/fair-share advanced course;
- full rewind and branching;
- official signed packs;
- static web build;
- installer signing automation;
- pilot study and learning analytics through consented exported study data, not hidden telemetry.

## Phase 6: advanced simulation, later

- fictional multi-node cluster;
- NCCL/fabric scenarios;
- preemption and requeue;
- advanced administrator diagnostics;
- local instructor verification tools;
- optional LMS evidence export;
- no multiplayer or real cluster integration without a separate product decision.

---

# 40. MVP acceptance criteria

The MVP is accepted when all P0 criteria below are demonstrated with stored evidence.

## 40.1 Application and security

- [ ] Tauri app launches on macOS Apple Silicon.
- [ ] No localhost server starts.
- [ ] Complete MVP runs with network disabled.
- [ ] Tauri capability manifest contains only approved capabilities.
- [ ] No shell/process/SSH/HTTP runtime dependency or API is present.
- [ ] CSP denies remote resources and connections.
- [ ] Imported malformed session files fail safely.
- [ ] Application-owned runtime crates forbid unsafe code.
- [ ] About page displays independent-simulator disclaimer.

## 40.2 Simulation

- [ ] Native and WASM core produce the same canonical digest for golden scenarios.
- [ ] Equal seed and command transcript reproduce equal jobs, allocations, and grading.
- [ ] Pause, step, real-time, ×10, and ×60 work.
- [ ] Simulation worker remains responsive under 12 actors and 100 jobs.
- [ ] Resource capacity invariants pass property tests.
- [ ] Reset-to-start reproduces initial world exactly.

## 40.3 Scheduler and terminal

- [ ] `sinfo`, `squeue`, `srun`, `salloc`, `scancel`, `scontrol show job`, `scontrol show node`, `sbatch`, and `sacct` required subsets work.
- [ ] One-GPU job sees one job-local GPU.
- [ ] Two simultaneous one-GPU jobs receive distinct physical virtual GPUs.
- [ ] Unsatisfiable GPU, CPU, and memory requests are rejected or held according to scenario policy.
- [ ] Pending reason updates after resources become available.
- [ ] Job/step resources release exactly once.
- [ ] Unsupported commands cannot reach the host.

## 40.4 Files and workloads

- [ ] Learner creates and edits a virtual batch script.
- [ ] `#SBATCH` directives produce a valid job request.
- [ ] Default and explicit output files are created virtually.
- [ ] Single-GPU workload emits deterministic logs and metrics.
- [ ] GPU OOM, host OOM, timeout, cancellation, and script failure are distinguishable.
- [ ] No virtual path accesses host storage.

## 40.5 Learning

- [ ] Four guided labs are complete.
- [ ] State-based evidence records successful and failed attempts.
- [ ] Progressive hints work and are recorded.
- [ ] Valid alternative commands receive credit in test scenarios.
- [ ] Free-play sandbox uses the same simulator core.
- [ ] Command transcript and competency summary render.

## 40.6 Persistence

- [ ] Sessions autosave after commands.
- [ ] Forced application close restores the latest valid state.
- [ ] Corrupt latest snapshot falls back safely or produces recovery report.
- [ ] `.dgxlab` export/import round-trips a session.
- [ ] Storage reset does not alter the installed application bundle.

---

# 41. v1.0 acceptance criteria

In addition to MVP acceptance:

## 41.1 Cross-platform

- [ ] macOS Apple Silicon, Windows x86-64, and Linux x86-64 packages install and launch.
- [ ] Core golden scenarios produce equal canonical digests on all target platforms.
- [ ] Native file dialogs and session import/export work on all targets.
- [ ] IndexedDB persistence and crash recovery work on all targets.

## 41.2 Complete curriculum

- [ ] Twelve labs cover competencies C1–C12.
- [ ] Arrays, dependencies, checkpoint/resume, and multi-GPU workloads work.
- [ ] Advanced concurrent actors produce reproducible contention.
- [ ] At least one node/storage/container failure scenario is complete.
- [ ] All lessons pass content, accessibility, and technical review.

## 41.3 Certification

- [ ] Multiple-choice, multi-select, and fill-in-the-blank items work.
- [ ] Assessment selection is deterministic from a recorded seed.
- [ ] Practical assessment contributes 60%, MCQ/multi-select 25%, and fill-in 15% under the default blueprint.
- [ ] 80% overall and 70% knowledge pass rules are enforced.
- [ ] Missing critical practical competency blocks certification.
- [ ] Scenario Control invalidates an assessment.
- [ ] Assisted attempts are labeled.
- [ ] Pass and fail certificates/reports generate correctly.
- [ ] Evidence digest recalculates after import.
- [ ] Finalized assessment cannot be mutated through ordinary UI.

## 41.4 Packs

- [ ] Built-in content is compiled and validated.
- [ ] `.dgxlabpack` imports after schema/hash/size/path validation.
- [ ] Unsigned local pack is clearly labeled.
- [ ] Invalidly signed pack is rejected/quarantined.
- [ ] Pack cannot include executable code.
- [ ] Scenario compiler reproduces identical pack digest from identical canonical input/toolchain conditions.

## 41.5 Accessibility

- [ ] Core course and certification can be completed keyboard-only.
- [ ] Terminal transcript mode is screen-reader usable.
- [ ] Cluster view has equivalent table/text representation.
- [ ] Reduced-motion mode removes nonessential animation.
- [ ] 200% text scaling preserves core functionality.

## 41.6 Release governance

- [ ] Licenses and third-party notices are complete.
- [ ] No NVIDIA logo or copied brand styling is present.
- [ ] Public naming/trademark review has a recorded decision.
- [ ] Stable installers are signed/notarized where release policy requires.
- [ ] Release manifest and checksums are published.

---

# 42. Risks and mitigations

| Risk | Probability | Impact | Mitigation |
|---|---:|---:|---|
| Simulator subtly teaches incorrect Slurm behavior | Medium | High | explicit subset/versioning, official-doc review, golden cases, site-expert review |
| Scope expands into a full Unix/Slurm clone | High | High | curriculum-driven support manifest, strict non-goals, P0/P1/P2 gates |
| Custom terminal/editor consumes excessive effort | Medium | Medium | constrained feature set, accessibility-first transcript, switch UI component only if prototype fails |
| Leptos/Tauri WebView differences cause platform bugs | Medium | High | early Windows/Linux smoke tests, worker/persistence fixtures, target-platform CI |
| Web Worker behavior differs across WebViews | Low/Medium | High | early spike, bounded fallback design if needed, release smoke tests |
| IndexedDB implementation differs across platforms | Medium | High | persistence adapter, transaction tests, export safety, corruption recovery |
| Large event logs become slow or bloated | Medium | Medium | snapshots, compact codec, coalesced metrics, content-addressed files |
| Determinism breaks through floating point or unordered data | Medium | High | fixed-point/integer core, ordered maps, canonical serialization, cross-platform digests |
| Imported pack parser becomes attack surface | Medium | High | data-only format, strict limits, fuzzing, quarantine, no executable content |
| Tauri permissions gradually expand | Medium | Critical | capability diff gate, denylist, ADR for every new native permission |
| A future developer adds a real backend | Low/Medium | Critical | architectural prohibition, repository tests, no generic backend trait |
| Certification overclaims identity or integrity | High | High | trust-level labels, replayable evidence, no proctored claim |
| Question bank rewards memorization | Medium | Medium | blueprints, practical majority weight, parameterized scenarios, competency review |
| Fill-in scoring rejects valid answers | Medium | Medium | accepted aliases, item test cases, review tooling, appeal/report evidence |
| Practical grading yields false positives | Medium | High | state-based critical checks, adversarial solution tests, evidence review |
| Practical grading yields false negatives | Medium | High | alternative paths, canonicalization, pilot testing, item-level diagnostics |
| Public use of “DGX Lab” creates trademark issue | Medium | High | legal review, original branding, disclaimer, centralized rename capability |
| Open-source packs contain poor or misleading content | High | Medium | trust states, official signatures, schema validation, clear authorship/license |
| Solo developer is overwhelmed | High | High | walking skeleton, four-lab MVP, pure core, generated schemas/tests, defer administration/multiplayer |
| Product becomes visually impressive but educationally weak | Medium | High | competency-first design, practical evidence, pilot study, no vanity animation |
| No telemetry limits product insight | Certain | Low/Medium | consented pilot exports, local reports, explicit study workflows rather than covert analytics |
| Learners transfer simulator-specific output too literally | Medium | Medium | compatibility notices, concepts before formatting, compare-site callouts |
| Certificate PDF varies across platforms | Medium | Medium | HTML canonical report, qualified print pipeline, digest evidence independent of pixels |

---

# 43. Open implementation decisions requiring ADRs

These decisions do not reopen the approved product direction.

1. Exact pinned Tauri 2 and Leptos revisions for bootstrap.
2. Trunk and wasm-bindgen build integration details.
3. Worker initialization and message transport strategy in each target WebView.
4. Postcard versus CBOR for worker messages and canonical records.
5. Zstandard versus deflate/miniz compression for session and pack data.
6. IndexedDB abstraction crate versus small direct `web-sys` wrapper.
7. Canonical state-digest serialization.
8. Deterministic pseudo-random generator algorithm.
9. Exact H200-like simulated HBM value and profile disclaimer.
10. Terminal rendering strategy and ANSI subset.
11. Virtual editor component implementation.
12. Report-to-PDF implementation across platforms.
13. Official pack signature library and key-rotation process.
14. Static scenario reachability checker design.
15. Multi-select partial-credit default.
16. Fill-in accepted-pattern representation without unsafe regex complexity.
17. Accessibility testing stack for Tauri WebViews.
18. Windows installer type and Linux primary package format.
19. Code-signing/notarization custody and CI process.
20. Public release name after trademark review.

---

# 44. Explicitly deferred features

- real SLURM connection;
- SSH or cluster account integration;
- arbitrary shell or Python execution;
- real containers;
- multiplayer classrooms;
- teacher dashboard served over a network;
- cloud accounts and progress sync;
- runtime telemetry;
- automatic updater;
- LMS/LTI integration;
- SCORM export;
- institutionally verified identity/proctoring;
- AI tutor;
- course-authoring GUI;
- executable plugins;
- MIG;
- GPU shards;
- MPS;
- full administrator configuration editor;
- real hardware monitoring;
- mobile/tablet application;
- collaborative shared simulations;
- public pack marketplace;
- automatic translation of course content;
- multi-node simulation before P2.

---

# 45. Definition of done

A DGX Lab feature is done only when it includes:

- implementation in the correct crate/layer;
- typed contracts;
- unit tests;
- property or state-machine tests where applicable;
- native and WASM consideration;
- deterministic replay consideration;
- persistence/migration behavior;
- accessibility behavior;
- localization keys;
- security/capability review;
- failure and recovery behavior;
- documentation;
- course/assessment impact;
- acceptance evidence;
- no violation of the no-real-infrastructure invariant.

A lesson is done only when it includes:

- competency mapping;
- concept content;
- scenario;
- canonical solution;
- valid alternative solution;
- common misconception path;
- progressive hints;
- grading rules;
- accessibility review;
- question items where applicable;
- technical review against the simulator;
- license and attribution metadata.

---

# Appendix A. Example worker protocol

```rust
#[derive(Serialize, Deserialize)]
pub enum SimRequest {
    Initialize {
        profile_digest: String,
        scenario_digest: String,
        seed: u64,
    },
    ExecuteCommand {
        session_id: SessionId,
        command: String,
        expected_event_seq: u64,
    },
    SaveVirtualFile {
        path: VirtualPath,
        content: String,
        expected_version: u64,
    },
    AdvanceClock {
        mode: ClockAdvance,
    },
    RequestSnapshot,
    ResetScenario,
    SubmitKnowledgeAnswer {
        assessment_id: String,
        item_id: String,
        answer: LearnerAnswer,
    },
    FinalizeAssessment {
        assessment_id: String,
    },
}

#[derive(Serialize, Deserialize)]
pub enum SimResponse {
    Initialized {
        state: PublicWorldState,
        event_seq: u64,
    },
    TerminalOutput {
        lines: Vec<TerminalLine>,
        event_seq: u64,
    },
    StateDelta {
        delta: WorldDelta,
        from_seq: u64,
        to_seq: u64,
    },
    FullState {
        state: PublicWorldState,
        event_seq: u64,
    },
    EvidenceUpdate {
        update: EvidenceDelta,
    },
    AssessmentUpdate {
        update: AssessmentDelta,
    },
    Persist {
        batch: PersistenceBatch,
    },
    Error {
        error: SimError,
        recoverable: bool,
    },
}
```

The message transport is not the authoritative event store. It is a view/update channel over the worker's simulation state.

---

# Appendix B. Example scenario

```yaml
schema: dgxlab.scenario/v1
id: pending-gpu-contention-01
revision: 1.0.0
title: Diagnose a Pending GPU Job
cluster_profile: dgx-h200-8
seed_policy:
  mode: blueprint_seed

learner:
  username: learner
  account: research
  qos: normal

initial_files:
  - path: /home/learner/train.sbatch
    content: |
      #!/bin/bash
      #SBATCH --job-name=my-train
      #SBATCH --partition=gpu
      #SBATCH --gres=gpu:h200:1
      #SBATCH --cpus-per-task=8
      #SBATCH --mem=64G
      #SBATCH --time=00:30:00
      #SBATCH --output=logs/%x-%j.out

      module load singularity
      singularity exec --nv /containers/pytorch-lab.sif \
        python train.py --batch-size 64 --epochs 5

actors:
  - id: alice
    kind: scripted
    username: alice
    actions:
      - at: 00:00:00
        submit_job:
          name: vision-train
          partition: gpu
          gpus: 4
          cpus: 32
          memory_gib: 256
          workload: pytorch-vision-v1
          duration: 00:45:00

  - id: bob
    kind: scripted
    username: bob
    actions:
      - at: 00:00:00
        submit_job:
          name: language-train
          partition: gpu
          gpus: 4
          cpus: 32
          memory_gib: 512
          workload: pytorch-language-v1
          duration: 00:25:00

objectives:
  - id: submit-one-gpu
    competency: C7
    description: Submit the prepared job.
  - id: diagnose-pending
    competency: C7
    description: Determine why the job remains pending.
  - id: observe-start
    competency: C7
    description: Confirm that the job starts when resources become available.

hints:
  - trigger:
      no_progress_events: 3
    level: 1
    text: Inspect the queue and pay attention to the reason column.
  - trigger:
      command_error_count: 2
    level: 2
    text: `squeue` and `scontrol show job` expose pending-state evidence.

checks:
  - id: submitted
    critical: true
    points: 20
    assert:
      learner_job_exists:
        gpus: 1
  - id: inspected-reason
    points: 20
    assert:
      any_command_used:
        - "squeue"
        - "scontrol show job"
  - id: diagnosed-resources
    critical: true
    points: 30
    assert:
      learner_diagnosis: resources
  - id: eventually-running
    points: 30
    assert:
      learner_job_visited_state: RUNNING
```

---

# Appendix C. Example knowledge questions

## C.1 Single-answer multiple choice

```yaml
schema: dgxlab.question/v1
id: q-pending-resources-01
revision: 1.0.0
competency: C7
difficulty: foundational
type: single_choice
prompt: |
  Your job is in state `PD` with reason `(Resources)`. What does this most directly mean?
options:
  - id: a
    text: The batch script contains invalid Bash syntax.
  - id: b
    text: The requested resources are currently unavailable.
  - id: c
    text: The job has completed but accounting has not updated.
  - id: d
    text: The login node cannot reach the internet.
answer:
  correct: [b]
explanation: |
  `PD` means pending. `(Resources)` indicates that required schedulable resources
  are not currently available to start the job.
```

## C.2 Multi-select

```yaml
id: q-gpu-oom-actions-01
revision: 1.0.0
competency: C9
type: multi_select
prompt: Which changes may reduce GPU-memory demand for this workload?
options:
  - id: a
    text: Reduce the per-device batch size.
  - id: b
    text: Use gradient accumulation to preserve effective batch size.
  - id: c
    text: Increase `--mem` while changing nothing else.
  - id: d
    text: Enable an approved memory-saving training configuration.
answer:
  correct: [a, b, d]
scoring:
  mode: bounded_partial
  incorrect_penalty: 0.25
  minimum: 0
```

## C.3 Fill in the blank

```yaml
id: q-request-gpu-01
revision: 1.0.0
competency: C4
type: fill_blank
prompt: |
  Complete the command:
  `srun --gres=__________:1 --pty bash`
blanks:
  - id: blank-1
    normalization:
      trim: true
      case_insensitive: true
      normalize_whitespace: true
    accepted:
      - literal: gpu:h200
      - literal: gpu
        when_profile_allows_generic_gpu: true
explanation: |
  A typed request uses `gpu:h200:1`; some profiles also permit a generic `gpu:1` request.
```

---

# Appendix D. Initial certification blueprint

```yaml
schema: dgxlab.assessment-blueprint/v1
id: slurm-user-foundations-cert-v1
revision: 1.0.0
title: DGX Lab — SLURM User Foundations
attempts_allowed: 2
pass_policy:
  overall_percent: 80
  knowledge_percent: 70
  require_all_critical_practical: true
weights:
  practical: 60
  multiple_choice: 25
  fill_blank: 15
sections:
  - id: knowledge-a
    type: question_pool
    competencies: [C1, C2, C3, C4, C5, C6]
    item_count: 15
  - id: knowledge-b
    type: question_pool
    competencies: [C7, C8, C9, C10, C11, C12]
    item_count: 15
  - id: practical-1
    type: scenario
    scenario_pool: [cert-batch-job-a, cert-batch-job-b]
    critical_competencies: [C3, C4, C6]
  - id: practical-2
    type: scenario
    scenario_pool: [cert-diagnose-a, cert-diagnose-b]
    critical_competencies: [C7, C9]
  - id: practical-3
    type: scenario
    scenario_pool: [cert-resume-a, cert-resume-b]
    critical_competencies: [C10, C12]
policy:
  hints: disabled
  scenario_control: invalidates
  clock_control: scenario_defined
  reference_sheet: basic-command-summary
```

---

# Appendix E. Example session manifest

```yaml
schema: dgxlab.session/v1
format_version: 1.0.0
session_id: 0190-example
origin:
  app_version: 1.0.0
  simulator_compatibility: sim-v1
course:
  id: slurm-fundamentals
  revision: 1.0.0
scenario:
  id: pending-gpu-contention-01
  revision: 1.0.0
  digest: sha256:...
seed: 20260805
state:
  last_event_sequence: 482
  snapshot_sequence: 450
  finalized_assessment: false
contents:
  events: events.cbor.zst
  snapshots: snapshots/
  virtual_files: virtual-files/
hashes:
  algorithm: sha256
  manifest: sha256:...
```

---

# Appendix F. Command-coverage matrix for v1

| Command | MVP | v1.0 | Notes |
|---|---:|---:|---|
| `sinfo` | yes | yes | common filters/formats only |
| `squeue` | yes | yes | reasons, user, partition, job filters |
| `sbatch` | yes | yes | supported directives/options |
| `srun` | yes | yes | allocation and job-step subset |
| `salloc` | yes | yes | interactive allocation subset |
| `scancel` | yes | yes | own jobs and array tasks |
| `scontrol show job` | yes | yes | detailed state |
| `scontrol show node` | yes | yes | resource/state detail |
| `scontrol show partition` | no | yes | advanced profile |
| `scontrol show reservation` | no | yes | advanced profile |
| `sacct` | yes | yes | jobs/steps, selected fields |
| `sstat` | no | yes | live selected metrics |
| `sprio` | no | yes | simplified factors |
| `squeue --start` | no | yes | deterministic estimate |
| `sacctmgr show` | no | limited | read-only advanced lessons |
| `module` | yes | yes | simulated environment |
| `singularity exec` | yes | yes | simulated only |
| `python` | registered | registered | synthetic workloads only |
| `torchrun` | no | yes | synthetic multi-GPU only |
| `nvidia-smi` | yes | yes | simulated allocated devices |

---

# Appendix G. Source mapping

| PRD area | Source basis |
|---|---|
| Generic eight-H200 profile, CPU/memory, cgroup isolation | ORCA UAT Report Rev 1.1 [@orca2026UAT] |
| Slurm 25.05, Singularity, accounting, paths generalized from production | ORCA Installation Report Rev 1.0 [@orca2026Installation] |
| GPU/scheduler/monitoring concepts used in visual scenarios | ORCA Monitoring Guide Rev 1.0 [@orca2026Monitoring] |
| Operational failure inspiration such as container/storage/GPU diagnosis | ORCA Operations Runbook Rev 1.0 [@orca2026Runbook] |
| Tauri capability and CSP boundary | Tauri 2 official documentation [@tauri2026Capabilities; @tauri2026CSP] |
| Native file dialogs | Tauri dialog plugin documentation [@tauri2026Dialog] |
| Tauri + Leptos frontend configuration | Tauri Leptos guide [@tauri2026Leptos] |
| Leptos client-side WASM model | Leptos official book [@leptos2026CSR] |
| Job states | SchedMD Slurm documentation [@slurmJobStates] |
| Batch-script behavior | SchedMD `sbatch` documentation [@slurmSbatch] |
| Pending reason presentation | SchedMD `squeue` documentation [@slurmSqueue] |
| GPU GRES and visibility | SchedMD GRES and prolog/epilog documentation [@slurmGRES; @slurmPrologEpilog] |
| QOS and limit concepts | SchedMD QOS/resource-limit documentation [@slurmQOS; @slurmResourceLimits] |
| Brand and trademark caution | NVIDIA official brand/trademark material [@nvidia2026Brand; @nvidiaTrademarks] |

---

# References

[@orca2026UAT] Faculty of Medicine Siriraj Hospital. **ORCA HPC AI Cluster User Acceptance Test Report, Rev 1.1.** 12 July 2026. Internal source.

[@orca2026Installation] Faculty of Medicine Siriraj Hospital. **ORCA HPC AI Cluster Installation Report (As-Built), Rev 1.0.** 13 July 2026. Internal source.

[@orca2026Monitoring] Faculty of Medicine Siriraj Hospital. **ORCA Monitoring Guide, Rev 1.0.** 13 July 2026. Internal source.

[@orca2026Runbook] Faculty of Medicine Siriraj Hospital. **ORCA Operations Runbook, Rev 1.0.** 13 July 2026. Internal source.

[@tauri2026Capabilities] Tauri Contributors. **Tauri 2: Capabilities.** Official documentation, accessed 5 August 2026.

[@tauri2026CSP] Tauri Contributors. **Tauri 2: Content Security Policy.** Official documentation, accessed 5 August 2026.

[@tauri2026Dialog] Tauri Contributors. **Tauri 2 Dialog Plugin.** Official documentation, accessed 5 August 2026.

[@tauri2026Leptos] Tauri Contributors. **Tauri 2 Frontend Configuration: Leptos.** Official documentation, accessed 5 August 2026.

[@leptos2026CSR] Leptos Project. **Client-Side Rendering and Deployment.** Official Leptos Book, accessed 5 August 2026.

[@slurmJobStates] SchedMD. **Slurm Workload Manager: Job State Codes.** Official documentation, accessed 5 August 2026.

[@slurmSbatch] SchedMD. **Slurm Workload Manager: sbatch.** Official documentation, accessed 5 August 2026.

[@slurmSqueue] SchedMD. **Slurm Workload Manager: squeue and Job Reason Codes.** Official documentation, accessed 5 August 2026.

[@slurmGRES] SchedMD. **Slurm Workload Manager: Generic Resource Scheduling.** Official documentation, accessed 5 August 2026.

[@slurmPrologEpilog] SchedMD. **Slurm Workload Manager: Prolog and Epilog Guide.** Official documentation, accessed 5 August 2026.

[@slurmQOS] SchedMD. **Slurm Workload Manager: Quality of Service.** Official documentation, accessed 5 August 2026.

[@slurmResourceLimits] SchedMD. **Slurm Workload Manager: Resource Limits.** Official documentation, accessed 5 August 2026.

[@nvidia2026Brand] NVIDIA Corporation. **NVIDIA Logo and Brand Guidelines.** Official brand guidance, accessed 5 August 2026.

[@nvidiaTrademarks] NVIDIA Corporation. **Trademark Notices.** Official documentation, accessed 5 August 2026.

---

# Final product definition

**DGX Lab** is a standalone Tauri desktop application that places a deterministic Rust/WASM simulation of a shared GPU cluster inside a safe, offline learning environment. It teaches SLURM through coherent state, realistic contention, synthetic AI workloads, observable failure, guided practice, and practical certification evidence.

Its defining boundary is as important as its feature set:

```text
realistic scheduler semantics
+ realistic consequences
+ simulated concurrent users
+ guided learning and certification

without

real shell
real SLURM
real SSH
real GPU
real cluster credentials
or network dependence
```

The product succeeds when a learner can make costly cluster mistakes repeatedly, understand exactly why they failed, demonstrate recovery, and arrive at a real DGX environment already knowing how not to turn an eight-GPU system into a very expensive group misunderstanding.
