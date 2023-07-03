use rs_llama_cpp::{gpt_params_c, run_inference, str_to_mut_i8};

pub fn generate(prompt: &str, init: &str, max: u16, sampling: bool, stopping: bool, temp: f32, beams: u8) -> String {
    let mut tokens = String::new();
    let params = gpt_params_c {
        n_threads: 8,
        temp: 0.0,
        use_mlock: true,
        model: str_to_mut_i8("./models/13B/ggml-model.bin"),
        prompt: str_to_mut_i8(&format!("Here is a short greeting message in English: \"{}", prompt)),
        ..Default::default()
    };

    run_inference(params, |x| {
        tokens.push_str(x);

        if x.ends_with("\"") {
            false // stop inference
        } else {
            true // continue inference
        }
    });

    tokens
}


/*
fn main() {
    let mut runtime = Runtime::new().unwrap();
    runtime.block_on(generate_text()).unwrap();
}
*/
