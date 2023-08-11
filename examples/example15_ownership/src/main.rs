use std::fmt;
fn main() {
    normal();

    println!("===Borrow===================");
    borrow();
}

fn normal() {
    let immutable_box = Box::new(5u32);
    println!("immutable_box contains {}", immutable_box);
    // *immutable_box = 4;

    let mut mutable_box = immutable_box;

    *mutable_box = 4;
    println!("mutable_box now contains {}", mutable_box);

}

fn part() {
    #[derive(Debug)]
    struct Person {
        name: String,
        age: u8,
    }

    let person = Person {
        name: String::from("Alice"),
        age: 20,
    };

    let Person{name, ref age} = person;

    println!("The person's age is {}", age);
    println!("The person's name is {}", name);

    // println!("Ther person struct is {:?}", person);

    println!("The persion's age from person struct is {}", person.age)
}

struct Point {x : i32, y : i32, z: i32}
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
fn borrow() {
    let mut point = Point{x:0, y: 1, z: 2};

    let borrowed_point = &point;
    let another_borrow = &point;

    println!("Point has coordinates: {}", borrowed_point);
    println!("Point has coordinates: {}", another_borrow);

    let mutable_borrow = &mut point;

    mutable_borrow.x = 5;
    mutable_borrow.y = 6;
    mutable_borrow.z = 1;

    println!("Point has coordinates: {}", mutable_borrow);
    mutable_borrow.x = 1;
    println!("Point has coordinates: {}", mutable_borrow);

    let new_borrowed_point = &point;
    println!("Point now has coordinates: {}", new_borrowed_point);
    // println!("Point has coordinates: {}", mutable_borrow);
}