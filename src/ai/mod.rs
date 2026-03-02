use reqwest::Client;
use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Serialize)]
struct LMStudioRequest {
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: i32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct LMStudioResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

pub struct LLMEngine {
    client: Client,
    url: String,
}

impl LLMEngine {
    pub fn new() -> Self {
        let client = Client::new();
        let url = "http://localhost:1234/v1/chat/completions".to_string(); // LM Studio API endpoint
        LLMEngine { client, url }
    }

    pub async fn process(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = "Generate a random color for a 2D sprite (e.g., red, blue, green). Respond with just the color name.";
        let request_body = LMStudioRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.7,
            max_tokens: 10,
        };

        let response = self.client
            .post(&self.url)
            .json(&request_body)
            .send()
            .await?;

        let response_data: LMStudioResponse = response.json().await?;
        if let Some(choice) = response_data.choices.first() {
            Ok(choice.message.content.trim().to_string())
        } else {
            Err("No response from LM Studio".into())
        }
    }

    pub fn parse_color(description: &str) -> [f32; 4] {
        let mut rng = rand::thread_rng();
        match description.to_lowercase().as_str() {
            "red" => [1.0, 0.0, 0.0, 1.0],
            "blue" => [0.0, 0.0, 1.0, 1.0],
            "green" => [0.0, 1.0, 0.0, 1.0],
            _ => [rng.gen(), rng.gen(), rng.gen(), 1.0],
        }
    }
}
