use crate::Instant;
use alloc::{boxed::Box, collections::VecDeque};
use core::{
    future::Future,
    hash::BuildHasher,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use embedded_time::duration::Milliseconds;
use hashbrown::HashMap;

struct DefaultHasher;

impl BuildHasher for DefaultHasher {
    type Hasher = Hasher;

    fn build_hasher(&self) -> Hasher {
        Hasher::default()
    }
}

#[derive(Default)]
struct Hasher {
    val: Option<i32>,
}

impl core::hash::Hasher for Hasher {
    fn finish(&self) -> u64 {
        self.val.expect("Hasher did not have a value ready") as u64
    }
    fn write(&mut self, hash: &[u8]) {
        match hash.try_into() {
            Ok(hash) => {
                if self.val.is_some() {
                    panic!(
                        "`self.val` already has a value, {} -> {}",
                        self.val.unwrap(),
                        i32::from_be_bytes(hash)
                    );
                }
                self.val = Some(i32::from_be_bytes(hash))
            }
            Err(_) => unreachable!(),
        };
    }
}

struct Task<T> {
    fut: Pin<Box<dyn Future<Output = T>>>,
}

impl<T> Task<T> {
    fn new(fut: impl Future<Output = T> + 'static) -> Self {
        Self { fut: Box::pin(fut) }
    }
}

pub fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    crate::interrupt_free(|| {
        let executor = unsafe { &mut EXECUTOR.as_mut().unwrap() };
        let id = TaskId(executor.next_task_id);
        executor.next_task_id += 1;

        executor.tasks.insert(id, Task::new(fut));
        executor.queue.push_back(id);
    });
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct TaskId(i32);

impl TaskId {
    pub fn to_waker(self) -> Waker {
        fn clone(ptr: *const ()) -> RawWaker {
            RawWaker::new(ptr, VTABLE)
        }
        fn wake(ptr: *const ()) {
            let task_id = TaskId(ptr as usize as i32);
            crate::interrupt_free(|| {
                let queue = unsafe { &mut EXECUTOR.as_mut().unwrap().queue };
                if !queue.contains(&task_id) {
                    queue.push_back(task_id);
                }
            });
        }
        fn drop(_: *const ()) {}
        static VTABLE: &RawWakerVTable = &RawWakerVTable::new(clone, wake, wake, drop);
        let raw = RawWaker::new(self.0 as usize as *const (), VTABLE);
        unsafe { Waker::from_raw(raw) }
    }
    pub fn from_context(context: &Context<'_>) -> Self {
        let data = context.waker().as_raw().data();
        Self(data as usize as i32)
    }
}

struct Executor {
    next_task_id: i32,
    tasks: HashMap<TaskId, Task<()>, DefaultHasher>,
    queue: VecDeque<TaskId>,
}

// We are a single threaded WASM binary
unsafe impl Send for Executor {}
unsafe impl Sync for Executor {}

static mut EXECUTOR: Option<Executor> = None;

struct Sleep {
    target: Instant,
}

impl Future for Sleep {
    type Output = ();
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<()> {
        let target = self.as_ref().target.clone();
        let now = Instant::now();
        if now >= target {
            Poll::Ready(())
        } else {
            let task_id = TaskId::from_context(ctx);
            let diff = target - now;
            unsafe { crate::ffi::notify_after(diff.0 as i32, task_id.0) };
            Poll::Pending
        }
    }
}

pub async fn sleep(duration: impl Into<Milliseconds>) {
    Sleep {
        target: Instant::now() + duration.into(),
    }
    .await
}

pub fn start_async(main: impl Future<Output = !> + 'static) -> ! {
    let mut executor = Executor {
        next_task_id: 2,
        tasks: HashMap::with_hasher(DefaultHasher),
        queue: VecDeque::new(),
    };
    executor.tasks.insert(
        TaskId(1),
        Task::new(async {
            main.await;
        }),
    );
    executor.queue.push_back(TaskId(1));
    unsafe { EXECUTOR = Some(executor) };
    loop {
        loop {
            let executor = unsafe { EXECUTOR.as_mut().unwrap() };
            let task_id = match crate::interrupt_free(|| executor.queue.pop_front()) {
                Some(task_id) => task_id,
                None => break,
            };
            let fut = executor.tasks.get_mut(&task_id).expect("Task not found");
            let waker = task_id.to_waker();
            match fut.fut.as_mut().poll(&mut Context::from_waker(&waker)) {
                Poll::Ready(()) if task_id.0 == 1 => {
                    panic!("Main thread exited. This is a bug.");
                }
                Poll::Ready(()) => {
                    executor.tasks.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }

        // no tasks ready to be polled
        unsafe {
            crate::ffi::wait_for_interrupt();
        }
    }
}
