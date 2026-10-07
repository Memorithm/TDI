use super::super::super::types::Error;

pub(crate) fn lane(gate: usize, density: usize, k: usize) -> Result<Vec<usize>, Error> {
    let mut seen = vec![false; k];
    let mut out = Vec::with_capacity(density);
    for index in 0..density {
        let mask = 1 + ((257 * gate + 71 * index) % (k - 1));
        if seen[mask] {
            return Err(Error::DuplicateGeneratedMask);
        }
        seen[mask] = true;
        out.push(mask);
    }
    Ok(out)
}
