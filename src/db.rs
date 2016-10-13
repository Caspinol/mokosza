extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;

use std::error::Error;
use std::io;

use self::r2d2_postgres::postgres::error;
use self::r2d2_postgres::postgres::types::ToSql;
use self::r2d2_postgres::{ PostgresConnectionManager, TlsMode };
use self::r2d2::Pool;

#[derive(Clone)]
pub struct DBPool(pub Pool<PostgresConnectionManager>);

pub fn new_pool(conn_string: &str, poolsize: u32) -> DBPool {
    let config = r2d2::Config::builder().pool_size(poolsize).build();
    let manager = PostgresConnectionManager::new(conn_string, TlsMode::None)
        .expect("Failed to establish DB connection");
    match r2d2::Pool::new(config, manager) {
        Ok(pool) => {
            DBPool(pool)
        },
        Err(_) => {
            panic!("Failed to create pool")
        }
    }
}

pub fn execute(pool: &DBPool, query: &str, params: &[&ToSql]) -> Result<u64, error::Error> {
    match pool.0.get() {
        Ok(c) => {
            c.execute(query, params)
        },
        Err(err) => {
            Err(error::Error::Io(io::Error::new(io::ErrorKind::Other,
                                                err.description())))
        }
    }
}

pub fn prepared_stmt<T: ToSql>(pool: &DBPool, query: &str, list: &Vec<T>)
                               -> Result<u64, error::Error> {
    let mut modified = 0;
    match pool.0.get() {
        Ok(c) => {
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
        Err(err) => {
            Err(error::Error::Io(io::Error::new(io::ErrorKind::Other,
                                                err.description())))
        }
    }
}

pub fn store_domains(pool: &DBPool, list: &Vec<String>) {
    let query = "INSERT INTO domain_list (domain_url) VALUES ($1)";
    let _ = prepared_stmt(pool, query, &list);
}

pub fn next_domain(pool: &DBPool) -> Option<String> {
    let q = "SELECT domain_url FROM domain_list WHERE crawled_at is null \
             AND status='new' LIMIT 1";
    
    match pool.0.get() {
        Err(_) => None,
        Ok(conn) => {
            match conn.query(q, &[]) {
                Err(_) => None,
                Ok(row) => {
                    let q = "UPDATE domain_list SET status='processing'  WHERE domain_url=$1";
                    match row.into_iter().next() {
                        None => { None },
                        Some(url_row) => {
                            let url: String = url_row.get(0);
                            let _ = execute(pool, q, &[&url.as_str()]);
                            Some(url)
                        }
                    }
                }
            }
        }
    }
}

pub fn domain_done(pool: &DBPool, url: &str) -> Result<(),()> {
    let q = "UPDATE domain_list SET status='done', crawled_at=NOW() \
             WHERE domain_url=$1";
    
    match execute(pool, q, &[&url]) {
        Err(_) => Err(()),
        Ok(_) => Ok(())
    }
}

pub fn domain_err(pool: &DBPool, url: &str) -> Result<(),()> {
    let q = "UPDATE domain_list SET status='invalid', crawled_at=NOW() \
             WHERE domain_url=$1";
    let res = execute(pool, q, &[&url]);
    match res {
        Err(_) => Err(()),
        Ok(_) => Ok(())
    }
}

#[cfg(test)]

#[test]
fn test_db_query() {
    let pool = new_pool("postgresql://mokosza:mokoszamokosza@\
                           catdamnit.chs4hglw5opg.eu-west-1.rds.amazonaws.com:5432/mokosza");

    let _ = execute(&pool, "DROP TABLE test;", &[]);
    let res = execute(&pool, "CREATE TABLE test (\
                              id              SERIAL PRIMARY KEY,\
                              col1            VARCHAR NOT NULL,\
                              col2            BIGINT\
                              )", &[]);
    assert_eq!(res.unwrap(), 0u64);
    
    let res = execute(&pool, "INSERT INTO test(col1, col2) VALUES($1, $2)",
                      &[&"test", &123456i64]);
    assert_eq!(res.unwrap(), 1u64);

    let conn = pool.0.get().unwrap();
    let rows = conn.query("SELECT id, col1, col2 FROM test WHERE col1 = $1",
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
    
    let res = execute(&pool, "DROP TABLE test;", &[]);
    assert_eq!(res.unwrap(), 0u64);
}
