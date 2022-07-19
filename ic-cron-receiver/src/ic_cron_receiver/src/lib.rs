mod env;
mod lifetime;

use crate::env::{CanisterEnv, EmptyEnv, Environment};
use candid::CandidType;
use ic_cdk::print;
use ic_cdk_macros::*;
use serde::Deserialize;

use std::cell::{Ref, RefCell};

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
    items: Vec<String>,
}

#[ic_cdk_macros::query]
fn greet(name: String) -> String {
    format!("Hello, {}! Welcome!", name)
}

#[update(name = "add")]
fn add(name: String) -> bool {
    RUNTIME_STATE.with(|state| add_impl(name, &mut state.borrow_mut()))
}

fn add_impl(name: String, runtime_state: &mut RuntimeState) -> bool {
    let now = runtime_state.env.now();

    // id is printed just to check that random works as expected
    print(format!(
        "Adding {} at {} with id {}",
        name,
        now,
        runtime_state.env.random_u32()
    ));
    runtime_state.data.items.push(name);

    true
}

#[query(name = "getAll")]
fn get_all() -> Vec<String> {
    RUNTIME_STATE.with(|state| get_all_impl(state.borrow()))
}

fn get_all_impl(runtime_state: Ref<RuntimeState>) -> Vec<String> {
    runtime_state.data.items.iter().cloned().collect()
}

#[update(name = "remote_heartbeat")]
fn remote_heartbeat(task_id: Option<u64>) {
    let task_id = task_id.unwrap_or(0);
    ic_cdk::print(format!("Received a remote heartbeat for task {}", task_id));
}
