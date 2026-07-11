use tdi_core::{Action, State};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let initial = State::new(0b0101, 4)?;
    let perturbed = Action::Flip { node: 1 }.apply(initial)?;

    println!("TDI-1 core initialized");
    println!("initial   : {initial}");
    println!("perturbed : {perturbed}");

    Ok(())
}
