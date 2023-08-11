use std::{marker::PhantomData, ops::Add};

fn main() {
    let one_foot: Length<Inch> = Length(12.0, PhantomData);
    let one_meter: Length<Mm> = Length(1000.0, PhantomData);

    let two_feet = one_foot + one_foot;
    let two_meter = one_meter + one_meter;

    println!("one foot + one foot = {:?} in", two_feet.0);
    println!("one meter + one meter = {:?} mm", two_meter);

    // let one_feter = one_foot + one_meter;
}

#[derive(Debug, Clone, Copy)]
enum Inch {}

#[derive(Debug, Clone, Copy)]
enum Mm {}

#[derive(Debug, Clone, Copy)]
struct Length<Unit>(f64, PhantomData<Unit>);

impl<Unit> Add for Length<Unit> {
    type Output = Length<Unit>;

    fn add(self, rhs: Self) -> Self::Output {
        Length(self.0 + rhs.0, PhantomData)
    }
}
