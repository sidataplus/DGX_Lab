//! Pending-reason explanations for the learning UI (not a full Slurm reason catalog).

use slurm_model::PendingReason;

#[must_use]
pub fn explain_pending(reason: PendingReason) -> &'static str {
    match reason {
        PendingReason::Resources => {
            "The job is eligible, but the requested CPUs, memory, or GPUs are not free on any eligible node right now."
        }
        PendingReason::Priority => {
            "Higher-priority or earlier-submitted work is ahead in the simulated queue ordering."
        }
        PendingReason::Dependency => {
            "The job is waiting on a dependency (for example afterok) that has not been satisfied."
        }
        PendingReason::InvalidAccount => {
            "The requested account is not valid in this teaching profile."
        }
        PendingReason::QosMaxJobsPerUserLimit => {
            "A QOS limit on running jobs per user is blocking this submission."
        }
        PendingReason::QosMaxGresPerUser => {
            "A QOS limit on GPUs (GRES) per user is blocking this submission."
        }
        PendingReason::Reservation => {
            "The job requires a reservation that is not currently available to this user."
        }
        PendingReason::PartitionDown => {
            "The target partition is not accepting jobs in the current scenario."
        }
        PendingReason::None => "No pending reason is recorded.",
    }
}
