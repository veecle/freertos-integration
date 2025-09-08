use veecle_freertos_sys::bindings::{BaseType_t, taskYIELD};

/// Keep track of whether we need to yield the execution to a different
/// task at the end of the interrupt.
///
/// Should be dropped as the last thing inside a interrupt.
#[derive(Debug)]
pub struct InterruptContext {
    x_higher_priority_task_woken: BaseType_t,
}

impl Default for InterruptContext {
    fn default() -> InterruptContext {
        InterruptContext::new()
    }
}

impl InterruptContext {
    /// Instantiate a new context.
    pub fn new() -> InterruptContext {
        InterruptContext {
            x_higher_priority_task_woken: 0,
        }
    }

    pub fn get_task_field_mut(&mut self) -> *mut BaseType_t {
        &raw mut self.x_higher_priority_task_woken
    }
    pub fn higher_priority_task_woken(&self) -> BaseType_t {
        self.x_higher_priority_task_woken
    }
}

impl Drop for InterruptContext {
    fn drop(&mut self) {
        if self.x_higher_priority_task_woken == 1 {
            taskYIELD()
        }
    }
}
