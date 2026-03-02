use lmstudio_client::{Client, Request};

pub struct LLMEngine {
    client: Client,
}

impl LLMEngine {
    pub fn new() -> Self {
        // Connect to the local LM Studio server (default at http://localhost:1234)
        let client = Client::new("http://localhost:1234")
            .expect("Failed to connect to LM Studio");
        LLMEngine { client }
    }

    pub fn process(&mut self) -> String {
        // Example: generate a 2D sprite description
        let prompt = "Generate a 2D sprite of a dragon.";
        let request = Request::new(prompt);
        match self.client.generate(request) {
            Ok(response) => response.text,
            Err(_) => String::new(),
        }
    }
}
