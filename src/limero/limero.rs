use crossbeam::channel::{bounded, Receiver, Sender};
use log::{debug, error, info, trace, warn};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::cell::RefCell;
use crate::limero::Source;
use crate::limero::Sink;

//type Callback = Box<dyn Fn() + Send + Sync + 'static>;

type Callback = Arc<Mutex<Box<dyn 'static + FnMut() + Send + Sync>>>;
pub type SafeThread = Arc<RefCell<Thread>>;
pub type SafeTimer = Arc<RefCell<Timer>>;


pub enum ThreadCommand {
    WakeUp,            // Wake up the thread and recalc timers
    Terminate,         // Terminate the thread
    Execute(Callback), // Execute a callback on the thread
}

pub struct Timer {
    expires_at: Instant,
    repeat: bool,
    active: bool,
    interval: Duration,
    callback: Option<Callback>,
}

 impl Timer {
    fn new(repeat: bool, interval: Duration) -> Timer {
        Timer {
            expires_at: Instant::now() + interval,
            repeat,
            active: true,
            interval,
            callback:None,
        }
    }
    pub fn on_expire(&mut self,cb : Callback) {
        self.callback=Some(cb);
    }
    fn start(&mut self) {
        self.expires_at = Instant::now() + self.interval;
    }
    fn restart(&mut self) {
        self.expires_at = self.expires_at + self.interval;
    }
    fn stop(&mut self) {
        self.active = false;
    }
    fn expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
    fn delta(&self) -> Duration {
        self.expires_at - Instant::now()
    }
}


pub struct Thread {
    receiver: Receiver<ThreadCommand>,
    sender: Sender<ThreadCommand>,
    timers: Vec<SafeTimer>,
    thread_handle: Option<thread::JoinHandle<()>>,
    please_stop: bool,
}

impl Thread {
    pub fn new() -> Thread {
        let (sender, receiver) = bounded(100);
        let thread = Thread {
            receiver,
            sender,
            timers: Vec::new(),
            thread_handle: None,
            please_stop: false,
        };
        thread
    }

    pub fn start(&mut self) {
        self.please_stop = false;
        let thread_handle = thread::spawn(move || {
            info!("Thread started");
        });
    }

    pub fn send(&self, command: ThreadCommand) {
        self.sender.send(command).expect("Failed to send command");
    }

    pub fn new_timer(
        & mut self,
        repeat: bool,
        interval: Duration,
    ) -> SafeTimer {
        let timer = Arc::new(RefCell::new(Timer::new(repeat, interval)));
        self.timers.push(timer.clone());
        timer
    }

    pub fn del_timer(&mut self, timer: &Timer) {}

    fn next_timer_expiration(&self) -> Option<Duration> {
        let mut min: Option<Duration> = None;
       for timer in self.timers.iter() {
        let t =  timer.as_ref().borrow();
        if  t.active && !t.expired() {
            min = Some(t.delta());
        }
       }
         min     
    }

    fn execute_all_active_and_expired_timers(&mut self) {
        let now = Instant::now();
        for timer in self.timers.iter() {
            let mut t =  timer.borrow_mut();
            if  t.active && !t.expired() {
                t.callback.as_ref().unwrap().lock().unwrap()();
                if  t.repeat {
                    t.restart();
                } else {
                    t.stop();
                }
            }
        }
    }

    fn wait_for_callback_on_receiver_channel(&mut self) {
        let timeout = self.next_timer_expiration();
        match self
            .receiver
            .recv_timeout(timeout.unwrap_or(Duration::from_secs(1)))
        {
            Ok(ThreadCommand::WakeUp) => {
                self.execute_all_active_and_expired_timers();
            }
            Ok(ThreadCommand::Terminate) => {
                // Terminate the thread
                self.please_stop = true;
            }
            Ok(ThreadCommand::Execute(callback)) => {
                callback.lock().unwrap()();
            }
            Err(_) => {
                self.execute_all_active_and_expired_timers();
            }
        }
    }

    pub fn run(&mut self) {
        info!("Thread::run");
        loop {
            self.wait_for_callback_on_receiver_channel();
            if self.please_stop {
                break;
            };
        }
    }
}


