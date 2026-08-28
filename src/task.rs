use std::{
    cmp::Reverse,
    time::{Duration, Instant},
};

use crate::state::State;

pub struct Task {
    time: Instant,
    run: Box<dyn FnOnce(&mut State)>,
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.time.cmp(&other.time))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

pub fn update_tasks(state: &mut State) {
    let task_is_due = state
        .task_queue
        .peek()
        .is_some_and(|Reverse(task)| task.time <= Instant::now());

    if task_is_due {
        let Reverse(task) = state.task_queue.pop().unwrap();
        (task.run)(state);
        update_tasks(state);
    }
}

pub fn add_task(time: Duration, callback: impl FnOnce(&mut State) + 'static, state: &mut State) {
    state.task_queue.push(Reverse(Task {
        time: Instant::now() + time,
        run: Box::new(callback),
    }));
}

pub fn add_data_task<T>(
    time: Duration,
    data: T,
    callback: impl FnOnce(&mut State, T) + 'static,
    state: &mut State,
) where
    T: 'static,
{
    add_task(time, |state| callback(state, data), state);
}
