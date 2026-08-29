"use strict";

const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => [...document.querySelectorAll(selector)];
const escapeHtml = (value) => String(value).replace(/[&<>"']/g, ch => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[ch]));

const DEFAULT_SCRIPT = `#!/bin/bash
#SBATCH --job-name=train-h200
#SBATCH --partition=gpu
#SBATCH --gres=gpu:h200:1
#SBATCH --cpus-per-task=8
#SBATCH --mem=64G
#SBATCH --time=00:30:00
#SBATCH --output=logs/%x-%j.out

module load singularity/4.5.0
srun singularity exec --nv /containers/pytorch-lab.sif \\
  python train.py --batch-size 64 --epochs 5`;

const steps = [
  ["Inspect the cluster state", "sinfo"],
  ["Allocate one GPU", "allocation"],
  ["Verify the allocation environment", "env"],
  ["Inspect GPU visibility", "gpu"],
  ["Release resources cleanly", "released"]
];

let state;
function newState(scenario="dgx-h200-8") {
  const virtual = scenario === "dgx-contended" ? [
    makeJob(56790,"alice",4,"RUNNING",null,0,150),
    makeJob(56791,"bob",4,"RUNNING",null,0,105)
  ] : scenario === "dgx-degraded" ? [
    {...makeJob(20418,"learner",4,"OUT_OF_MEMORY",null,0,0), name:"train-llm", elapsed:88},
    {...makeJob(20392,"learner",4,"COMPLETED",null,0,0), name:"train-llm", elapsed:120}
  ] : [];
  return {
    scenario, now:0, speed:1, paused:false, nextJobId:10000, jobs:virtual,
    activeJob:null, selectedJob:null, terminal:[], events:[],
    evidence:{sinfo:false,allocation:false,env:false,gpu:false,released:false},
    hintLevel:0, script:localStorage.getItem("dgxlab-prototype-script") || DEFAULT_SCRIPT
  };
}
function makeJob(id,user,gpus,status,reason,start=0,duration=60){ return {id,user,gpus,status,reason,name:"train",cpus:gpus*8,memory:gpus*64,start,duration,elapsed:0}; }
function reset(scenario=state?.scenario || "dgx-h200-8") {
  state = newState(scenario); addTerm("DGX Lab prototype ready. Type help for supported simulated commands.","info");
  if (scenario === "dgx-degraded") addEvent("Job 20418 ended OUT_OF_MEMORY; checkpoint evidence is available.");
  render();
}

function addTerm(text,kind="stdout") { state.terminal.push({text,kind}); if(state.terminal.length>180)state.terminal.shift(); }
function addCommand(text) { addTerm(`${promptText()} ${text}`,"command"); }
function addEvent(text) { state.events.unshift({at:state.now,text}); if(state.events.length>30)state.events.pop(); }
function promptText(){ return state.activeJob ? "learner@dgx-h200-01:~$" : "learner@dgx-login-01:~$"; }
function formatTime(seconds){ const s=Math.max(0,Math.floor(seconds)); return [Math.floor(s/3600),Math.floor(s/60)%60,s%60].map(v=>String(v).padStart(2,"0")).join(":"); }
function parseNumberOption(command,name,defaultValue){ const re=new RegExp(`(?:${name}=|${name}\\s+)([^\\s]+)`);const m=command.match(re);return m?m[1]:defaultValue; }
function parseGpus(command){ const m=command.match(/--(?:gres=)?gpus?(?:=|\s+)(?:gpu(?::h200)?[:=]?)?(\d+)/i)||command.match(/--gres=gpu(?::h200)?:([0-9]+)/i); return m?Number(m[1]):command.includes("--gres=gpu")?1:0; }
function freeGpuIndices(){ const used=new Set(); for(const job of state.jobs.filter(j=>j.status==="RUNNING")) for(const i of (job.gpuIndices||[])) used.add(i); return Array.from({length:8},(_,i)=>i).filter(i=>!used.has(i)); }
function assign(job){ const free=freeGpuIndices(); if(free.length<job.gpus){ job.status="PENDING";job.reason="Resources";addEvent(`Job ${job.id} pending: Resources`);return false; } job.gpuIndices=free.slice(0,job.gpus);job.status="RUNNING";job.reason=null;job.start=state.now;addEvent(`Job ${job.id} started on dgx-h200-01`);return true; }
function schedulePending(){ for(const job of state.jobs.filter(j=>j.status==="PENDING").sort((a,b)=>a.id-b.id)) assign(job); }
function submitJob({name="train",user="learner",gpus=1,cpus=8,memory=64,duration=75,interactive=false}){
  const job=makeJob(state.nextJobId++,user,gpus,"PENDING","Priority",state.now,duration);Object.assign(job,{name,cpus,memory,interactive});state.jobs.push(job);state.selectedJob=job.id;addEvent(`Job ${job.id} submitted by ${user}`);assign(job);if(interactive&&job.status==="RUNNING"){state.activeJob=job.id;state.evidence.allocation=true;addTerm(`srun: Granted job allocation ${job.id}`,"info");}render();return job;
}
function finishJob(job,status="COMPLETED"){
  if(!job||["COMPLETED","FAILED","CANCELLED","TIMEOUT","OUT_OF_MEMORY"].includes(job.status))return;
  job.status=status;job.reason=null;job.elapsed=state.now-job.start;addEvent(`Job ${job.id} ${status}`);if(state.activeJob===job.id)state.activeJob=null;schedulePending();
}
function tick(){ if(state.paused)return; state.now+=state.speed; for(const job of state.jobs.filter(j=>j.status==="RUNNING"&&!j.interactive)){job.elapsed=state.now-job.start;if(job.elapsed>=job.duration)finishJob(job,"COMPLETED");} renderLight(); }
setInterval(tick,1000);

function runCommand(raw){ const command=raw.trim();if(!command)return;addCommand(command);const tokens=command.split(/\s+/);const cmd=tokens[0];
  if(cmd==="help") addTerm("Supported: sinfo squeue srun salloc sbatch scontrol sacct scancel nvidia-smi echo env exit ls cat module singularity python torchrun clear","info");
  else if(cmd==="clear"){state.terminal=[];}
  else if(cmd==="sinfo"){state.evidence.sinfo=true;addTerm("PARTITION AVAIL TIMELIMIT NODES STATE NODELIST\ngpu*     up    02:00:00     1 " + (freeGpuIndices().length===8?"idle":"mix ") + " dgx-h200-01");}
  else if(cmd==="squeue"){addTerm(queueText());}
  else if(cmd==="srun"||cmd==="salloc"){
    if(state.activeJob){addTerm("srun: a simulated interactive allocation is already active","error");}
    else { const gpus=parseGpus(command);const cpus=Number(parseNumberOption(command,"--cpus-per-task",4));const mem=String(parseNumberOption(command,"--mem","16G"));const memory=Number(mem.replace(/[^0-9.]/g,""))||16; const job=submitJob({name:"interactive",gpus,cpus,memory,duration:1800,interactive:command.includes("--pty")||cmd==="salloc"}); if(job.status==="PENDING")addTerm(`srun: job ${job.id} queued and waiting for resources`,"info"); }
  }
  else if(cmd==="echo"){
    if(command.includes("$SLURM_JOB_ID")) {state.evidence.env=true;addTerm(state.activeJob?String(state.activeJob):"");}
    else if(command.includes("$CUDA_VISIBLE_DEVICES")){state.evidence.env=true;const j=state.jobs.find(x=>x.id===state.activeJob);addTerm(j?j.gpuIndices.map((_,i)=>i).join(","):"");}
    else addTerm(command.slice(5));
  }
  else if(cmd==="env"){state.evidence.env=true;const j=state.jobs.find(x=>x.id===state.activeJob);addTerm(`USER=learner\nSLURM_JOB_ID=${state.activeJob||""}\nCUDA_VISIBLE_DEVICES=${j?j.gpuIndices.map((_,i)=>i).join(","):""}`);}
  else if(cmd==="nvidia-smi"){
    if(!state.activeJob){addTerm("NVIDIA-SMI has no visible devices on the simulated login node.","error");}
    else {state.evidence.gpu=true;const j=state.jobs.find(x=>x.id===state.activeJob); if(command.includes("-L")) addTerm(j.gpuIndices.map((_,i)=>`GPU ${i}: H200 (UUID: SIM-GPU-${String(j.gpuIndices[i]).padStart(4,"0")})`).join("\n"));else addTerm(`Simulated H200 utilization: 84% · memory 61 GiB / 141 GiB`);}
  }
  else if(cmd==="exit"){
    if(!state.activeJob)addTerm("logout"); else {const j=state.jobs.find(x=>x.id===state.activeJob);finishJob(j,"COMPLETED");state.evidence.released=true;addTerm("logout\nsrun: Relinquishing job allocation", "info");}
  }
  else if(cmd==="sbatch"){
    const job=submitFromScript();addTerm(`Submitted batch job ${job.id}`,"info");
  }
  else if(cmd==="scontrol"&&tokens[1]==="show"&&tokens[2]==="job"){
    const id=Number(tokens[3]);const j=state.jobs.find(x=>x.id===id);addTerm(j?jobControlText(j):`slurm_load_jobs error: Invalid job id specified`,j?"stdout":"error");
  }
  else if(cmd==="scontrol"&&tokens[1]==="show"&&tokens[2]==="node") addTerm(`NodeName=dgx-h200-01 State=MIXED CPUs=224 RealMemory=1857528 Gres=gpu:h200:8 AllocGRES=${8-freeGpuIndices().length}`);
  else if(cmd==="sacct") addTerm(accountingText());
  else if(cmd==="scancel") {const id=Number(tokens[1]);const j=state.jobs.find(x=>x.id===id);if(!j)addTerm("scancel: Invalid job id specified","error");else{finishJob(j,"CANCELLED");addTerm(`Job ${id} cancelled`,"info");}}
  else if(cmd==="ls") addTerm(command.includes("checkpoints")?"epoch-001.pt  epoch-002.pt  epoch-003.pt":"README.txt  checkpoints  logs  train.sbatch");
  else if(cmd==="cat") addTerm(command.includes("train.sbatch")?state.script:"This is a simulated file.");
  else if(cmd==="module") addTerm(command.includes("load")?"Loaded simulated module(s).":"Currently Loaded Modules: singularity/4.5.0");
  else if(["singularity","python","python3","torchrun"].includes(cmd)) addTerm("Synthetic workload command accepted inside the simulator. No host process was created.","info");
  else addTerm(`dgxlab: unsupported command '${escapeHtml(cmd)}'. Type help for the simulated command set.`,"error");
  render();
}
function queueText(){let out="JOBID USER     STATE          GPUS REASON";for(const j of state.jobs)out+=`\n${j.id} ${j.user.padEnd(8)} ${j.status.padEnd(14)} ${String(j.gpus).padEnd(4)} ${j.reason||"-"}`;return out;}
function accountingText(){let out="JobID JobName       User     State          Elapsed GPUs";for(const j of state.jobs)out+=`\n${j.id} ${j.name.padEnd(13)} ${j.user.padEnd(8)} ${j.status.padEnd(14)} ${formatTime(j.elapsed||0)} ${j.gpus}`;return out;}
function jobControlText(j){return `JobId=${j.id} JobName=${j.name} UserId=${j.user}\n   JobState=${j.status} Reason=${j.reason||"None"}\n   NumCPUs=${j.cpus} MinMemoryNode=${j.memory}G TresPerNode=gres/gpu:h200:${j.gpus}\n   NodeList=${j.status==="RUNNING"?"dgx-h200-01":"(null)"}`;}
function parseScript(){const text=state.script;const get=(re,def)=>text.match(re)?.[1]||def;return {name:get(/^#SBATCH\s+--job-name=(\S+)/m,"train"),gpus:Number(get(/^#SBATCH\s+--gres=gpu(?::h200)?:([0-9]+)/m,"1")),cpus:Number(get(/^#SBATCH\s+--cpus-per-task=([0-9]+)/m,"4")),memory:Number(get(/^#SBATCH\s+--mem=([0-9]+)/m,"32")),duration:75};}
function submitFromScript(){const spec=parseScript();return submitJob({...spec,user:"learner",interactive:false});}

function render(){renderLight();renderTerminal();renderProgress();renderExamMap();}
function renderLight(){
  $("#scenario-select").value=state.scenario;$("#footer-scenario").textContent=state.scenario.toUpperCase();$("#sim-time").textContent=formatTime(state.now);$("#pause-btn").textContent=state.paused?"Resume":"Pause";$("#prompt").textContent=promptText();
  const gpuOwners=Array(8).fill(null);const warning=state.scenario==="dgx-degraded"?2:null;for(const j of state.jobs.filter(x=>x.status==="RUNNING"))for(const idx of j.gpuIndices||[])gpuOwners[idx]=j;
  $("#gpu-grid").innerHTML=gpuOwners.map((j,i)=>`<div class="gpu-card ${i===warning?"warning":j?(j.user==="learner"?"learner":"virtual"):"idle"}"><strong>GPU ${i}</strong><span>H200 · ${i===warning?"Warning":j?"Allocated":"Idle"}</span><span class="gpu-owner">${j?escapeHtml(j.user):"available"}</span></div>`).join("");
  const running=state.jobs.filter(j=>j.status==="RUNNING");const cpu=running.reduce((n,j)=>n+j.cpus,0);const memory=running.reduce((n,j)=>n+j.memory,0);$("#cpu-meter").textContent=`${Math.round(cpu/224*100)}%`;$("#memory-meter").textContent=`${Math.round(memory/1813*100)}%`;
  $("#queue-count").textContent=`${state.jobs.length} jobs`;$("#queue-body").innerHTML=state.jobs.map(j=>`<tr data-job="${j.id}"><td>${j.id}</td><td>${escapeHtml(j.user)}</td><td class="state-${j.status.toLowerCase()}">${j.status}</td><td>${j.gpus}</td><td>${j.reason||"—"}</td></tr>`).join("")||`<tr><td colspan="5" class="muted">Queue empty</td></tr>`;
  $$("#queue-body tr[data-job]").forEach(row=>row.onclick=()=>{state.selectedJob=Number(row.dataset.job);renderLight();});
  const selected=state.jobs.find(j=>j.id===(state.selectedJob||state.activeJob));$("#job-details").innerHTML=selected?[['Job ID',selected.id],['State',selected.status],['GPUs',selected.gpus],['CPUs',selected.cpus],['Memory',selected.memory+' GiB'],['Reason',selected.reason||'—']].map(([k,v])=>`<div class="detail"><span>${k}</span>${v}</div>`).join(""):"No learner job selected.";
  $("#event-list").innerHTML=state.events.map(e=>`<li><span class="muted">${formatTime(e.at)}</span> ${escapeHtml(e.text)}</li>`).join("")||'<li class="muted">No events yet.</li>';
  renderTimeline();drawChart();
}
function renderTerminal(){const el=$("#terminal-output");el.innerHTML=state.terminal.map(line=>`<div class="term-line term-${line.kind}">${escapeHtml(line.text)}</div>`).join("");el.scrollTop=el.scrollHeight;}
function renderProgress(){const reached=[state.evidence.sinfo,state.evidence.allocation,state.evidence.env,state.evidence.gpu,state.evidence.released];let current=reached.findIndex(v=>!v);if(current<0)current=5;$("#progress-list").innerHTML=steps.map(([label],i)=>`<li><span class="step-dot ${reached[i]?"done":i===current?"current":""}">${reached[i]?"✓":i+1}</span><span>${label}</span><span class="status-text ${reached[i]?"done":i===current?"current":""}">${reached[i]?"Completed":i===current?"In progress":"Pending"}</span></li>`).join("");$("#competency-list").innerHTML=steps.map(([label],i)=>`<li class="${reached[i]?"done":""}">${label}</li>`).join("");}
function renderTimeline(){const rows=[];for(let i=0;i<8;i++){const j=state.jobs.find(x=>x.status==="RUNNING"&&(x.gpuIndices||[]).includes(i));rows.push(`<div class="timeline-row"><span>GPU ${i}</span><div class="bar-track"><div class="bar ${j?(j.user==="learner"?"learner":"virtual"):"idle"}" style="width:${j?Math.min(100,35+(j.duration-j.elapsed)/j.duration*65):100}%">${j?escapeHtml(j.user)+" ("+j.id+")":"Idle / available"}</div></div></div>`);}$("#timeline-bars").innerHTML=rows.join("");}
function drawChart(){const c=$("#util-canvas");if(!c)return;const ctx=c.getContext("2d");const w=c.width,h=c.height;ctx.clearRect(0,0,w,h);ctx.strokeStyle="#243b53";ctx.lineWidth=1;for(let y=20;y<h;y+=40){ctx.beginPath();ctx.moveTo(0,y);ctx.lineTo(w,y);ctx.stroke();}const util=(8-freeGpuIndices().length)/8;for(const [color,offset] of [["#4ddd82",util],["#aa8cff",Math.min(.9,util*.65+.12)]]){ctx.strokeStyle=color;ctx.lineWidth=3;ctx.beginPath();for(let x=0;x<w;x+=12){const v=offset+Math.sin((x+state.now)/34)*.03;const y=h-20-v*(h-40);if(x===0)ctx.moveTo(x,y);else ctx.lineTo(x,y);}ctx.stroke();}}
function renderExamMap(){const rows=[['Job submission',80],['Resource allocation',67],['GPU management',83],['Scheduling & queues',60],['Troubleshooting',50]];$("#competency-bars").innerHTML=rows.map(([name,p])=>`<div class="competency-row"><div><span>${name}</span><span>${p}%</span></div><div class="progress-track"><div class="progress-fill" style="width:${p}%"></div></div></div>`).join("");}

$("#terminal-form").addEventListener("submit",e=>{e.preventDefault();const input=$("#terminal-input");runCommand(input.value);input.value="";input.focus();});$("#clear-terminal").onclick=()=>{state.terminal=[];renderTerminal();};
$("#scenario-select").onchange=e=>reset(e.target.value);$("#pause-btn").onclick=()=>{state.paused=!state.paused;renderLight();};$$('[data-speed]').forEach(b=>b.onclick=()=>{state.speed=Number(b.dataset.speed);$$('[data-speed]').forEach(x=>x.classList.toggle('active',x===b));});
$$('.mode-tab').forEach(btn=>btn.onclick=()=>{$$('.mode-tab').forEach(x=>x.classList.remove('active'));$$('.mode-view').forEach(x=>x.classList.remove('active'));btn.classList.add('active');$(`#${btn.dataset.mode}-view`).classList.add('active');});
$("#hint-btn").onclick=()=>{state.hintLevel=Math.min(3,state.hintLevel+1);const hints=["Start by inspecting partitions and node state with `sinfo`.","Use `srun` with `--gres=gpu:h200:1`, CPU, memory, time, and `--pty bash`.","Inside the allocation, inspect `$CUDA_VISIBLE_DEVICES` and run `nvidia-smi -L`; finish with `exit`."];const box=$("#hint-box");box.textContent=hints[state.hintLevel-1];box.classList.remove('hidden');};$("#reset-lab-btn").onclick=()=>reset();
$("#script-editor").value=DEFAULT_SCRIPT;$("#script-editor").oninput=e=>{state.script=e.target.value;localStorage.setItem("dgxlab-prototype-script",state.script);};$("#validate-script").onclick=()=>{const errors=[];if(!state.script.includes("#SBATCH"))errors.push("No #SBATCH directives found");if(!/--gres=gpu(?::h200)?:\d+/.test(state.script))errors.push("No GPU GRES request");$("#script-status").textContent=errors.length?errors.join("; "):"Valid simulator script";};$("#submit-script").onclick=()=>{const job=submitFromScript();addTerm(`Submitted batch job ${job.id}`,"info");render();};$("#sandbox-contended").onclick=()=>{reset("dgx-contended");$("#scenario-select").value="dgx-contended";};$("#sandbox-clear").onclick=()=>{state.jobs=state.jobs.filter(j=>j.user!=="learner");state.activeJob=null;schedulePending();render();};
$("#exam-form").onsubmit=e=>{e.preventDefault();let answered=0,score=0;const q1=new FormData(e.target).get("q1");if(q1){answered++;if(q1==="b")score++;}const q2=[...e.target.querySelectorAll('input[name="q2"]:checked')].map(x=>x.value).sort().join(',');if(q2){answered++;if(q2==="a,b")score++;}const q3=$("#q3").value.trim().toLowerCase();if(q3){answered++;if(["gpu:h200","gpu"].includes(q3))score++;}const percent=Math.round(score/3*100);$("#exam-score").textContent=percent+"%";$("#exam-progress").textContent=answered+" / 3";const box=$("#exam-feedback");box.textContent=`${score}/3 correct (${percent}%). This prototype demonstrates deterministic local scoring; the Rust engine implements the full 60/25/15 certification gate.`;box.classList.remove('hidden');};$("#reset-exam").onclick=()=>{$("#exam-form").reset();$("#exam-score").textContent="—";$("#exam-progress").textContent="0 / 3";$("#exam-feedback").classList.add('hidden');};

reset();
