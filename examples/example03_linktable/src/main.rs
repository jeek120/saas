use std::fmt;

fn main() {
    let mut list1 = List::new();
    list1 = list1.prepend(3);
    list1 = list1.prepend(2);
    list1 = list1.prepend(1);

    println!("linked list has length: {}", list1.len());
    println!("{}", list1.stringify());
}

enum List<T>
where
    T: fmt::Display,
{
    Cons(T, Box<List<T>>),
    Nil,
}

impl<T> List<T>
where
    T: fmt::Display,
{
    fn new() -> List<T> {
        List::Nil
    }

    fn prepend(self, elem: T) -> List<T> {
        List::Cons(elem, Box::new(self))
    }

    fn len(&self) -> u32 {
        match *self {
            List::Cons(_, ref next) => 1 + next.len(),
            List::Nil => 0,
        }
    }

    fn stringify(&self) -> String {
        match *self {
            List::Cons(ref e, ref next) => format!("{}->{}", e, next.stringify()),
            List::Nil => format!("Nil"),
        }
    }
}
