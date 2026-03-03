use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct PromptResponse {
    prompt_id: String,
}

#[derive(Deserialize)]
struct PromptHistory {
    outputs: std::collections::HashMap<String, OutputNode>,
}

#[derive(Deserialize)]
struct OutputNode {
    images: Option<Vec<ImageInfo>>,
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
        let url = "http://127.0.0.1:8188".to_string();
        AssetGenerator { client, url }
    }

    /// Lists workflow JSON files from the local `workflows/` directory.
    pub async fn list_workflows(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let workflows_dir = std::path::Path::new("workflows");
        if !workflows_dir.exists() {
            std::fs::create_dir_all(workflows_dir)?;
            return Ok(Vec::new());
        }
        let mut names = Vec::new();
        for entry in std::fs::read_dir(workflows_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    names.push(stem.to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    /// Loads a workflow JSON file from the local `workflows/` directory.
    fn load_workflow_from_file(workflow_name: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let mut path = std::path::PathBuf::from("workflows");
        let filename = if workflow_name.ends_with(".json") {
            workflow_name.to_string()
        } else {
            format!("{}.json", workflow_name)
        };
        path.push(&filename);

        if !path.exists() {
            return Err(format!(
                "Workflow file not found: {}. Place ComfyUI API-format JSON files in the 'workflows/' directory.",
                path.display()
            ).into());
        }

        let contents = std::fs::read_to_string(&path)?;
        let workflow: Value = serde_json::from_str(&contents)?;
        Ok(workflow)
    }

    /// Converts a ComfyUI web-format workflow (with nodes array) to API format,
    /// or returns it as-is if it's already in API format.
    fn workflow_to_api_format(workflow: &Value) -> Result<Value, Box<dyn std::error::Error>> {
        if let Some(nodes) = workflow.get("nodes").and_then(|n| n.as_array()) {
            let links = workflow.get("links").and_then(|l| l.as_array());
            let mut api_prompt = serde_json::Map::new();

            let mut link_map: std::collections::HashMap<i64, (i64, i64)> = std::collections::HashMap::new();
            if let Some(links) = links {
                for link in links {
                    if let Some(link_arr) = link.as_array() {
                        if link_arr.len() >= 4 {
                            let link_id = link_arr[0].as_i64().unwrap_or(0);
                            let source_node = link_arr[1].as_i64().unwrap_or(0);
                            let source_slot = link_arr[2].as_i64().unwrap_or(0);
                            link_map.insert(link_id, (source_node, source_slot));
                        }
                    }
                }
            }

            for node in nodes {
                let node_id = node.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
                let class_type = node.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string();

                let mut inputs = serde_json::Map::new();

                // Process node inputs (linked connections)
                if let Some(node_inputs) = node.get("inputs").and_then(|v| v.as_array()) {
                    for input_def in node_inputs {
                        let input_name = input_def.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        if let Some(link_id) = input_def.get("link").and_then(|v| v.as_i64()) {
                            if let Some(&(src_node, src_slot)) = link_map.get(&link_id) {
                                inputs.insert(
                                    input_name.to_string(),
                                    serde_json::json!([src_node.to_string(), src_slot]),
                                );
                            }
                        }
                    }
                }

                // Map widgets_values to the correct input names
                if let Some(widgets) = node.get("widgets_values").and_then(|v| v.as_array()) {
                    let widget_names = get_widget_names_for_class(&class_type);
                    for (i, val) in widgets.iter().enumerate() {
                        if i < widget_names.len() {
                            let name = widget_names[i];
                            if !inputs.contains_key(name) {
                                inputs.insert(name.to_string(), val.clone());
                            }
                        }
                    }
                }

                let mut node_obj = serde_json::Map::new();
                node_obj.insert("class_type".to_string(), serde_json::json!(class_type));
                node_obj.insert("inputs".to_string(), Value::Object(inputs));

                api_prompt.insert(node_id.to_string(), Value::Object(node_obj));
            }

            Ok(Value::Object(api_prompt))
        } else {
            Ok(workflow.clone())
        }
    }

    /// Injects a text prompt into the workflow by finding CLIPTextEncode nodes.
    fn inject_prompt(api_workflow: &mut Value, prompt_text: &str) {
        if let Some(obj) = api_workflow.as_object_mut() {
            for (_node_id, node) in obj.iter_mut() {
                if let Some(class_type) = node.get("class_type").and_then(|v| v.as_str()) {
                    if class_type == "CLIPTextEncode" {
                        if let Some(inputs) = node.get_mut("inputs") {
                            if let Some(text_val) = inputs.get("text") {
                                let current = text_val.as_str().unwrap_or("");
                                if current.contains("bad") || current.contains("ugly")
                                    || current.contains("worst") || current.contains("negative")
                                {
                                    continue;
                                }
                            }
                            inputs.as_object_mut().map(|m| {
                                m.insert("text".to_string(), serde_json::json!(prompt_text));
                            });
                            return;
                        }
                    }
                }
            }
        }
    }

    /// Generate a sprite using a named ComfyUI workflow and a text prompt.
    pub async fn generate_sprite(&mut self, workflow_name: &str, prompt_text: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let workflow = Self::load_workflow_from_file(workflow_name)?;

        let mut api_workflow = Self::workflow_to_api_format(&workflow)?;

        Self::inject_prompt(&mut api_workflow, prompt_text);

        let request_body = serde_json::json!({
            "prompt": api_workflow
        });

        println!("Sending prompt to ComfyUI...");

        let resp = self.client
            .post(&format!("{}/prompt", self.url))
            .json(&request_body)
            .send()
            .await?;

        let status = resp.status();
        let body_text = resp.text().await?;

        if !status.is_success() {
            return Err(format!("ComfyUI returned HTTP {}: {}", status, body_text).into());
        }

        let prompt_response: PromptResponse = serde_json::from_str(&body_text)
            .map_err(|e| format!("Failed to parse prompt response: {} — body: {}", e, body_text))?;

        let prompt_id = prompt_response.prompt_id;
        println!("Prompt queued with ID: {}", prompt_id);

        let max_attempts = 120;
        for attempt in 0..max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            let history_resp = self.client
                .get(&format!("{}/history/{}", self.url, prompt_id))
                .send()
                .await?;

            if !history_resp.status().is_success() {
                continue;
            }

            let history_text = history_resp.text().await?;
            let history: Value = serde_json::from_str(&history_text)?;

            if let Some(prompt_data) = history.get(&prompt_id) {
                let prompt_history: PromptHistory = serde_json::from_value(prompt_data.clone())?;

                for (_node_id, output) in &prompt_history.outputs {
                    if let Some(images) = &output.images {
                        if let Some(image_info) = images.first() {
                            let image_url = format!(
                                "{}/view?filename={}&subfolder={}&type={}",
                                self.url,
                                urlencoding::encode(&image_info.filename),
                                urlencoding::encode(&image_info.subfolder),
                                urlencoding::encode(&image_info.image_type)
                            );
                            let image_bytes = self.client
                                .get(&image_url)
                                .send()
                                .await?
                                .bytes()
                                .await?;
                            println!("Image received: {} bytes", image_bytes.len());
                            return Ok(image_bytes.to_vec());
                        }
                    }
                }
            }

            if attempt % 10 == 0 && attempt > 0 {
                println!("Still waiting for generation... ({}s)", attempt);
            }
        }

        Err("Timed out waiting for ComfyUI to generate the image".into())
    }
}

fn get_widget_names_for_class(class_type: &str) -> Vec<&'static str> {
    match class_type {
        "KSampler" => vec!["seed", "control_after_generate", "steps", "cfg", "sampler_name", "scheduler", "denoise"],
        "KSamplerAdvanced" => vec!["add_noise", "noise_seed", "control_after_generate", "steps", "cfg", "sampler_name", "scheduler", "start_at_step", "end_at_step", "return_with_leftover_noise"],
        "CheckpointLoaderSimple" => vec!["ckpt_name"],
        "CLIPTextEncode" => vec!["text"],
        "EmptyLatentImage" => vec!["width", "height", "batch_size"],
        "SaveImage" => vec!["filename_prefix"],
        "PreviewImage" => vec![],
        "VAEDecode" => vec![],
        "VAEEncode" => vec![],
        "LoraLoader" => vec!["lora_name", "strength_model", "strength_clip"],
        "CLIPSetLastLayer" => vec!["stop_at_clip_layer"],
        "LatentUpscale" => vec!["upscale_method", "width", "height", "crop"],
        "LatentUpscaleBy" => vec!["upscale_method", "scale_by"],
        _ => vec![],
    }
}
