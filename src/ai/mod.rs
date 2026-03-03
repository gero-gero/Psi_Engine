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

    /// Checks if a JSON value is in ComfyUI API format (object with string node IDs
    /// mapping to objects with "class_type" and "inputs").
    fn is_api_format(workflow: &Value) -> bool {
        if let Some(obj) = workflow.as_object() {
            // API format: keys are node IDs (numeric strings), values have "class_type"
            for (key, val) in obj {
                // Skip non-node keys
                if key == "extra" || key == "version" || key == "config" {
                    continue;
                }
                // Check if this looks like a node definition
                if val.get("class_type").is_some() && val.get("inputs").is_some() {
                    return true;
                }
                // If it has "nodes" array, it's web format
                if key == "nodes" {
                    return false;
                }
            }
        }
        false
    }

    /// Validates that an API-format workflow doesn't contain subgraph UUIDs as class_types.
    fn validate_api_workflow(workflow: &Value) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(obj) = workflow.as_object() {
            for (node_id, node) in obj {
                if let Some(class_type) = node.get("class_type").and_then(|v| v.as_str()) {
                    // UUID pattern check: contains hyphens and is 36 chars (like "7b34ab90-36f9-45ba-a665-71d418f0df18")
                    if class_type.len() == 36 && class_type.chars().filter(|c| *c == '-').count() == 4 {
                        return Err(format!(
                            "Node '{}' has subgraph type '{}'. This workflow uses ComfyUI subgraphs \
                            which cannot be sent directly via the API. Please export the workflow \
                            using 'Save (API Format)' in ComfyUI (enable Dev Mode in settings first).",
                            node_id, class_type
                        ).into());
                    }
                }
            }
        }
        Ok(())
    }

    /// Injects a text prompt into an API-format workflow by finding CLIPTextEncode nodes
    /// or any node with a "text" input.
    fn inject_prompt(api_workflow: &mut Value, prompt_text: &str) {
        if let Some(obj) = api_workflow.as_object_mut() {
            // First pass: look for CLIPTextEncode nodes (positive prompt)
            for (_node_id, node) in obj.iter_mut() {
                if let Some(class_type) = node.get("class_type").and_then(|v| v.as_str()) {
                    if class_type == "CLIPTextEncode" {
                        if let Some(inputs) = node.get_mut("inputs") {
                            if let Some(text_val) = inputs.get("text") {
                                let current = text_val.as_str().unwrap_or("");
                                // Skip negative prompt nodes
                                if current.contains("bad") || current.contains("ugly")
                                    || current.contains("worst") || current.contains("negative")
                                {
                                    continue;
                                }
                            }
                            if let Some(m) = inputs.as_object_mut() {
                                m.insert("text".to_string(), serde_json::json!(prompt_text));
                            }
                            return;
                        }
                    }
                }
            }

            // Second pass: look for PrimitiveStringMultiline or any node with a "value" that's a string
            for (_node_id, node) in obj.iter_mut() {
                if let Some(class_type) = node.get("class_type").and_then(|v| v.as_str()) {
                    if class_type == "PrimitiveStringMultiline" || class_type == "PrimitiveString" {
                        if let Some(inputs) = node.get_mut("inputs") {
                            if let Some(m) = inputs.as_object_mut() {
                                m.insert("value".to_string(), serde_json::json!(prompt_text));
                            }
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

        // Determine if this is API format or web format
        let api_workflow = if Self::is_api_format(&workflow) {
            println!("Workflow is in API format");
            workflow
        } else if workflow.get("nodes").is_some() {
            return Err(
                "This workflow is in ComfyUI web/graph format (not API format). \
                Please re-export it using 'Save (API Format)' in ComfyUI. \
                To enable this option: Settings → Enable Dev Mode Options, \
                then use the 'Save (API Format)' button in the menu."
                .into()
            );
        } else {
            // Try using it as-is (might be API format without class_type in first node)
            println!("Workflow format unclear, attempting to use as API format");
            workflow
        };

        // Validate no subgraph UUIDs
        Self::validate_api_workflow(&api_workflow)?;

        let mut api_workflow = api_workflow;
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

