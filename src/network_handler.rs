use anyhow::Result;
use reqwest::{self, Client};

pub const MODELS: [&str; 10] = [
    "meta-llama/Llama-3.1-8B-Instruct:fastest",
    "google/gemma-3-27b-it",
    "qwen/qwen3-coder-next",
    "qwen/qwen3.5-397b-a17b",
    "stepfun/step-3.5-flash",
    "mistralai/mistral-7b-instruct-v0.2",
    "mistralai/mixtral-8x7b-instruct",
    "z-ai/glm-5",
    "thedrummer/skyfall-36b-v2",
    "undi95/remm-slerp-l2-13b",
];

#[derive(serde::Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(serde::Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(serde::Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Default)]
pub struct NetworkHandler {
    client: Client,
    api_key: String,
    model_index: usize,
    url: String,
}
pub enum Provider {
    OpenRouter,
    HuggingFace,
}

pub fn get_all_models() -> &'static [&'static str] {
    &MODELS
}

impl NetworkHandler {
    pub fn default() -> Self {
        NetworkHandler {
            client: reqwest::Client::new(),
            api_key: std::env::var("HUGGINGFACE_API_KEY").expect("Set Hugging Face API Token"),
            model_index: 0,
            url: String::from("https://router.huggingface.co/v1/chat/completions"),
        }
    }

    pub fn provider(&mut self, api_provider: Provider) {
        match api_provider {
            Provider::OpenRouter => {
                self.api_key =
                    std::env::var("OPENROUTER_API_KEY").expect("Set Openrouter API Token");
                self.url = String::from("https://openrouter.ai/api/v1/chat/completions");
            }
            Provider::HuggingFace => {
                self.api_key =
                    std::env::var("HUGGINGFACE_API_KEY").expect("Set Openrouter API Token");
                self.url = String::from("https://router.huggingface.co/v1/chat/completions");
            }
        }
    }

    pub fn get_selected_model(&self) -> String {
        String::from(MODELS[self.model_index])
    }

    pub fn set_selected_model(&mut self, index: usize) {
        self.model_index = index;
    }

    pub async fn send_prompt(&self, c_prompt: &str) -> Result<String> {
        if c_prompt.trim().is_empty() {
            return Ok(String::new());
        }

        let chat_request = ChatRequest {
            model: String::from(MODELS[self.model_index]),
            messages: vec![Message {
                role: "user".to_string(),
                content: c_prompt.to_string(),
            }],
        };

        let response = self
            .client
            .post(&self.url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&chat_request)
            .send()
            .await?;
        let status = response.status();

        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read error body>".to_string());
            return Err(anyhow::anyhow!("API error {status}: {body}"));
        }

        let data: ChatResponse = response.json().await?;
        let content = data
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("API response missing choices[0]"))?
            .message
            .content;
        Ok(content)
    }
}
