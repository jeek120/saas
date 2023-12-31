
macro_rules! say_hello {
    () => {
        println!("Hello!");
    }
}

macro_rules! create_function {
    ($func_name: ident) => {
        fn $func_name() {
            println!("You called {:?}()", stringify!($func_name))
        }
    };
}

macro_rules! print_result {
    // 此宏接收一个`expr`类型的表达式，并将它作为字符串参数，连同其结果一起输出
    // `expr` 指示符表示表达式
    ($expression: expr) => {
        println!("{:?} = {:?}", stringify!($expression), $expression)
    };
}

macro_rules! test {
    ($left:expr; and $right:expr) => {
        println!("{:?} and {:?} is {:?}",
            stringify!($left),
            stringify!($right),
            $left && $right
        )
    };

    ($left:expr; or $right:expr) => {
        println!("{:?} or {:?} is {:?}",
            stringify!($left),
            stringify!($right),
            $left || $right
        )
    };
}

macro_rules! find_min {
    ($x: expr) => { $x};
    ($x: expr, $($y: expr), +) => {
        std::cmp::min($x, find_min!($($y), +))
    }
}


create_function!(foo);
create_function!(bar);

fn main() {
    say_hello!();
    foo();
    bar();

    print_result!(1u32 + 1);

    print_result!({
        let x = 1u32;
        x * x + 2 * x -1
    });

    test!(1i32 + 1 == 2i32; and 2i32 * 2 == 4i32);
    test!(true; or false);

    println!("{}", find_min!(1u32));
    println!("{}", find_min!(1u32+2, 2u32));
    println!("{}", find_min!(5u32, 2u32*3, 4u32));
}
