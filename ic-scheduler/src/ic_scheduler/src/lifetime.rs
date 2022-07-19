use crate::{CanisterEnv, Data, RuntimeState, RUNTIME_STATE};

use candid::Principal;
use ic_cdk::api::call::CallResult;
#[allow(unused_imports)]
use ic_cdk_macros::{heartbeat, init, post_upgrade, pre_upgrade};

#[init]
fn init() {
    let env = Box::new(CanisterEnv::new());
    let data = Data::default();
    let runtime_state = RuntimeState { env, data };

    RUNTIME_STATE.with(|state| *state.borrow_mut() = runtime_state);
}

#[pre_upgrade]
fn pre_upgrade() {
    RUNTIME_STATE.with(|state| ic_cdk::storage::stable_save((&state.borrow().data,)).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
    let env = Box::new(CanisterEnv::new());
    let (data,): (Data,) = ic_cdk::storage::stable_restore().unwrap();
    let runtime_state = RuntimeState { env, data };

    RUNTIME_STATE.with(|state| *state.borrow_mut() = runtime_state);
}

#[heartbeat]
fn heartbeat() {
    let now = RUNTIME_STATE.with(|state| state.borrow().env.now());

    // grab all the scheduled units that were due before "now"
    let ready = RUNTIME_STATE.with(|state| {
        state
            .borrow_mut()
            .data
            .business_state
            .schedule_queue
            .pop_before(now)
    });

    // send the inter-canister call
    for unit in ready.iter() {
        // execute the task
        ic_cdk::spawn(call_canister(unit.task.canister_id, unit.task.task_id));
    }
}

async fn call_canister(canister_id: Principal, task_id: u64) {
    let call_succeeded: CallResult<(bool,)> =
        ic_cdk::api::call::call(canister_id, "remote_heartbeat", (task_id,)).await;

    ic_cdk::print(format!(
        "Called {} with {} --- Got: {:?}",
        canister_id, task_id, call_succeeded
    ));
}
