// Source: https://github.com/rust-lang/promote-release/blob/master/src/curl_helper.rs
#![allow(dead_code)]
use std::io::Read;

use anyhow::Context;
use curl::easy::Easy;

pub trait BodyExt {
    fn with_body<S>(&mut self, body: S) -> Request<'_, S>;
    fn without_body(&mut self) -> Request<'_, ()>;
}

impl BodyExt for Easy {
    fn with_body<S>(&mut self, body: S) -> Request<'_, S> {
        Request {
            body: Some(body),
            client: self,
        }
    }
    fn without_body(&mut self) -> Request<'_, ()> {
        Request {
            body: None,
            client: self,
        }
    }
}

pub struct Request<'a, S> {
    body: Option<S>,
    client: &'a mut Easy,
}

impl<S: serde::Serialize> Request<'_, S> {
    pub fn send_with_response<T: serde::de::DeserializeOwned>(self) -> anyhow::Result<T> {
        let mut response = Vec::new();
        let body = self.body.map(|body| serde_json::to_vec(&body).unwrap());
        {
            let mut transfer = self.client.transfer();
            // The unwrap in the read_function is basically guaranteed to not
            // happen: reading into a slice can't fail. We can't use `?` since the
            // return type inside transfer isn't compatible with io::Error.
            if let Some(mut body) = body.as_deref() {
                transfer.read_function(move |dest| Ok(body.read(dest).unwrap()))?;
            }
            transfer.write_function(|new_data| {
                response.extend_from_slice(new_data);
                Ok(new_data.len())
            })?;
            transfer.perform()?;
        }
        serde_json::from_slice(&response)
            .with_context(|| format!("{}", String::from_utf8_lossy(&response)))
    }

    pub fn send(self) -> anyhow::Result<()> {
        let body = self.body.map(|body| serde_json::to_vec(&body).unwrap());
        {
            let mut transfer = self.client.transfer();
            // The unwrap in the read_function is basically guaranteed to not
            // happen: reading into a slice can't fail. We can't use `?` since the
            // return type inside transfer isn't compatible with io::Error.
            if let Some(mut body) = body.as_deref() {
                transfer.read_function(move |dest| Ok(body.read(dest).unwrap()))?;
            }
            transfer.perform()?;
        }
        Ok(())
    }
}
