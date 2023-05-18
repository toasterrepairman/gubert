use rust_bert::gpt_neo::{GptNeoConfigResources, GptNeoMergesResources, GptNeoModelResources, GptNeoVocabResources};
use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::text_generation::{TextGenerationConfig, TextGenerationModel};
use rust_bert::resources::{LocalResource, RemoteResource};
use tch::Device;

use anyhow::Result;
use std::path::PathBuf;
use tokio::runtime::Runtime;
use std::rc::Rc;

pub async fn gptneo_generate() -> Result<String, anyhow::Error> {
    // Resources paths

    let config_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoConfigResources::GPT_NEO_125M,
    ));
    let vocab_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoVocabResources::GPT_NEO_125M,
    ));
    let merges_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoMergesResources::GPT_NEO_125M,
    ));
    let model_resource = Box::new(RemoteResource::from_pretrained(
        GptNeoModelResources::GPT_NEO_125M,
    ));

    // Set-up model

    let generation_config = TextGenerationConfig {
        model_type: ModelType::GPTNeo,
        model_resource,
        config_resource,
        vocab_resource,
        merges_resource: Some(merges_resource),
        min_length: 10,
        max_length: Some(32),
        do_sample: false,
        early_stopping: true,
        num_beams: 1,
        num_return_sequences: 1,
        device: Device::cuda_if_available(),
        ..Default::default()
    };

    let model = TextGenerationModel::new(generation_config)?;

    // Generate text

    let prompts = [
        "It was a very nice and sunny",
        "It was a gloom winter night, and",
    ];
    let output = model.generate(&prompts, None);

    let mut response = "";

    for sentence in output {
        let response = format!("{}{}", response, sentence);
    }

    Ok(response.to_string())
}

/*
fn main() {
    let mut runtime = Runtime::new().unwrap();
    runtime.block_on(generate_text()).unwrap();
}
*/
