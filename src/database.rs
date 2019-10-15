#![allow(dead_code)]

use diesel::{
    connection::Connection as _
};

use rocket::{
    data::Data,
    fairing::{
        Fairing,
        Info,
        Kind
    },
    Outcome,
    request::Request,
    Rocket
};

use std::{
    any::Any,
    error::Error,
    sync::{
        Arc,
        RwLock
    }
};

use crate::Connection;
use crate::LockedConnection;
use crate::Configuration;
use crate::Settings;
use crate::error;
use crate::Result;

#[derive(Clone, Debug)]
pub struct Database
{
    _configuration: Arc<RwLock<Option<Configuration>>>,
    _database: Arc<Connection>
}

impl Default for Database {
    fn default() -> Self
    {
        Self {
            _database: Arc::new(Connection::default()),
            _configuration: Arc::new(RwLock::new(None))
        }
    }
}

impl Database
{
    pub fn new() -> Self
    {
        Self::default()
    }

    pub fn has_configuration(&self) -> bool
    {
        if let Ok(configuration) = self._configuration.read() {
            return configuration.is_some();
        }

        false
    }

    fn settings(&self) -> Result<Settings> {
        let guard = self._configuration.read();

        if guard.is_err() {
            return Err(error::Error::new(
                error::ErrorKind::Other, guard.unwrap_err().description()
            ));
        }
        let guard = guard.unwrap();

        if guard.is_none() {
            return Err(error::Error::new(
                error::ErrorKind::Other, "no configuration available"
            ));
        }
        let configuration = guard.as_ref().unwrap();

        let url_value = configuration.get("url").map_err(|err| error::Error::new(
            error::ErrorKind::Other,
            err.description()
        ))?.ok_or(error::Error::new(
            error::ErrorKind::MissingValue, "no `url` value in configuration"
        ))?;

        let url = url_value.as_str().ok_or(error::Error::new(
            error::ErrorKind::FormatError,
            "invalid format for `url` in configuration."
        ))?.to_owned();
        
        Settings::new(url)
    }

    pub fn initialized(&self) -> Result<bool>
    {
        self._database.initialized()
    }

    fn initialize(&self) -> Result<()> {
        let settings = self.settings()?;
        let database = match settings.url().scheme() {
            "mysql" => {
                let mysql = diesel::MysqlConnection::establish(
                    settings.url().as_str()
                ).unwrap();

                Some(Box::new(mysql) as Box<dyn Any>)
            },
            "postgres" | "postgresql" => {
                let postgresql = diesel::PgConnection::establish(
                    settings.url().as_str()
                ).unwrap();

                Some(Box::new(postgresql) as Box<dyn Any>)
            },
            "sqlite" => {
                let sqlite = diesel::SqliteConnection::establish(
                    settings.url().path()
                ).unwrap();

                Some(Box::new(sqlite) as Box<dyn Any>)
            },
            _ => { None }
        };

        let mut guard = self._database.lock().map_err(|_err| error::Error::new(
            error::ErrorKind::Other, "failed to update database connection"
        ))?;

        *guard = database;

        Ok(())
    }

    fn lock<'lock>(&'lock self) -> Result<LockedConnection<'lock>>
    {
        let settings = self.settings()?;
        let lock = self._database.lock();

        if lock.is_err() {
            return Err(error::Error::new(
                error::ErrorKind::Other, "database got poisoned"
            ));
        }
        let mut guard = lock.unwrap();

        match guard.as_mut() {
            None => Err(error::Error::new(
                error::ErrorKind::Other, "database is not ready"
            )),
            Some(boxed_database) => {
                let conn = match settings.url().scheme() {
                    "mysql" => {
                        crate::locked_connection::Connection::mysql(
                            boxed_database.downcast_mut::<diesel::MysqlConnection>().ok_or(
                                error::Error::new(
                                    error::ErrorKind::Other,
                                    "failed to downcast database"
                                )
                            )?
                        )
                    },
                    "postgres" | "postgresql" => {
                        crate::locked_connection::Connection::pg(
                            boxed_database.downcast_mut::<diesel::PgConnection>().ok_or(
                                error::Error::new(
                                    error::ErrorKind::Other,
                                    "failed to downcast database"
                                )
                            )?
                        )
                    },
                    "sqlite" => {
                        crate::locked_connection::Connection::sqlite(
                            boxed_database.downcast_mut::<diesel::SqliteConnection>().ok_or(
                                error::Error::new(
                                    error::ErrorKind::Other,
                                    "failed to downcast database"
                                )
                            )?
                        )
                    }
                    _ => { unimplemented!() }
                };

                Ok(LockedConnection::new(guard, conn))
            }
        }
    }

    pub fn interact<T, E, MysqlF, PgF, SqliteF>(&self, mysql_f: MysqlF, pg_f: PgF, sqlite_f: SqliteF) -> Result<T>
        where E: From<diesel::result::Error> + Error,
              MysqlF: FnOnce(&mut diesel::mysql::MysqlConnection) -> std::result::Result<T, E>,
              PgF: FnOnce(&mut diesel::pg::PgConnection) -> std::result::Result<T, E>,
              SqliteF: FnOnce(&mut diesel::sqlite::SqliteConnection) -> std::result::Result<T, E>,
              
    {
        let lock = self.lock();

        if lock.is_err() {
            return Err(error::Error::new(
                error::ErrorKind::Other, "database got poisoned"
            ));
        }
        let mut guard = lock.unwrap();

        match guard.conn_mut() {
            crate::locked_connection::Connection::Unknown => {
                unimplemented!()
            },
            crate::locked_connection::Connection::Mysql(conn) => {
                mysql_f(conn)
            },
            crate::locked_connection::Connection::Pg(conn) => {
                pg_f(conn)
            },
            crate::locked_connection::Connection::Sqlite(conn) => {
                sqlite_f(conn)
            },
        }.map_err(|err| {
            error::Error::new(error::ErrorKind::Other, err.description() )
        })
    }
}

impl Fairing for Database
{
    fn info(&self) -> Info
    {
        Info {
            name: "Diesel dynamic database",
            kind: Kind::Attach | Kind::Request
        }
    }

    fn on_attach(&self, rocket: Rocket)
        -> std::result::Result<Rocket, Rocket>
    {
        Ok(rocket.manage((*self).clone()))
    } 

    fn on_request(&self, request: &mut Request<'_>, _data: &Data)
    {
        if !self.initialized().unwrap_or(false) {
            // If configuration is not yet available
            if !self.has_configuration() {
                // Tries to get configuration
                let configuration = match request.guard::<Configuration>() {
                    Outcome::Success(configuration) => configuration,
                    Outcome::Forward(_) => { unreachable!() },
                    Outcome::Failure(_err) => {
                        // Ignores the absence of configuration
                        //   => Guard will give an error
                        return ;
                    }
                };

                // Stores configuration
                if let Ok(mut lock) = self._configuration.write() {
                    *lock = Some(configuration);
                }
            }

            // Initialize database connection
            let _ = self.initialize();
        }
    }
}