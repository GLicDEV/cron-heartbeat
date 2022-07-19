use candid::Principal;
use ic_cdk::export::candid::{CandidType, Deserialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::MILLIS_TO_SECONDS;

#[derive(Clone, Deserialize, CandidType, Default)]
pub struct BusinessState {
    pub(crate) schedule_queue: ScheduleQueue,
}

#[derive(Clone, Deserialize, CandidType, Default)]
pub struct ScheduleQueue(BinaryHeap<ScheduleUnit>);

#[derive(Clone, Deserialize, CandidType, Debug)]
pub struct ScheduleUnit {
    pub(crate) timestamp: u64,
    pub(crate) task: Task,
}

#[derive(Clone, Deserialize, CandidType, Debug)]
pub struct Task {
    pub(crate) interval_seconds: u64,
    pub(crate) canister_id: Principal,
    pub(crate) task_id: u64,
}

#[allow(dead_code)]
impl ScheduleQueue {
    pub fn push(&mut self, unit: ScheduleUnit) -> bool {
        self.0.push(unit);
        true
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn pop_one(&mut self) -> Option<ScheduleUnit> {
        self.0.pop()
    }

    pub fn remove_one(&mut self, canister_id: Principal, task_id: u64) {
        self.0
            .retain(|unit| !(unit.task.canister_id == canister_id && unit.task.task_id == task_id));
    }

    pub fn remove_all(&mut self, canister_id: Principal) {
        self.0
            .retain(|unit| !(unit.task.canister_id == canister_id));
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn schedule_task(&mut self, timestamp: u64, task: Task) -> bool {
        self.push(ScheduleUnit { timestamp, task })
    }

    pub fn add_task(
        &mut self,
        timestamp: u64,
        canister_id: Principal,
        interval_seconds: u64,
        task_id: Option<u64>,
    ) -> bool {
        let task_id = task_id.unwrap_or(0);

        self.schedule_task(
            timestamp,
            Task {
                interval_seconds,
                canister_id,
                task_id,
            },
        )
    }

    pub fn pop_before(&mut self, timestamp: u64) -> Vec<ScheduleUnit> {
        let mut peek = self.0.peek();

        if peek.is_none() {
            return Vec::new();
        }

        let mut res = Vec::new();

        while peek.unwrap().timestamp <= timestamp {
            let mut new_task = self.0.pop().unwrap();
            res.push(new_task.clone());

            // re-add the task to the "queue" at timestamp + interval_seconds +1
            // so we don't loop forever in case interval_seconds is 0 for some tasks.
            new_task.timestamp =
                timestamp + (new_task.task.interval_seconds * MILLIS_TO_SECONDS) + 1;
            self.push(new_task);

            peek = self.0.peek();
            if peek.is_none() {
                break;
            }
        }

        res
    }
}
impl PartialEq for ScheduleUnit {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl Eq for ScheduleUnit {}

impl PartialOrd for ScheduleUnit {
    // We need to reverse the order of timestamps. BinaryHeap max needs to be the
    // "earliest" timestamp, so we extract them in chronological order.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp
            .partial_cmp(&other.timestamp)
            .map(|x| x.reverse())
    }

    // lower is greater
    fn lt(&self, other: &Self) -> bool {
        self.timestamp.gt(&other.timestamp)
    }

    fn le(&self, other: &Self) -> bool {
        self.timestamp.ge(&other.timestamp)
    }

    // greater is lower
    fn gt(&self, other: &Self) -> bool {
        self.timestamp.lt(&other.timestamp)
    }

    fn ge(&self, other: &Self) -> bool {
        self.timestamp.le(&other.timestamp)
    }
}

impl Ord for ScheduleUnit {
    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        std::cmp::max_by(self, other, Ord::cmp)
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        std::cmp::min_by(self, other, Ord::cmp)
    }

    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        if self.timestamp < max.timestamp {
            max
        } else if self.timestamp > min.timestamp {
            min
        } else {
            self
        }
    }

    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp).reverse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue() {
        let mut business_state = BusinessState::default();

        let task = Task {
            interval_seconds: 60,
            canister_id: Principal::anonymous(),
            task_id: 0,
        };

        let unit = ScheduleUnit {
            timestamp: 10001,
            task,
        };

        business_state.schedule_queue.push(unit.clone());

        assert_eq!(business_state.schedule_queue.len(), 1);

        business_state.schedule_queue.pop_one();

        assert_eq!(business_state.schedule_queue.len(), 0);
    }

    #[test]
    fn test_queue_pop_before() {
        let mut business_state = BusinessState::default();

        let task = Task {
            interval_seconds: 60,
            canister_id: Principal::anonymous(),
            task_id: 0,
        };

        let unit = ScheduleUnit {
            timestamp: 10001,
            task,
        };

        business_state.schedule_queue.push(unit.clone());

        let just_one = business_state.schedule_queue.pop_before(10005);

        assert_eq!(just_one.len(), 1);

        let just_one = business_state.schedule_queue.pop_before(1000);

        assert_eq!(just_one.len(), 0);
    }

    #[test]
    fn test_queue_push_same_val() {
        let mut business_state = BusinessState::default();

        let task = Task {
            interval_seconds: 60,
            canister_id: Principal::anonymous(),
            task_id: 0,
        };

        let unit = ScheduleUnit {
            timestamp: 10001,
            task,
        };

        business_state.schedule_queue.push(unit.clone());
        business_state.schedule_queue.push(unit.clone());
        business_state.schedule_queue.push(unit.clone());

        assert_eq!(business_state.schedule_queue.len(), 3);

        let all_three = business_state.schedule_queue.pop_before(1000000);

        assert_eq!(all_three.len(), 3);
    }

    #[test]
    fn test_print() {
        println!("Hello World!");
    }
}
