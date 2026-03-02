use lmstudio_client::{Client, Request};
use rand::Rng;

pub struct LLMEngine {
    client: Client,
}

impl LLMEngine {
    pub fn new() -> Self {
        let client = Client::new("http://localhost:1234")
            .expect("Failed to connect to LM Studio");
        LLMEngine { client }
    }

    pub fn process(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = "Generate a random color for a 2D sprite (e.g., red, blue, green). Respond with just the color name.";
        let request = Request::new(prompt);
        let response = self.client.generate(request)?;
        Ok(response.text.trim().to_string())
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
