//! Native golden digest for the Lab 04 transcript scaffold.
//! Kept under tests/ for documentation; the executable test lives in sim-session.
//!
//! Canonical commands (NEXT_ACTIONS / M1 target):
//! sinfo
//! srun --gres=gpu:h200:1 --cpus-per-task=8 --mem=64G --time=00:30:00 --pty bash
//! echo $SLURM_JOB_ID
//! echo $CUDA_VISIBLE_DEVICES
//! nvidia-smi -L
//! exit
//! sacct
