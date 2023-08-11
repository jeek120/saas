use std::fmt;

fn main() {
    let from = Matrix(1.1, 1.2, 2.1, 2.2);
    println!("{}after transpose is:{}", &from, transpose(&from))
}

#[derive(Debug)]
struct Matrix(f32, f32, f32, f32);

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\n({x1:>width$} {y1:>width$})\n({x2:>width$} {y2:>width$})\n",
            x1 = self.0,
            y1 = self.1,
            x2 = self.2,
            y2 = self.3,
            width = 8,
        )
    }
}
fn transpose(from: &Matrix) -> Matrix {
    Matrix(from.0, from.2, from.1, from.3)
}
