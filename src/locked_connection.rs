#![allow(dead_code)]

use diesel;

use std::{
    any::Any,
    mem::ManuallyDrop,
    sync::{
        MutexGuard,
    }
};

/// Makes use of ManuallyDrop to avoid dropping the pointers inside the boxes as
/// they only are copies of downcasted Any inside the Database struct.
pub(crate) enum Connection {
    // Default status of a locked connection
    Unknown,

    // MySql Connection
    Mysql(ManuallyDrop<Box<diesel::MysqlConnection>>),

    // PgSql Connection
    Pg(ManuallyDrop<Box<diesel::PgConnection>>),

    // Sqlite Connection
    Sqlite(ManuallyDrop<Box<diesel::SqliteConnection>>),
}

impl Connection {
    pub fn mysql(mysql: &mut diesel::MysqlConnection) -> Self {
        Self::Mysql(
            ManuallyDrop::new(
                unsafe {
                    Box::from_raw(mysql)
                }
            )
        )
    }

    pub fn pg(pg: &mut diesel::PgConnection) -> Self {
        Self::Pg(
            ManuallyDrop::new(
                unsafe {
                    Box::from_raw(pg)
                }
            )
        )
    }

    pub fn sqlite(sqlite: &mut diesel::SqliteConnection) -> Self {
        Self::Sqlite(
            ManuallyDrop::new(
                unsafe {
                    Box::from_raw(sqlite)
                }
            )
        )
    }

    #[inline]
    pub fn is_unknown(&self) -> bool {
        match *self {
            Self::Unknown => true,
            _ => false
        }
    }

    #[inline]
    pub fn is_mysql(&self) -> bool {
        match *self {
            Self::Mysql(_) => true,
            _ => false
        }
    }

    #[inline]
    pub fn is_pg(&self) -> bool {
        match *self {
            Self::Pg(_) => true,
            _ => false
        }
    }

    #[inline]
    pub fn is_sqlite(&self) -> bool {
        match *self {
            Self::Sqlite(_) => true,
            _ => false
        }
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        (self.is_unknown() && other.is_unknown()) ||
        (self.is_mysql() && other.is_mysql()) ||
        (self.is_pg() && other.is_pg()) ||
        (self.is_sqlite() && other.is_sqlite())
    }
}

impl Eq for Connection {}

impl Default for Connection {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error>
    {
        write!(f, "{}", match self {
            Self::Unknown => "Unkown",
            Self::Mysql(_) => "MysqlConnection",
            Self::Pg(_) => "PgConnection",
            Self::Sqlite(_) => "SqliteConnection"
        })
    }
}

/// 
#[derive(Debug)]
pub(crate) struct LockedConnection<'lock> {
    guard: MutexGuard<'lock, Option<Box<dyn Any>>>,
    connection: Connection
}

impl<'lock> LockedConnection<'lock> {
    pub fn new(
        guard: MutexGuard<'lock, Option<Box<dyn Any>>>,
        connection: Connection
    ) -> Self
    {
        Self {
            guard,
            connection
        }
    }

    pub fn from_mysql_connection(
        guard: MutexGuard<'lock, Option<Box<dyn Any>>>,
        mysql_connection: &'lock mut diesel::mysql::MysqlConnection
    ) -> Self
    {
        Self {
            guard: guard,
            connection: Connection::Mysql(ManuallyDrop::new( unsafe {
                Box::from_raw(mysql_connection as *mut diesel::mysql::MysqlConnection)
            } ))
        }
    }

    pub fn from_sqlite_connection(
        guard: MutexGuard<'lock, Option<Box<dyn Any>>>,
        sqlite_connection: &'lock mut diesel::sqlite::SqliteConnection
    ) -> Self
    {
        Self {
            guard: guard,
            connection: Connection::Sqlite(ManuallyDrop::new( unsafe {
                Box::from_raw(sqlite_connection as *mut diesel::sqlite::SqliteConnection)
            } ))
        }
    }

    pub fn from_pg_connection(
        guard: MutexGuard<'lock, Option<Box<dyn Any>>>,
        pg_connection: &'lock mut diesel::pg::PgConnection
    ) -> Self
    {
        Self {
            guard: guard,
            connection: Connection::Pg(ManuallyDrop::new( unsafe {
                Box::from_raw(pg_connection as *mut diesel::pg::PgConnection)
            } ))
        }
    }

    pub fn conn(&self) -> &Connection {
        &self.connection
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

impl<'lock> Drop for LockedConnection<'lock> {
    fn drop(&mut self) {
        // Don't do anything: compiler will drop the MutexGuard and unlock the
        // underlying Mutex.
    }
}
