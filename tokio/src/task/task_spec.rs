use std::cmp::Ordering;
use std::time::SystemTime;

/// Provides the runtime with information about the characteristics of a task.
#[derive(Debug)]
pub struct TaskSpec {
    /// The deadline by which the task should complete.
    /// The runtime will prioritize tasks with earlier deadlines,
    /// provided that they have the same priority.
    pub deadline: Option<SystemTime>,
    /// The priority of the task. The runtime will attempt execute tasks with larger
    /// priorities first.
    pub priority: u8,
}

impl Default for TaskSpec {
    fn default() -> Self {
        Self {
            deadline: None,
            priority: 0,
        }
    }
}

impl Ord for TaskSpec {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => match (&self.deadline, &other.deadline) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(dl), Some(dr)) => dl.cmp(dr).reverse(),
            },
            ord => ord,
        }
    }
}

impl PartialOrd for TaskSpec {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TaskSpec {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialEq for TaskSpec {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
