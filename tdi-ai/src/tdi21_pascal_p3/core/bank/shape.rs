use super::super::super::types::Error;

pub(crate) fn domain(n: u8) -> Result<usize, Error> {
    if matches!(n, 9 | 11 | 13 | 14) {
        Ok(1usize << n)
    } else {
        Err(Error::UnsupportedWidth)
    }
}

pub(crate) fn density(d: usize, k: usize) -> Result<(), Error> {
    if d == 0 || d >= k {
        Err(Error::InvalidDensity)
    } else {
        Ok(())
    }
}
