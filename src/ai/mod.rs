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

    /// Checks if a node's title or text content suggests it's a negative prompt.
    fn is_negative_prompt_node(node: &Value) -> bool {
        // Check _meta title
        if let Some(meta) = node.get("_meta") {
            if let Some(title) = meta.get("title").and_then(|v| v.as_str()) {
                let title_lower = title.to_lowercase();
                if title_lower.contains("negative") {
                    return true;
                }
            }
        }
        // Check the text content for negative prompt indicators
        if let Some(inputs) = node.get("inputs") {
            if let Some(text) = inputs.get("text").and_then(|v| v.as_str()) {
                let text_lower = text.to_lowercase();
                if text_lower.contains("bad") || text_lower.contains("ugly")
                    || text_lower.contains("worst") || text_lower.contains("deformed")
                    || text_lower.contains("blurry") || text_lower.contains("low quality")
                {
                    return true;
                }
            }
        }
        false
    }

    /// Injects a text prompt into an API-format workflow by finding prompt-related nodes.
    /// Injects into PrimitiveStringMultiline first (as it feeds CLIPTextEncode),
    /// then falls back to CLIPTextEncode nodes directly.
    fn inject_prompt(api_workflow: &mut Value, prompt_text: &str) {
        let mut injected = false;

        if let Some(obj) = api_workflow.as_object_mut() {
            // Collect node IDs sorted for deterministic order
            let mut node_ids: Vec<String> = obj.keys().cloned().collect();
            node_ids.sort_by(|a, b| {
                let a_num: i64 = a.parse().unwrap_or(i64::MAX);
                let b_num: i64 = b.parse().unwrap_or(i64::MAX);
                a_num.cmp(&b_num)
            });

            // First pass: PrimitiveStringMultiline / PrimitiveString nodes (these feed into CLIPTextEncode)
            for node_id in &node_ids {
                if let Some(node) = obj.get_mut(node_id.as_str()) {
                    if let Some(class_type) = node.get("class_type").and_then(|v| v.as_str()) {
                        if class_type == "PrimitiveStringMultiline" || class_type == "PrimitiveString" {
                            if let Some(inputs) = node.get_mut("inputs") {
                                if let Some(m) = inputs.as_object_mut() {
                                    println!("Injecting prompt into node {} ({})", node_id, class_type);
                                    m.insert("value".to_string(), serde_json::json!(prompt_text));
                                    injected = true;
                                }
                            }
                        }
                    }
                }
            }

            // If we injected into a Primitive node, it feeds CLIPTextEncode via links,
            // so we're done.
            if injected {
                println!("Prompt injected via PrimitiveString node(s)");
                return;
            }

            // Second pass: CLIPTextEncode nodes directly (skip negative prompts)
            for node_id in &node_ids {
                if let Some(node) = obj.get(node_id.as_str()) {
                    if let Some(class_type) = node.get("class_type").and_then(|v| v.as_str()) {
                        if class_type == "CLIPTextEncode" {
                            if Self::is_negative_prompt_node(node) {
                                println!("Skipping negative prompt node {}", node_id);
                                continue;
                            }
                            // Check if text input is a link (array) — if so, it's fed by another node
                            if let Some(inputs) = node.get("inputs") {
                                if let Some(text_val) = inputs.get("text") {
                                    if text_val.is_array() {
                                        println!("Node {} text is a link, skipping direct injection", node_id);
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
                // Now mutably borrow to inject
                if let Some(node) = obj.get_mut(node_id.as_str()) {
                    if let Some(class_type) = node.get("class_type").and_then(|v| v.as_str()).map(|s| s.to_string()) {
                        if class_type == "CLIPTextEncode" {
                            if let Some(inputs) = node.get_mut("inputs") {
                                if let Some(text_val) = inputs.get("text") {
                                    if text_val.is_array() {
                                        continue;
                                    }
                                }
                                if let Some(m) = inputs.as_object_mut() {
                                    println!("Injecting prompt into node {} (CLIPTextEncode)", node_id);
                                    m.insert("text".to_string(), serde_json::json!(prompt_text));
                                    injected = true;
                                }
                            }
                        }
                    }
                }
            }

            if !injected {
                println!("WARNING: Could not find any node to inject the prompt into!");
                println!("Available nodes:");
                for node_id in &node_ids {
                    if let Some(node) = obj.get(node_id.as_str()) {
                        if let Some(ct) = node.get("class_type").and_then(|v| v.as_str()) {
                            println!("  Node {}: {}", node_id, ct);
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

