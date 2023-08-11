pub mod record;
pub mod cloudflare;

pub trait Domain {
    fn SetRecord(&self, record: &record::Record);
}
