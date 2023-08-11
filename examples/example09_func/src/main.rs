fn main() {
    fn sum_odd_numbers(up_to: u32) -> u32 {
        let mut acc = 0;
        for i in 0..up_to {
            let addition = match i % 2 == 0 {
                true => i,
                false => continue,
            };
            acc = acc + addition
        }
        acc
    }
    println!(
        "Sum of odd number up to 9 (excluding): {}",
        sum_odd_numbers(9)
    );
}
