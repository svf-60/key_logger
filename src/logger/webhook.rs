use serde::Serialize;

const WEBHOOK_URL: &'static str = "https://discord.com/api/webhooks/1491276301904908470/1WlqsWVH6mmVg9geyKKvRlNYBiLkQNdokVQcf_zs01yeq2_2GjEehYhdrY2kPqxkR4e8";

#[derive(Serialize)]
struct Embed {
    title: String,
    description: String,
}

#[derive(Serialize)]
struct WebhookBody {
    embeds: Vec<Embed>,
}

pub fn post_to_webhook(text: String) {
    let body = WebhookBody {
        embeds: vec![Embed {
            title: "Test".to_string(),
            description: text,
        }],
    };

    ureq::post(WEBHOOK_URL)
        .header("Content-Type", "application/json")
        .send_json(&body)
        .unwrap();
}
