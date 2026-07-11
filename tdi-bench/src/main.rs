use tdi_core::{Action, State, TableSystem, TdiSignature, explore};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let zero = State::new(0b00, 2)?;
    let one = State::new(0b01, 2)?;
    let two = State::new(0b10, 2)?;
    let three = State::new(0b11, 2)?;

    let mut system = TableSystem::new(2)
        .map_err(|error| format!("cannot create transition system: {error:?}"))?;

    system
        .insert(zero, Action::Noop, vec![one, two])
        .map_err(|error| format!("cannot insert transition: {error:?}"))?;
    system
        .insert(one, Action::Noop, vec![three])
        .map_err(|error| format!("cannot insert transition: {error:?}"))?;
    system
        .insert(two, Action::Noop, vec![zero])
        .map_err(|error| format!("cannot insert transition: {error:?}"))?;

    let report = explore(&system, zero, &[Action::Noop, Action::Noop])
        .map_err(|error| format!("exploration failed: {error:?}"))?;

    let signature = TdiSignature::from_report(&report)
        .map_err(|error| format!("signature failed: {error:?}"))?;

    println!("TDI-1 exact prospective signature");
    println!("reachable profile : {:?}", signature.reachable_profile());
    println!("path profile      : {:?}", signature.path_profile());

    let returns: Vec<String> = signature
        .return_profile()
        .iter()
        .map(|ratio| format!("{}/{}", ratio.numerator(), ratio.denominator()))
        .collect();

    println!("return profile    : {returns:?}");

    Ok(())
}
