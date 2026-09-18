//! Trusted finite-state implementation choices for the generic search fixture.
//!
//! Input contains only initial state and horizon, never an expected answer,
//! verifier identity or fitness. Measurement belongs to the external evaluator.
//! `table` calls the actual tdi-core table; `modular` computes the same cycle;
//! `incorrect` is an intentional negative control that must never be measured.

use std::collections::BTreeMap;
use tdi_core::{Action, State, TableSystem, TransitionSystem};

fn run() -> Result<(), String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.len() != 6 {
        return Err("expected implementation, start and steps".into());
    }
    let mut args = BTreeMap::new();
    for pair in raw.chunks_exact(2) {
        if !matches!(pair[0].as_str(), "--implementation" | "--start" | "--steps")
            || args.insert(pair[0].as_str(), pair[1].as_str()).is_some()
        {
            return Err("unknown or duplicate argument".into());
        }
    }
    let start: u64 = args["--start"]
        .parse()
        .map_err(|_| "invalid initial state")?;
    let steps: u64 = args["--steps"].parse().map_err(|_| "invalid horizon")?;
    if start > 3 || steps > 65536 {
        return Err("finite fixture bounds exceeded".into());
    }
    let state = match args["--implementation"] {
        "table" => {
            let mut table = TableSystem::new(2).map_err(|_| "table initialization")?;
            for i in 0..4 {
                table
                    .insert(
                        State::new(i, 2).unwrap(),
                        Action::Noop,
                        vec![State::new((i + 1) % 4, 2).unwrap()],
                    )
                    .map_err(|_| "table transition")?;
            }
            let mut state = State::new(start, 2).unwrap();
            for _ in 0..steps {
                state = table
                    .successors(state, Action::Noop)
                    .map_err(|_| "missing transition")?[0];
            }
            state.bits()
        }
        "modular" => (start + steps) % 4,
        "incorrect" => (start + steps + 1) % 4,
        _ => return Err("unknown implementation".into()),
    };
    println!("{{\"schema\":1,\"state\":{state}}}");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("finite search input: {error}");
        std::process::exit(21);
    }
}
