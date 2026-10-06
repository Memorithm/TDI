use super::{Cell, Error, Schedule, Split, run_cell};

fn run_matrix(split: Split, widths: [u8; 2]) -> Result<Vec<Cell>, Error> {
    let mut cells = Vec::with_capacity(108);
    for n in widths {
        let k = 1usize << n;
        for density in [4usize, 32, 256] {
            for schedule in [
                Schedule::Affine,
                Schedule::LowWeightFirst,
                Schedule::HighWeightFirst,
            ] {
                for query_load in [16usize, 64, k] {
                    for reuse in [1usize, 8] {
                        cells.push(run_cell(split, n, density, schedule, query_load, reuse)?);
                    }
                }
            }
        }
    }
    debug_assert_eq!(cells.len(), 108);
    Ok(cells)
}

pub fn run_development_matrix() -> Result<Vec<Cell>, Error> {
    run_matrix(Split::Development, [9, 11])
}

pub fn run_validation_matrix() -> Result<Vec<Cell>, Error> {
    run_matrix(Split::Validation, [13, 14])
}

pub fn run_pascal_p3_development_matrix() -> Result<Vec<Cell>, Error> {
    run_development_matrix()
}

pub fn run_pascal_p3_validation_matrix() -> Result<Vec<Cell>, Error> {
    run_validation_matrix()
}
