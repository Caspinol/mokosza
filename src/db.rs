extern crate postgres;

use std::error::Error;
use std::io;

use self::postgres::{Connection, SslMode};
use self::postgres::types::ToSql;
use self::postgres::rows::Rows;
use self::postgres::error;

pub struct DB {
    pub conn: Option<Connection>,
}

impl DB {
    pub fn new(conn_string: &str) -> Result<DB, error::ConnectError> {
        match Connection::connect(conn_string, SslMode::None) {
            Ok(conn) => {
                Ok(DB { conn: Some(conn), })
            },
            Err(err) => {
                format!("DB connection error: \n{}", err.description());
                Err(err)
            }
        }
    }

    pub fn query(&self, query: &str, params: &[&ToSql]) -> Result<Rows, error::Error> {
        match self.conn {
            Some(ref c) => c.query(query, params),
            None => {
                Err(error::Error::Io(io::Error::new(io::ErrorKind::NotConnected,
                                                    "DB: We are not connected to DB")))
            }
        }
    }

    pub fn execute(&self, query: &str, params: &[&ToSql]) -> Result<u64, error::Error> {
        match self.conn {
            Some(ref c) => c.execute(query, params),
            None => {
                Err(error::Error::Io(io::Error::new(io::ErrorKind::NotConnected,
                                                    "DB: We are not connected to DB")))
            }
        }
    }
    
    pub fn prepared_stmt<T: ToSql>(&self, query: &str, list: &Vec<T>)
                                   -> Result<u64, error::Error> {
        let mut modified = 0;
        match self.conn {
            Some(ref c) => {
                match c.prepare(query) {
                    Ok(stmt) => {
                        for elem in list {
                            let _ = stmt.execute(&[&elem]).map_err(|err| err)
                                .and_then(|res| Ok(modified += res));
                        }
                        Ok(modified)
                    },
                    Err(err) => Err(err)
                }
            },
            None => {
                Err(error::Error::Io(io::Error::new(io::ErrorKind::NotConnected,
                                                    "DB: We are not connected to DB")))
            }
        }
    }

    pub fn store_domains(&self, list: &Vec<String>) {
        let query = "INSERT INTO domain_list (domain_url) VALUES ($1)";
        let _ = self.prepared_stmt(query, &list);
    }
    
    pub fn next_domain(&self) -> Option<String> {
        let q = "SELECT domain_url FROM domain_list WHERE crawled_at is null \
                 AND status='new' LIMIT 1";

        match self.query(q, &[]) {
            Err(_) => None,
            Ok(row) => {
                let q = "UPDATE domain_list SET status='processing'  WHERE domain_url=$1";
                match row.into_iter().next() {
                    None => { None },
                    Some(url_row) => {
                        let url: String = url_row.get(0);
                        let _ = self.execute(q, &[&url.as_str()]);
                        Some(url)
                    }
                }
            }
        }
    }

    pub fn domain_done(&self, url: &str) -> Result<(),()> {
        let q = "UPDATE domain_list SET status='done', crawled_at=NOW() \
                 WHERE domain_url=$1";

        match self.execute(q, &[&url]) {
            Err(_) => Err(()),
            Ok(_) => Ok(())
        }
    }

    pub fn domain_err(&self, url: &str) -> Result<(),()> {
        let q = "UPDATE domain_list SET status='invalid', crawled_at=NOW() \
                 WHERE domain_url=$1";
        let res = self.execute(q, &[&url]);
        match res {
            Err(_) => Err(()),
            Ok(_) => Ok(())
        }
    }
}

#[cfg(test)]
#[test]
fn test_db_connection() {
    
    let db = DB::new("postgresql://mokosza:mokoszamokosza@\
                      catdamnit.chs4hglw5opg.eu-west-1.rds.amazonaws.com:5432/mokosza");
    if let Ok(c) = db {
        assert!(c.conn.is_some());
    }
}

#[test]
fn test_db_query() {
    let db_new = DB::new("postgresql://mokosza:mokoszamokosza@\
                      catdamnit.chs4hglw5opg.eu-west-1.rds.amazonaws.com:5432/mokosza");

    if let Ok(db) = db_new {
        assert!(db.conn.is_some());

        let _ = db.execute("DROP TABLE test;", &[]);
        let res = db.execute("CREATE TABLE test (\
                              id              SERIAL PRIMARY KEY,\
                              col1            VARCHAR NOT NULL,\
                              col2            BIGINT\
                              )", &[]);
        assert_eq!(res.unwrap(), 0u64);
        
        let res = db.execute("INSERT INTO test(col1, col2) VALUES($1, $2)",
                             &[&"test", &123456i64]);
        assert_eq!(res.unwrap(), 1u64);
        
        let rows = db.query("SELECT id, col1, col2 FROM test WHERE col1 = $1",
                            &[&"test"]);
        let rows = rows.unwrap();
        for row in rows.iter() {
            let id: i32 = row.get("id");
            let col1: String = row.get(1);
            let col2: i64 = row.get("col2");
            assert_eq!(id, 1);
            assert_eq!(col1, "test");
            assert_eq!(col2, 123456);
        }
        
        let res = db.execute("DROP TABLE test;", &[]);
        assert_eq!(res.unwrap(), 0u64);
    }
}
