pub type Matrix3 = [[f64; 3]; 3];

pub fn multiply(left: &Matrix3, right: &Matrix3) -> Matrix3 {
    let mut product = [[0.0; 3]; 3];
    for (row, product_row) in product.iter_mut().enumerate() {
        for (column, cell) in product_row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| left[row][k] * right[k][column]).sum();
        }
    }
    product
}

pub fn transform(matrix: &Matrix3, vector: [f64; 3]) -> [f64; 3] {
    matrix.map(|row| row[0] * vector[0] + row[1] * vector[1] + row[2] * vector[2])
}

/// `None` when the matrix is singular or holds a NaN.
pub fn inverse(matrix: &Matrix3) -> Option<Matrix3> {
    let m = matrix;
    let cofactor =
        |r1: usize, c1: usize, r2: usize, c2: usize| m[r1][c1] * m[r2][c2] - m[r1][c2] * m[r2][c1];
    let adjugate = [
        [
            cofactor(1, 1, 2, 2),
            -cofactor(0, 1, 2, 2),
            cofactor(0, 1, 1, 2),
        ],
        [
            -cofactor(1, 0, 2, 2),
            cofactor(0, 0, 2, 2),
            -cofactor(0, 0, 1, 2),
        ],
        [
            cofactor(1, 0, 2, 1),
            -cofactor(0, 0, 2, 1),
            cofactor(0, 0, 1, 1),
        ],
    ];
    let determinant =
        m[0][0] * adjugate[0][0] + m[0][1] * adjugate[1][0] + m[0][2] * adjugate[2][0];
    if determinant.is_nan() || determinant.abs() < f64::EPSILON {
        return None;
    }
    Some(adjugate.map(|row| row.map(|cell| cell / determinant)))
}

/// A matrix as the pixel math and the shader use it.
pub type RgbMatrix = [[f32; 3]; 3];

pub fn to_f32(matrix: &Matrix3) -> RgbMatrix {
    matrix.map(|row| row.map(|cell| cell as f32))
}

pub fn transformed(matrix: &RgbMatrix, [first, second, third]: [f32; 3]) -> [f32; 3] {
    matrix.map(|row| row[0] * first + row[1] * second + row[2] * third)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: Matrix3 = [[2.0, 0.0, 1.0], [1.0, 3.0, 0.0], [0.0, 1.0, 4.0]];

    #[test]
    fn inverse_times_matrix_is_identity() {
        let product = multiply(&inverse(&SAMPLE).unwrap(), &SAMPLE);

        for (row, product_row) in product.iter().enumerate() {
            for (column, cell) in product_row.iter().enumerate() {
                let expected = if row == column { 1.0 } else { 0.0 };
                assert!((cell - expected).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn singular_matrix_has_no_inverse() {
        let singular = [[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [0.0, 1.0, 0.0]];

        assert!(inverse(&singular).is_none());
    }

    #[test]
    fn transform_applies_rows_to_the_vector() {
        assert_eq!(transform(&SAMPLE, [1.0, 2.0, 3.0]), [5.0, 7.0, 14.0]);
    }
}
