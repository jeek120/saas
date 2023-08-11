use crate::Domain;
use zone::zones;
use hyper::{body::HttpBody as _, Client, Request, Uri, Body};
use std::{concat, error::Error};

pub mod zone;
pub mod resp;

macro_rules! BASE_API {
    () => {"https://api.cloudflare.com/client/v4"}
}
// const BASE_API: & 'static str = "https://api.cloudflare.com/client/v4";

struct auth {
    mail: String,
    apikey: String
}

pub struct cf {
    auth: auth,
}

pub fn new<'a, 'b, 'c>(mail: & 'a str, apikey: & 'b str) -> cf {
    cf { auth: auth { mail: String::from(mail), apikey: String::from(apikey) } }
}

impl cf {
    async fn zones(&self) -> Result<zones, Box<dyn Error + Send + Sync>> {
        let c = Client::new();


        let body:Body = Default::default();
        let resp = c.request(self.req(concat!(BASE_API!() , "/zones"), body)).await?;
        let buf = hyper::body::to_bytes(resp).await?;

        let result = serde_json::from_slice(&buf).unwrap();

        Ok(result)
    }

    fn req<T>(&self, url: &str, body: T) -> Request<T> {
        let mut req = Request::builder().uri(url).header("X-Auth-Key", self.auth.apikey.clone()).body(body).unwrap();
        req.headers_mut().insert("X-Auth-Key",  "123".parse().unwrap());
        req
    }
}

impl Domain for cf {
    fn SetRecord(&self, record: &crate::record::Record) {
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /**
     * MAIL=Clould.first@gmail.com
     * API_KEY=c7a13d0579e511a98e5f0b267235ed92e1a01
     */
    #[test]
    fn zones() {
        /*
        let cf = new("Clould.first@gmail.com", "c7a13d0579e511a98e5f0b267235ed92e1a01")
        let zones = cf.zones().await.unwrap();
        */
    }
}