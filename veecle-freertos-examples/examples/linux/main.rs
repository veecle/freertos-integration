//! A basic veecle-freertos-integration example for Linux.
use veecle_freertos_integration::*;

// SAFETY: We don't use any non-FreeRTOS threads.
#[global_allocator]
static GLOBAL: FreeRtosAllocator = unsafe { FreeRtosAllocator::new() };

fn main() {
    #[repr(align(128))]
    #[derive(Debug)]
    struct Test {
        _content: usize,
    }

    let x = Box::new(Test { _content: 42 });
    println!("Boxed Test '{x:?}' (allocator large alignment test)");
    assert!(core::ptr::addr_of!(*x).is_aligned());

    let x = Box::new(15);
    println!("Boxed int '{x}' (allocator test)");

    hooks::set_on_assert(|file_name, line| {
        println!("file name and line: {file_name}:{line}",);
    });

    println!("Starting FreeRTOS app ...");
    Task::new()
        .name(c"hello")
        .stack_size(128)
        .priority(TaskPriority(2))
        .start(|_this_task| {
            let mut i = 0;
            loop {
                println!("Hello from Task! {i}");
                CurrentTask::delay(Duration::from_ms(1000));
                i += 1;
            }
        })
        .unwrap();
    println!("Task registered");

    println!("Starting scheduler");
    veecle_freertos_integration::scheduler::start_scheduler();
    #[allow(unreachable_code)]
    loop {
        println!("Loop forever!");
    }
}
