use axum::async_trait;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::time::Interval;
use tokio::{
    spawn,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
};
pub trait CacheValue {
    fn is_expire(&self) -> bool;
}

#[async_trait]
pub trait Job {
    async fn execute(&self);
}

#[derive(Clone)]
pub struct Cache<K, V>(pub *mut HashMap<K, V>);

unsafe impl<K, V> Send for Cache<K, V> {}
unsafe impl<K, V> Sync for Cache<K, V> {}
impl<K, V> Deref for Cache<K, V> {
    type Target = HashMap<K, V>;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<K, V> DerefMut for Cache<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

#[allow(unused)]
impl<K, V> Cache<K, V>
where
    K: PartialEq,
    V: CacheValue,
{
    fn new(m: &mut HashMap<K, V>) -> Self {
        Cache(m)
    }

    pub fn clean_expired_value(&mut self) {
        self.retain(|_, v| v.is_expire());
    }

    pub async fn clean_task(&mut self, mut interval: Interval) {
        loop {
            interval.tick().await;
            self.clean_expired_value();
        }
    }
}

#[allow(unused)]
pub struct Executor {}
#[allow(unused)]
impl Executor {
    pub async fn run<T>(recv: Arc<Mutex<UnboundedReceiver<T>>>)
    where
        T: 'static + Job + Sync + Send,
    {
        spawn(async move {
            loop {
                let job = recv.lock().await.recv().await;
                match job {
                    Some(job) => job.execute().await,
                    None => {}
                }
            }
        });
    }
}
#[allow(unused)]
pub struct ExecutorPool<T> {
    sender: UnboundedSender<T>,
}
#[allow(unused)]
impl<T> ExecutorPool<T>
where
    T: 'static + Job + Sync + Send,
{
    pub async fn new(executor_num: usize) -> Self {
        let (sender, recv) = unbounded_channel();
        let recv = Arc::new(Mutex::new(recv));
        for _ in 0..executor_num {
            Executor::run(Arc::clone(&recv)).await;
        }
        ExecutorPool { sender }
    }
    pub fn get_sender(&self) -> UnboundedSender<T> {
        self.sender.clone()
    }
}
