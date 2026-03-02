use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
struct ComfyUIPrompt {
    #[serde(rename = "3")]
    text_encode: TextEncode,
    #[serde(rename = "4")]
    k_sampler: KSampler,
    #[serde(rename = "7")]
    vae_decode: VaeDecode,
    #[serde(rename = "6")]
    checkpoint_loader: CheckpointLoader,
    #[serde(rename = "5")]
    empty_latent: EmptyLatent,
    #[serde(rename = "8")]
    save_image: SaveImage,
}

#[derive(Serialize)]
struct TextEncode {
    inputs: TextEncodeInputs,
    class_type: String,
}

#[derive(Serialize)]
struct TextEncodeInputs {
    text: String,
    clip: Value,
}

#[derive(Serialize)]
struct KSampler {
    inputs: KSamplerInputs,
    class_type: String,
}

#[derive(Serialize)]
struct KSamplerInputs {
    seed: i64,
    steps: i32,
    cfg: f32,
    sampler_name: String,
    scheduler: String,
    denoise: f32,
    model: Value,
    positive: Value,
    negative: Value,
    latent_image: Value,
}

#[derive(Serialize)]
struct VaeDecode {
    inputs: VaeDecodeInputs,
    class_type: String,
}

#[derive(Serialize)]
struct VaeDecodeInputs {
    samples: Value,
    vae: Value,
}

#[derive(Serialize)]
struct CheckpointLoader {
    inputs: CheckpointLoaderInputs,
    class_type: String,
}

#[derive(Serialize)]
struct CheckpointLoaderInputs {
    config_name: String,
    ckpt_name: String,
}

#[derive(Serialize)]
struct EmptyLatent {
    inputs: EmptyLatentInputs,
    class_type: String,
}

#[derive(Serialize)]
struct EmptyLatentInputs {
    width: i32,
    height: i32,
    batch_size: i32,
}

#[derive(Serialize)]
struct SaveImage {
    inputs: SaveImageInputs,
    class_type: String,
}

#[derive(Serialize)]
struct SaveImageInputs {
    filename_prefix: String,
    images: Value,
}

#[derive(Deserialize)]
struct PromptResponse {
    prompt_id: String,
}

#[derive(Deserialize)]
struct HistoryResponse {
    #[serde(flatten)]
    history: std::collections::HashMap<String, PromptHistory>,
}

#[derive(Deserialize)]
struct PromptHistory {
    outputs: std::collections::HashMap<String, Output>,
}

#[derive(Deserialize)]
struct Output {
    images: Vec<ImageInfo>,
}

#[derive(Deserialize)]
struct ImageInfo {
    filename: String,
    subfolder: String,
    #[serde(rename = "type")]
    image_type: String,
}

pub struct AssetGenerator {
    client: Client,
    url: String,
}

impl AssetGenerator {
    pub fn new() -> Self {
        let client = Client::new();
        let url = "http://127.0.0.1:8188".to_string(); // ComfyUI API endpoint
        AssetGenerator { client, url }
    }

    pub async fn generate_sprite(&mut self, prompt: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Simple workflow for generating a 64x64 sprite
        let workflow = ComfyUIPrompt {
            text_encode: TextEncode {
                inputs: TextEncodeInputs {
                    text: prompt.to_string(),
                    clip: serde_json::json!(["6", 1]),
                },
                class_type: "CLIPTextEncode".to_string(),
            },
            k_sampler: KSampler {
                inputs: KSamplerInputs {
                    seed: rand::random::<i64>(),
                    steps: 20,
                    cfg: 8.0,
                    sampler_name: "euler".to_string(),
                    scheduler: "normal".to_string(),
                    denoise: 1.0,
                    model: serde_json::json!(["6", 0]),
                    positive: serde_json::json!(["3", 0]),
                    negative: serde_json::json!(["3", 1]),
                    latent_image: serde_json::json!(["5", 0]),
                },
                class_type: "KSampler".to_string(),
            },
            vae_decode: VaeDecode {
                inputs: VaeDecodeInputs {
                    samples: serde_json::json!(["4", 0]),
                    vae: serde_json::json!(["6", 2]),
                },
                class_type: "VAEDecode".to_string(),
            },
            checkpoint_loader: CheckpointLoader {
                inputs: CheckpointLoaderInputs {
                    config_name: "v1-inference.yaml".to_string(),
                    ckpt_name: "v1-5-pruned-emaonly.ckpt".to_string(), // Adjust to your model
                },
                class_type: "CheckpointLoaderSimple".to_string(),
            },
            empty_latent: EmptyLatent {
                inputs: EmptyLatentInputs {
                    width: 64,
                    height: 64,
                    batch_size: 1,
                },
                class_type: "EmptyLatentImage".to_string(),
            },
            save_image: SaveImage {
                inputs: SaveImageInputs {
                    filename_prefix: "sprite".to_string(),
                    images: serde_json::json!(["7", 0]),
                },
                class_type: "SaveImage".to_string(),
            },
        };

        let prompt_response: PromptResponse = self.client
            .post(&format!("{}/prompt", self.url))
            .json(&workflow)
            .send()
            .await?
            .json()
            .await?;

        let prompt_id = prompt_response.prompt_id;

        // Poll for completion
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let history: HistoryResponse = self.client
                .get(&format!("{}/history", self.url))
                .send()
                .await?
                .json()
                .await?;

            if let Some(prompt_history) = history.history.get(&prompt_id) {
                if let Some(output) = prompt_history.outputs.get("8") {
                    if let Some(image_info) = output.images.first() {
                        let image_url = format!("{}/view?filename={}&subfolder={}&type={}",
                            self.url, image_info.filename, image_info.subfolder, image_info.image_type);
                        let image_bytes = self.client
                            .get(&image_url)
                            .send()
                            .await?
                            .bytes()
                            .await?;
                        return Ok(image_bytes.to_vec());
                    }
                }
            }
        }
    }
}
