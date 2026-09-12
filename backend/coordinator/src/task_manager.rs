// Task manager: manages rendering tasks and segment lifecycle

use common::{TaskId, RenderingTask, Segment, SegmentId, SegmentStatus, WorkerId, LotteryEntry, EntryType};
use std::collections::HashMap;

/// Manages rendering tasks and their segments
pub struct TaskManager {
    tasks: HashMap<TaskId, RenderingTask>,
    segments: HashMap<SegmentId, Segment>,
    segment_assignments: HashMap<SegmentId, WorkerId>,
    completed_segments: Vec<SegmentId>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            segments: HashMap::new(),
            segment_assignments: HashMap::new(),
            completed_segments: Vec::new(),
        }
    }

    pub async fn add_task(&mut self, task: RenderingTask) {
        self.tasks.insert(task.id, task);
    }

    pub async fn add_segment(&mut self, segment: Segment) {
        self.segments.insert(segment.id, segment);
    }

    pub async fn mark_segment_completed(&mut self, segment_id: &SegmentId, worker_id: &WorkerId) {
        if let Some(segment) = self.segments.get_mut(segment_id) {
            segment.assigned_to = Some(worker_id.clone());
            segment.status = SegmentStatus::Verified;
        }
        self.segment_assignments.insert(*segment_id, worker_id.clone());
        self.completed_segments.push(*segment_id);
    }

    pub async fn assign_segment(&mut self, segment_id: &SegmentId, worker_id: &WorkerId) {
        if let Some(segment) = self.segments.get_mut(segment_id) {
            segment.assigned_to = Some(worker_id.clone());
            segment.status = SegmentStatus::Assigned;
        }
        self.segment_assignments.insert(*segment_id, worker_id.clone());
    }

    pub async fn is_task_complete(&self, task_id: TaskId) -> bool {
        let total = self.segments.values().filter(|s| s.task_id == task_id).count();
        let completed = self.completed_segments
            .iter()
            .filter(|s| s.task_id == task_id)
            .count();
        total > 0 && total == completed
    }

    pub async fn get_lottery_entries(&self, task_id: TaskId) -> Vec<LotteryEntry> {
        self.completed_segments
            .iter()
            .filter(|seg_id| seg_id.task_id == task_id)
            .enumerate()
            .map(|(idx, seg_id)| LotteryEntry {
                id: idx as u64,
                task_id,
                worker_id: self
                    .segment_assignments
                    .get(seg_id)
                    .cloned()
                    .unwrap_or_else(|| common::WorkerId("unknown".into())),
                segment_id: *seg_id,
                entry_type: EntryType::SegmentCompletion,
                created_at: 0,
            })
            .collect()
    }

    pub async fn get_task(&self, task_id: TaskId) -> Option<RenderingTask> {
        self.tasks.get(&task_id).cloned()
    }

    pub async fn get_pending_segments(&self, limit: usize) -> Vec<Segment> {
        self.segments
            .values()
            .filter(|s| s.status == SegmentStatus::Pending)
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn task_count(&self) -> u64 {
        self.tasks.len() as u64
    }

    pub async fn pending_segment_count(&self) -> u64 {
        self.segments
            .values()
            .filter(|s| s.status == SegmentStatus::Pending)
            .count() as u64
    }

    pub async fn completed_segment_count(&self) -> u64 {
        self.completed_segments.len() as u64
    }
}
