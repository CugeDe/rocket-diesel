#![allow(dead_code)]

use url::Url;
use crate::error;
use crate::Result;
use std::error::Error as _;

#[derive(Clone, Debug)]
pub struct Settings {
    _url: Url,
}

impl Settings
{
    pub fn new(url: String) -> Result<Self>
    {
        let url = Url::parse(&url).map_err(|_err| error::Error::new(
            error::ErrorKind::Other, _err.description()
        ))?;

        Ok(Self {
            _url: url
        })
    }

    pub fn url(&self) -> &Url {
        &self._url
    }

    pub fn url_mut(&mut self) -> &mut Url {
        &mut self._url
    }
}