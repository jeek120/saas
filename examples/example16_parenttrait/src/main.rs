fn main() {
}

struct What {
}

impl CompSciStudent for What {
    fn git_username(&self) -> String {
        String::from("what@gitlab.com")
    }

}

impl Person for What {
    fn name(&self) -> String {
        String::from("what")
    }
}

impl Student for What {
    fn university(&self) -> String {
        String::from("beijing")
    }   
}

impl Programmer for What {
    fn fav_language(&self) -> String {
        String::from("rust")
    }
}

trait Person {
    fn name(&self) -> String;
}

trait Student: Person {
    fn university(&self) -> String;
}

trait Programmer {
    fn fav_language(&self) -> String;
}

trait CompSciStudent: Programmer + Student {
    fn git_username(&self) -> String;
}

fn comp_sci_student_greeting(student: &dyn CompSciStudent) -> String {
    format!(
        "My name is {} and I attend {}. My favorite language is {}. My Git username is {}",
        student.name(),
        student.university(),
        student.fav_language(),
        student.git_username()
    )
}
