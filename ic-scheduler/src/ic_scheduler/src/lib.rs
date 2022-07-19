#![feature(binary_heap_retain)]

mod business_logic;
mod env;
mod lifetime;

use crate::env::{CanisterEnv, EmptyEnv, Environment};
use business_logic::{BusinessState, ScheduleUnit, Task};
use candid::{candid_method, CandidType, Principal};
// use ic_cdk::print;
use ic_cdk_macros::*;
use serde::Deserialize;

use std::cell::{RefCell, RefMut};

const MILLIS_TO_SECONDS: u64 = 1_000_000_000;

thread_local! {
    static RUNTIME_STATE: RefCell<RuntimeState> = RefCell::default();
}

struct RuntimeState {
    pub env: Box<dyn Environment>,
    pub data: Data,
}

impl Default for RuntimeState {
    fn default() -> Self {
        RuntimeState {
            env: Box::new(EmptyEnv {}),
            data: Data::default(),
        }
    }
}

#[derive(CandidType, Default, Deserialize)]
struct Data {
    business_state: BusinessState,
}

#[candid_method(update)]
#[update(name = "start_cron")]
fn start_cron(interval_seconds: u64, task_id: Option<u64>) -> bool {
    RUNTIME_STATE.with(|state| start_cron_impl(interval_seconds, task_id, state.borrow_mut()))
}

// Should we create the schedule unit here or just call into business_state?
fn start_cron_impl(
    interval_seconds: u64,
    task_id: Option<u64>,
    mut runtime_state: RefMut<RuntimeState>,
) -> bool {
    // let task_id = task_id.unwrap_or(0);

    let timestamp = runtime_state.env.now() + interval_seconds * MILLIS_TO_SECONDS;
    let canister_id = runtime_state.env.caller();

    runtime_state.data.business_state.schedule_queue.add_task(
        timestamp,
        canister_id,
        interval_seconds,
        task_id,
    )

    // let task = Task {
    //     interval_seconds,
    //     canister_id: runtime_state.env.caller(),
    //     task_id,
    // };

    // let unit = ScheduleUnit {
    //     timestamp: runtime_state.env.now() + interval_seconds * MILLIS_TO_SECONDS,
    //     task,
    // };

    // runtime_state.data.business_state.schedule_queue.push(unit)
}

// Used only for testing
#[candid_method(update)]
#[update(name = "add_cron")]
fn add_cron(canister_id: Principal, interval_seconds: u64, task_id: Option<u64>) -> bool {
    RUNTIME_STATE
        .with(|state| add_cron_impl(canister_id, interval_seconds, task_id, state.borrow_mut()))
}

fn add_cron_impl(
    canister_id: Principal,
    interval_seconds: u64,
    task_id: Option<u64>,
    mut runtime_state: RefMut<RuntimeState>,
) -> bool {
    let task_id = task_id.unwrap_or(0);

    let task = Task {
        interval_seconds,
        canister_id,
        task_id,
    };

    let unit = ScheduleUnit {
        timestamp: runtime_state.env.now() + interval_seconds * MILLIS_TO_SECONDS,
        task,
    };

    runtime_state.data.business_state.schedule_queue.push(unit)
}

#[candid_method(update)]
#[update(name = "clear_all")]
fn clear_all() -> bool {
    RUNTIME_STATE.with(|state| {
        state
            .borrow_mut()
            .data
            .business_state
            .schedule_queue
            .clear()
    });

    ic_cdk::print("Cleared all entries");

    true
}

#[candid_method(update, rename = "stop_cron")]
#[update(name = "stop_cron")]
fn stop_cron_retain(task_id: Option<u64>) -> bool {
    RUNTIME_STATE.with(|state| stop_cron_retain_impl(task_id, state.borrow_mut()))
}

fn stop_cron_retain_impl(task_id: Option<u64>, mut runtime_state: RefMut<RuntimeState>) -> bool {
    let task_id = task_id.unwrap_or(0);
    let canister_id = runtime_state.env.caller();

    if task_id == 0 {
        ic_cdk::print(format!(
            "Removing all tasks for canister_id {}",
            canister_id
        ));
        runtime_state
            .data
            .business_state
            .schedule_queue
            .remove_all(canister_id);
    } else {
        ic_cdk::print(format!(
            "Removing task {} for canister_id {}",
            task_id, canister_id
        ));
        runtime_state
            .data
            .business_state
            .schedule_queue
            .remove_one(canister_id, task_id);
    }

    true
}

// Auto export the candid interface
candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

#[test]
fn check_governance_candid_file() {
    // candid::export_service!();
    let expected = __export_service();

    println!("{}", expected);
}
