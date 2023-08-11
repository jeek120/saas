use std::fmt;

fn main() {
    normal();
    debug();
    println!("== Dispaly function ==================");
    diaplay();

    return;
}

struct List(Vec<i32>);
impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vec = &self.0;
        write!(f, "[")?;
        for (i, v) in vec.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", v);
        }
        write!(f, "]")
    }
}

fn diaplay() {
    let minmax = MinMax(0, 14);
    println!("Dispaly: {}", minmax);
    println!("Debug: {:?}", minmax);

    let big_range = MinMax(-300, 300);
    let small_range = MinMax(-3, 3);

    println!(
        "The big range is {big} and the small is {small}",
        big = big_range,
        small = small_range,
    );

    let point = Point2D { x: 3.3, y: 7.2 };
    println!("Compare points:");
    println!("Display: {}", point);
    println!("Debug: {:?}", point);

    let v = List(vec![1, 2, 3]);
    println!("{}", v);
}

#[derive(Debug)]
struct MinMax(i64, i64);

impl fmt::Display for MinMax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[derive(Debug)]
struct Point2D {
    x: f64,
    y: f64,
}

impl fmt::Display for Point2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "x:{}, y:{}", self.x, self.y)
    }
}

fn debug() {
    // println!("{:}", UnPrintable(31));
    // println!("{:?}", UnPrintable(31));
    // println!("{:}", Structure(31));
    println!("Now {:?} will print!", Structure(3));
    println!("Now {:?} will print!", Deep(Structure(3)));

    let name = "Peter";
    let age = 27;
    let peter = Person { name, age };
    // 带有换行的格式化输出
    println!("{:#?}", peter);
}

fn normal() {
    println!("Hello, world!");

    println!("{} days", 31);

    // 位置参数
    println!("{0}, this is {1}. {1}, this is {0}", "Alice", "Bob");

    // 可以使用命名参数
    println!(
        "{subject} {verb} {object}",
        object = "the lazy dog",
        subject = "the quick brown fox",
        verb = "jumps over"
    );

    // 可以在 :后指定格式
    println!("{} of {:b} people know binary, the other half don't", 1, 2);

    // 可以指定宽度
    println!("{number:>width$}", number = 1, width = 6);

    println!("{number:>0width$}", number = 1, width = 6);
}

struct UnPrintable(i32);

#[derive(Debug)]
struct Structure(i32);

#[derive(Debug)]
struct Deep(Structure);

#[derive(Debug)]
struct Person<'a> {
    name: &'a str,
    age: u8,
}
