use anyhow::Result;

pub async fn llm_generate(model: &str, prompt: &str, init: &str, max: u16, sampling: bool, stopping: bool, temp: f32, beams: u8) -> Result<String, anyhow::Error> {
    let response = "uhm..... cheesed to meet you?";

    Ok(response.to_string())
}