use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,

    pub deepgram_api_key: String,
    pub deepl_api_key: String,
    pub elevenlabs_api_key: String,
    pub elevenlabs_voice_id: String,

    #[serde(default = "default_true")]
    pub enable_speculation: bool,

    #[serde(default = "default_confidence_threshold")]
    pub speculation_confidence_threshold: f32,

    #[serde(default = "default_min_interval_ms")]
    pub speculation_min_interval_ms: u64,

    #[serde(default = "default_revert_threshold")]
    pub revert_edit_distance_threshold: f32,
}

fn default_port() -> u16 {
    3001
}
fn default_true() -> bool {
    true
}
fn default_confidence_threshold() -> f32 {
    0.6
}
fn default_min_interval_ms() -> u64 {
    200
}
fn default_revert_threshold() -> f32 {
    0.2
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        envy::from_env::<Config>().map_err(Into::into)
    }
}
