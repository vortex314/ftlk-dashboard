use crossbeam::channel::{bounded, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use log::{debug, error, info, trace, warn};

type Callback = Box<dyn Fn() + Send + 'static>;

pub enum ThreadCommand {
    WakeUp,             // Wake up the thread and recalc timers
    Terminate,          // Terminate the thread
    Execute(Callback),  // Execute a callback on the thread
}

pub struct Timer {
    expires_at: Instant,
    repeat: bool,
    active: bool,
    interval: Duration,
    callback: Callback,
}

impl Timer {
    fn new(repeat: bool, interval: Duration, callback: Callback) -> Timer {
        Timer {
            expires_at:Instant::now() + interval,
            repeat,
            active: true,
            interval,
            callback,
        }
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
    timers: Vec<Timer>,
    thread_handle : Option<thread::JoinHandle<()>>,
}

impl Thread {
    pub fn new() -> Thread {
        let (sender, receiver) = bounded(100);
        let thread = Thread {
            receiver,
            sender,
            timers: Vec::new(),
            thread_handle:None,
        };
        thread
    }

    pub fn start(& mut self) {
        let thread_handle = thread::spawn( move || {
                info!("Thread started");
        });
    }

    pub fn send(&self, command: ThreadCommand) {
        self.sender.send(command).expect("Failed to send command");
    }


    pub fn new_timer(&mut self, repeat: bool, interval: Duration, callback: Callback) -> & mut Timer {
        let timer = Timer::new(repeat, interval, callback);
        self.timers.push(timer);
        self.timers.last_mut().unwrap()
    }
    
    fn next_timer_expiration(&self) -> Option<Duration> {
        self.timers.iter().filter(|t| t.active).map(|t| t.delta()).min()
    }

    fn execute_all_active_and_expired_timers(&mut self) {
        let now = Instant::now();
        for timer in self.timers.iter_mut().filter(|t| t.active && t.expired()) {
            (timer.callback)();
            if timer.repeat {
                timer.restart();
            } else {
                timer.stop();
            }
        }
    }

    fn wait_for_callback_on_receiver_channel(& mut self) {
        let timeout = self.next_timer_expiration();
        match self.receiver.recv_timeout(timeout.unwrap_or(Duration::from_secs(1))) {
            Ok(ThreadCommand::WakeUp) => {
                self.execute_all_active_and_expired_timers();
            }
            Ok(ThreadCommand::Terminate) => {
                // Terminate the thread
            }
            Ok(ThreadCommand::Execute(callback)) => {
                callback();
            }
            Err(_) => {
                self.execute_all_active_and_expired_timers();
            }
        }
    }

    fn run(&mut self) {
        loop {
            self.wait_for_callback_on_receiver_channel();
        }
    }
}
/* 
// Define a type for the callback function

enum TimerCommand {
    Set(Instant, Callback),
    Cancel,
}

// Function to run on a separate thread
fn thread_function(receiver: Receiver<TimerCommand>) {
    // Create a priority queue for timer events
    let mut timer_queue = std::collections::BinaryHeap::new();

    // Process events in a loop
    loop {
        // Calculate the timeout for the next event
        let now = Instant::now();
        let next_event = timer_queue.peek().map(|event| event.time.saturating_duration_since(now));

        // Wait for the next event or timeout
        if let Ok(command) = receiver.recv_timeout(next_event.unwrap_or(Duration::from_secs(1))) {
            match command {
                TimerCommand::Set(time, callback) => {
                    // Insert the timer event into the queue
                    timer_queue.push(TimerEvent { time, callback });
                }
                TimerCommand::Cancel => {
                    // Remove the next event from the queue
                    if let Some(event) = timer_queue.pop() {
                        if event.time > Instant::now() {
                            // If the event hasn't fired yet, put it back in the queue
                            timer_queue.push(event);
                        }
                    }
                }
            }
        }

        // Process fired timer events
        while let Some(event) = timer_queue.peek() {
            if event.time <= Instant::now() {
                // Execute the callback
                (event.callback)();

                // Remove the event from the queue
                timer_queue.pop();
            } else {
                break;
            }
        }
    }
}

// Structure to represent a timer event
struct TimerEvent {
    time: Instant,
    callback: Callback,
}

fn main() {
    // Create a channel to communicate between threads
    let (sender, receiver) = bounded::<TimerCommand>(10);

    // Spawn a new thread
    thread::spawn(move || {
        thread_function(receiver);
    });

    // Schedule some timer events
    let now = Instant::now();
    let mut smallest_timeout = Duration::from_secs(u64::max_value());

    for i in 1..=5 {
        let duration = Duration::from_secs(i);
        let callback = Box::new(move || {
            println!("Timer event {} fired after {:?} on thread {:?}", i, duration, thread::current().id());
        });
        let event_time = now + duration;
        let command = TimerCommand::Set(event_time, callback);
        sender.send(command).expect("Failed to send timer event");

        // Update the smallest timeout
        if duration < smallest_timeout {
            smallest_timeout = duration;
        }
    }

    // Sleep for the smallest timeout
    std::thread::sleep(smallest_timeout);

    // Cancel the next timer event
    sender.send(TimerCommand::Cancel).expect("Failed to send cancel command");

    // Sleep again to see the canceled event not firing
    std::thread::sleep(Duration::from_secs(10));

    // Close the channel to signal the thread to exit
    drop(sender);

    // Wait for the thread to finish
    thread::current().join().expect("Failed to join the thread");
}
*/