fn main() {
    let _a = Dropable{name: "a"};

    {
        let _b = Dropable{name: "b"};

        {
            let _c = Dropable{name: "c"};
            let _d = Dropable{name: "d"};

            println!("Exiting block B");
        }
        println!("Just exited block B");

        println!("Exiting block A");
    }
    println!("Just exited block A");

    drop(_a);

    println!("end of the main function");
}

struct Dropable {
    name: & 'static str,
}

impl Drop for Dropable {
    fn drop(&mut self) {
        println!("> Dropping {}", self.name)
    }
}