// Freenet Scaffold state: Byzantine-fault-tolerant consensus

use common::{LotteryState, RenderingTask, LotteryResult, WorkerId};
use std::collections::HashMap;

/// Distributed state using Freenet Scaffold pattern
pub struct FreenetScaffoldState {
    state: LotteryState,
}

impl FreenetScaffoldState {
    pub fn new() -> Self {
        Self {
            state: LotteryState::new(),
        }
    }

    pub fn add_task(&mut self, task: RenderingTask) {
        self.state.tasks.insert(task.id, task);
    }

    pub fn add_lottery_result(&mut self, result: LotteryResult) {
        self.state.lottery_results.push(result);
    }

    pub fn update_participant_credits(&mut self, worker_id: &WorkerId, credits: u64) {
        self.state
            .participant_records
            .entry(worker_id.clone())
            .and_modify(|r| r.credits_balance += credits)
            .or_insert_with(|| common::ParticipantRecord {
                worker_id: worker_id.clone(),
                credits_balance: credits,
                reputation_score: 50.0,
                segments_completed: 0,
                lottery_wins: 1,
                tasks_submitted: 0,
                joined_at: 0,
                badges: vec![],
            });
    }

    pub fn merge(&mut self, other: Self) {
        self.state.merge(other.state);
    }

    pub fn lottery_results_count(&self) -> u64 {
        self.state.lottery_results.len() as u64
    }

    pub fn epoch(&self) -> u64 {
        self.state.epoch
    }
}
