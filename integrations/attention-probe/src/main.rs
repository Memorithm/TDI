//! One bounded JSON request on stdin, one versioned response on stdout.
use std::io::{self, Read};
use tdi_attention_probe::{parse, MAX_BYTES};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = (|| {
        let (run, allow_cuda) = match args
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["validate"] => (false, false),
            ["run"] => (true, false),
            ["run", "--allow-cuda"] => (true, true),
            _ => return Err("usage: tdi-attention-probe validate | run [--allow-cuda]".to_string()),
        };
        let mut raw = Vec::new();
        io::stdin()
            .take((MAX_BYTES + 1) as u64)
            .read_to_end(&mut raw)
            .map_err(|e| e.to_string())?;
        let request = parse(&raw)?;
        if run {
            request.run(allow_cuda)
        } else {
            Ok(serde_json::json!({"schema":1,"status":"validated","contract":request.contract()?}))
        }
    })();
    match result {
        Ok(value) => println!("{value}"),
        Err(error) => {
            println!(
                "{}",
                serde_json::json!({"schema":1,"status":"rejected","error":error})
            );
            std::process::exit(21);
        }
    }
}
