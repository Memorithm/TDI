//! Bounded real-library worker for the shared adapter SDK.
//!
//! `cargo run -p tdi-bench --example engine_adapter_worker -- --adapter finite --steps 4 --seed 3 --plan-id <64 lowercase hex>`
//! Optional `--restore` accepts the previous plan-bound checkpoint string.
//! Output is one JSON object; rejected input/library failures exit 21. No files,
//! protected models, global RNG, network or scientific verdict are involved.

use tdi_ai::adapter_sdk::ReplayCodec;
use tdi_ai::experiment::StepContext;
use tdi_bench::engine_adapters::{AdapterError, FiniteCycle, JacobiSweep};

fn run<A: ReplayCodec<Error = AdapterError>>(
    mut adapter: A,
    steps: usize,
    plan: &str,
    seed: u64,
    restore: Option<&str>,
    observation: impl Fn(A::Observation) -> Vec<f64>,
) -> Result<(Vec<Vec<f64>>, String, usize), String> {
    let prefix = format!("{plan}{seed:016x}");
    if let Some(encoded) = restore {
        if encoded.len() < prefix.len()
            || !encoded.starts_with(&prefix)
            || encoded.len() > prefix.len() + 2 * adapter.contract().max_checkpoint_bytes
        {
            return Err("checkpoint plan/size mismatch".into());
        }
        let encoded = &encoded[prefix.len()..];
        if encoded.len() % 2 != 0
            || !encoded
                .bytes()
                .all(|x| x.is_ascii_digit() || (b'a'..=b'f').contains(&x))
        {
            return Err("invalid checkpoint encoding".into());
        }
        let bytes = (0..encoded.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&encoded[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let checkpoint = adapter
            .decode_checkpoint(&bytes)
            .map_err(|x| format!("{x:?}"))?;
        adapter = adapter.fork(&checkpoint).map_err(|x| format!("{x:?}"))?;
    }
    if steps > adapter.contract().max_steps - adapter.progress() {
        return Err("step budget exceeded".into());
    }
    let mut values = Vec::with_capacity(steps);
    for _ in 0..steps {
        let value = observation(
            adapter
                .advance(StepContext {
                    depth: adapter.progress() + 1,
                    noise_stream: 0,
                })
                .map_err(|x| format!("{x:?}"))?,
        );
        if value.iter().any(|x| !x.is_finite()) {
            return Err("non-finite observation".into());
        }
        values.push(value);
    }
    let mut encoded = prefix;
    for byte in adapter.encode_checkpoint().map_err(|x| format!("{x:?}"))? {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}").map_err(|x| x.to_string())?;
    }
    Ok((values, encoded, adapter.progress()))
}

fn main() {
    if let Err(error) = main_result() {
        // Error strings are generated internally, never interpolated input.
        eprintln!("adapter-error: {error}");
        println!("{{\"schema\":1,\"status\":\"Rejected\"}}");
        std::process::exit(21);
    }
}

fn main_result() -> Result<(), String> {
    let mut options = std::collections::BTreeMap::new();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() > 10 || args.len() % 2 != 0 {
        return Err("invalid argument count".into());
    }
    for pair in args.chunks_exact(2) {
        if !["--adapter", "--steps", "--seed", "--plan-id", "--restore"].contains(&pair[0].as_str())
            || options.insert(pair[0].as_str(), pair[1].as_str()).is_some()
        {
            return Err("unknown/duplicate argument".into());
        }
    }
    let plan = *options.get("--plan-id").ok_or("plan required")?;
    if plan.len() != 64
        || !plan
            .bytes()
            .all(|x| x.is_ascii_digit() || (b'a'..=b'f').contains(&x))
    {
        return Err("invalid plan identity".into());
    }
    let seed: u64 = options
        .get("--seed")
        .ok_or("seed required")?
        .parse()
        .map_err(|_| "invalid seed")?;
    let steps: usize = options
        .get("--steps")
        .ok_or("steps required")?
        .parse()
        .map_err(|_| "invalid steps")?;
    let name = *options.get("--adapter").ok_or("adapter required")?;
    let restore = options.get("--restore").copied();
    let (values, checkpoint, depth) = match name {
        "finite" => run(
            FiniteCycle::new(seed).map_err(|_| "finite generation failed")?,
            steps,
            plan,
            seed,
            restore,
            |x| vec![x as f64],
        )?,
        "jacobi" => run(
            JacobiSweep::new(vec![4.0, 4.0], vec![1.0], 0.0)
                .map_err(|_| "Jacobi generation failed")?,
            steps,
            plan,
            seed,
            restore,
            |x| x,
        )?,
        _ => return Err("unknown adapter".into()),
    };
    println!(
        "{{\"schema\":1,\"status\":\"Evaluated\",\"adapter\":\"{name}\",\"plan_id\":\"{plan}\",\"seed\":\"{seed}\",\"completed_depth\":{depth},\"observations\":{values:?},\"checkpoint\":\"{checkpoint}\"}}"
    );
    Ok(())
}
