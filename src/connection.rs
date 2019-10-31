#![allow(dead_code)]

use std::{
    any::Any,
    error::Error,
    sync::{
        LockResult,
        Mutex,
        MutexGuard,
    }
};

use crate::error;
use crate::Result;

#[derive(Debug)]
pub struct Connection
{
    _connection: Mutex<Option<Box<dyn Any>>>,
}

impl Connection
{
    pub fn new(connection: Option<Box<dyn Any>>) -> Self
    {
        Self {
            _connection: Mutex::new(connection)
        }
    }

    pub fn lock(&self) -> LockResult<MutexGuard<'_, Option<Box<dyn Any>>>>
    {
        log::debug!("Locking connection...");
        
        let lock = self._connection.lock();

        if lock.is_ok() {
            log::debug!("Successfully locked connection!");
        } else {
            log::debug!("Failed to lock connection.");
        }

        lock
    }

    pub fn initialized(&self) -> Result<bool>
    {
        let guard = self.lock().map_err(|err| error::Error::new(
            error::ErrorKind::Other,
            err.description()
        ))?;

        Ok(guard.is_some())
    }
}

impl Default for Connection
{
    fn default() -> Self
    {
        Self {
            _connection: Mutex::new(None)
        }
    }
}

unsafe impl Send for Connection {}
unsafe impl Sync for Connection {}