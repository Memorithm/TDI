use super::super::types::Error;

pub(crate) fn checked_add(target: &mut u64, amount: u64) -> Result<(), Error> {
    *target = target.checked_add(amount).ok_or(Error::CounterOverflow)?;
    Ok(())
}

pub(crate) fn checked_sum(left: u64, right: u64) -> Result<u64, Error> {
    left.checked_add(right).ok_or(Error::CounterOverflow)
}

pub(crate) fn signed_delta(control: u64, pascal: u64) -> i128 {
    i128::from(control) - i128::from(pascal)
}

pub(crate) fn checksum_update(hash: u64, address: usize, value: u8) -> u64 {
    let mixed = hash ^ (address as u64).rotate_left(17) ^ u64::from(value);
    mixed.wrapping_mul(0x1000_0000_01b3)
}
