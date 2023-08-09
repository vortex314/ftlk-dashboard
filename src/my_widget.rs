
enum MyEvents {
    Publish {
        topic: String,
        message: String,
    },
    Stop,
}

struct MyWidget{
    rx: mpsc::Receiver<MyEvents>,
    tx: mpsc::Sender<MyEvents>,
    thread: Option<Thread>,
    alive: bool,
    time_last: Instant,
    duration: Duration,
    pattern: String,
    topic: String,
    message: String,
    entry_list: EntryList,
    table: Table,
    entry_count: usize,
    row: usize,
}

impl MyWidget {

    fn new() -> Self {
        tokio::spawn( async move {
            let (tx, mut rx) = mpsc::channel::<MyEvents>(100);
            let mut widget = MyWidget::new();
            widget.rx = rx;
            widget.tx = tx;
            widget.thread = Some(thread::spawn(move || {
                let mut duration: Duration;
                const MAX_TIME: std::time::Duration = std::time::Duration::from_secs(10);
                let mut _time_last = std::time::Instant::now();
                let mut _alive: bool;
                loop {
                    if _time_last.elapsed() > MAX_TIME {
                        _alive = false;
                        duration = Duration::from_millis(1000);
                    } else {
                        _alive = true;
                        duration = MAX_TIME - _time_last.elapsed()
                    }
                    let event = time::timeout(duration, rx.recv()).await;
                    match event {
                        Ok(Ok(MyEvents::Publish { topic, message })) => {
                            if topic.starts_with(pattern) {
                                _time_last = std::time::Instant::now();
                                info!(
                                    "Widget pattern : {} topic: {}, message: {}",
                                    pattern, topic, message
                                );
                            }
                        }
                        Ok(Ok(MyEvents::Stop)) => {
                            info!("Widget stop");
                            break;
                        }
                        Ok(Err(e)) => {
                            error!("Error receiving: {}", e);
                        }
                        Err(e) => {
                            error!("Error receiving: {}", e);
                        }
                    }
                }
            }));
            widget
        }
        Self{}
    }
    fn on_message(& self, topic: &str, message: &str) {
        self.tx.send(MyEvents::Publish {
            topic: topic.to_string(),
            message: message.to_string(),
        });
    }

    fn handle_events(&mut self) {
        let event = time::timeout(self.duration, self.rx.recv()).await;
        match event {
            Ok(Ok(MyEvents::Publish { topic, message })) => {
                if topic.starts_with(pattern) {
                    self.time_last = std::time::Instant::now();
                    info!(
                        "Widget pattern : {} topic: {}, message: {}",
                        pattern, topic, message
                    );
                }
            }
            Ok(Ok(MyEvents::Stop)) => {
                info!("Widget stop");
                break;
            }
            Ok(Err(e)) => {
                error!("Error receiving: {}", e);
            }
            Err(e) => {
                error!("Error receiving: {}", e);
            }
        }
    }
    fn draw(&mut self) {
        
    }
}