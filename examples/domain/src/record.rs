
pub enum Type {
    A,
    CNAME,
}
pub struct Record {
    pub t: Type,
    pub host: String,
    pub value: String,
    pub ttl: i32,
}

impl Default for Record {
    fn default() -> Self {
        Self { t: Type::A, host: String::from("@"), value: Default::default(), ttl: 3600 }
    }
}