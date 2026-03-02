use lmstudio_client::{Client, Request};

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
        let prompt = "Generate a 2D sprite of a dragon.";
        let request = Request::new(prompt);
        let response = self.client.generate(request)?;
        Ok(response.text)
    }
}
