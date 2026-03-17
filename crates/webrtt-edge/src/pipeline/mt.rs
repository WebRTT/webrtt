// DeepL Machine Translation client.
//
// DeepL does not have a streaming API — it returns the full translation.
// Strategy: call DeepL with each partial STT transcript as it arrives.
// Accept that partial-sentence translations may have grammatical errors.
// REVERT corrects these when the final translation arrives.
//
// DeepL API endpoint:
//   POST https://api-free.deepl.com/v2/translate
//   Authorization: DeepL-Auth-Key {api_key}
//   Content-Type: application/json
//   Body: { "text": ["..."], "source_lang": "PT", "target_lang": "EN-US" }

use serde::Deserialize;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TranslationResult {
    pub text: String,
    pub tokens: Vec<String>,
}

#[allow(dead_code)]
pub struct DeepLClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeepLResponse {
    translations: Vec<DeepLTranslation>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeepLTranslation {
    text: String,
}

#[allow(dead_code)]
impl DeepLClient {
    pub fn new(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            base_url: "https://api-free.deepl.com".into(),
        }
    }

    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> anyhow::Result<TranslationResult> {
        let resp = self
            .http
            .post(format!("{}/v2/translate", self.base_url))
            .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
            .json(&serde_json::json!({
                "text": [text],
                "source_lang": source_lang,
                "target_lang": target_lang,
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<DeepLResponse>()
            .await?;

        let translated = resp
            .translations
            .into_iter()
            .next()
            .map(|t| t.text)
            .unwrap_or_default();

        let tokens: Vec<String> = translated.split_whitespace().map(String::from).collect();

        Ok(TranslationResult {
            text: translated,
            tokens,
        })
    }
}
