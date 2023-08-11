fn main() {
    let mut dolly: Sheep = Animal::new("Dolly");

    dolly.talk();
    dolly.shear();
    dolly.talk();
}

struct Sheep {
    naked: bool,
    name: & 'static str
}

trait Animal {
    fn new(name: &'static str) -> Self;
    fn name(&self) -> &'static str;
    fn noise(&self) -> &'static str;

    fn talk(&self) {
        println!("{} say {}", self.name(), self.noise())
    }
}

impl Sheep {
    fn is_naked(&self) -> bool {
        self.naked
    }

    fn shear(&mut self) {
        if self.is_naked() {
            println!("{} is already naked...", self.name())
        } else {
            println!("{} gets a haircut!", self.name)
        }
    }
}

impl Animal for Sheep {
    fn new(name: &'static str) -> Self {
        Sheep { naked: false, name: name }
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn noise(&self) -> &'static str {
        if self.is_naked() {
            "hahahahhaha"
        }else {
            "hehehehehhe"
        }
    }
}