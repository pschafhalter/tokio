cfg_rt! {
    use crate::future::Future;
    use crate::runtime::basic_scheduler;
    use crate::task::JoinHandle;

    use std::time::{SystemTime, Duration};
}

cfg_rt_multi_thread! {
    use crate::runtime::thread_pool;
}

#[derive(Debug, Clone)]
pub(crate) enum Spawner {
    #[cfg(feature = "rt")]
    Basic(basic_scheduler::Spawner),
    #[cfg(feature = "rt-multi-thread")]
    ThreadPool(thread_pool::Spawner),
}

impl Spawner {
    pub(crate) fn shutdown(&mut self) {
        #[cfg(feature = "rt-multi-thread")]
        {
            if let Spawner::ThreadPool(spawner) = self {
                spawner.shutdown();
            }
        }
    }
}

cfg_rt! {
    impl Spawner {
        pub(crate) fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
        where
            F: Future + Send + 'static,
            F::Output: Send + 'static,
        {
            match self {
                #[cfg(feature = "rt")]
                Spawner::Basic(spawner) => spawner.spawn(future, None),
                #[cfg(feature = "rt-multi-thread")]
                Spawner::ThreadPool(spawner) => spawner.spawn(future, None),
            }
        }

        pub(crate) fn spawn_with_deadline<F>(&self, future: F, deadline: SystemTime) -> JoinHandle<F::Output>
        where
            F: Future + Send + 'static,
            F::Output: Send + 'static,
        {
            match self {
                #[cfg(feature = "rt")]
                Spawner::Basic(spawner) => spawner.spawn(future, Some(deadline)),
                #[cfg(feature = "rt-multi-thread")]
                Spawner::ThreadPool(spawner) => spawner.spawn(future, Some(deadline)),
            }
        }
    }
}
