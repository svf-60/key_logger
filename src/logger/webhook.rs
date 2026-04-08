use std::{sync::mpsc, thread};

use serde::Serialize;

const WEBHOOK_URL: &'static str = "";

#[derive(Serialize)]
struct WebhookBody {
    embeds: Vec<Embed>,
}

#[derive(Serialize)]
struct Embed {
    title: String,
    description: String,
}

pub struct WebhookSender {
    sender: mpsc::Sender<String>,
}

impl WebhookSender {
    pub fn new() -> Self {
        let (sender, reciever) = mpsc::channel::<String>();

        thread::spawn(move || {
            let agent = ureq::agent();

            while let Ok(mut text) = reciever.recv() {
                text.retain(|c| c != '\r' && c != '\u{7f}' && c != '\x08');

                if text.is_empty() {
                    continue;
                }

                let body = WebhookBody {
                    embeds: vec![Embed {
                        title: "Test".to_string(),
                        description: text,
                    }],
                };

                if let Err(err) = agent
                    .post(WEBHOOK_URL)
                    .header("Content-Type", "application/json")
                    .send_json(&body)
                {
                    eprintln!("webhook failed: {err}");
                }
            }
        });

        Self { sender }
    }

    pub fn post(&self, text: &String) {
        let _ = self.sender.send(text.to_string());
    }
}
