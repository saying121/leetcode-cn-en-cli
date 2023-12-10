use std::{
    sync::{
        atomic::Ordering,
        mpsc::{self, Receiver, Sender},
        Arc, Condvar, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    self,
    event::{self, Event},
};
use miette::{IntoDiagnostic, Result};

use crate::leetcode::{
    qs_detail::Question, resps::run_res::RunResult, CUR_NEW_QS_INDEX_NUM,
    CUR_QS_INDEX_NUM, TOTAL_NEW_QS_INDEX_NUM, TOTAL_QS_INDEX_NUM,
};

pub enum UserEvent {
    TermEvent(Event),
    GetQsDone(Box<Question>),
    Syncing(f64),
    SyncDone,
    SyncingNew(f64),
    SyncDoneNew,

    SubmitCode(u32),
    SubmitDone(Box<RunResult>),
    TestCode(u32),
    TestDone(Box<RunResult>),
}

pub struct Events {
    pub rx: Receiver<UserEvent>,
    pub tx: Sender<UserEvent>,
    pub is_shutdown: bool,
}

impl Events {
    pub fn new(tick_rate: Duration, flag: Arc<Mutex<bool>>, cond: Arc<Condvar>) -> Self {
        // `tokio::sync::mpsc` hover
        // Unbounded channel: You should use the kind of channel that matches where the receiver is.
        // So for sending a message from async to sync, you should use the
        // standard library unbounded channel or crossbeam. Similarly,
        // for sending a message from sync to async, you should use an unbounded Tokio mpsc channel.
        let (tx, rx) = mpsc::channel();
        let event_tx = tx.clone();

        let mut last_tick = Instant::now();
        let mut last_tick_progress = Instant::now();

        thread::spawn(move || loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            let mut flag_v;
            if let Ok(v) = flag.try_lock() {
                flag_v = *v;
            } else {
                flag_v = true;
            }

            while !flag_v {
                flag_v = *cond
                    .wait(flag.lock().unwrap())
                    .unwrap();
            }

            if crossterm::event::poll(timeout).unwrap_or_default() {
                if let Ok(event) = event::read() {
                    event_tx
                        .send(UserEvent::TermEvent(event))
                        .expect("send event failed");
                }
            }

            let tot: f64 = TOTAL_QS_INDEX_NUM
                .load(Ordering::Acquire)
                .try_into()
                .unwrap_or_default();

            if tot > 0.0 && last_tick_progress.elapsed() > Duration::from_secs(1) {
                last_tick_progress = Instant::now();
                let cur = CUR_QS_INDEX_NUM.load(Ordering::Acquire);
                let cur: f64 = cur.try_into().unwrap_or_default();
                event_tx
                    .send(UserEvent::Syncing(cur / tot))
                    .expect("send error");
            }

            let tot: f64 = TOTAL_NEW_QS_INDEX_NUM
                .load(Ordering::Acquire)
                .try_into()
                .unwrap_or_default();
            if tot > 0.0 && last_tick_progress.elapsed() > Duration::from_secs(1) {
                last_tick_progress = Instant::now();
                let cur = CUR_NEW_QS_INDEX_NUM.load(Ordering::Acquire);
                let cur: f64 = cur.try_into().unwrap_or_default();
                event_tx
                    .send(UserEvent::SyncingNew(cur / tot))
                    .expect("send error");
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        });

        Self {
            rx,
            tx,
            is_shutdown: false,
        }
    }

    pub fn next(&self) -> Result<UserEvent> {
        self.rx.recv().into_diagnostic()
    }
}
