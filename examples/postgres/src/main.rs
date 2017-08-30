#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate futures_await as futures;
extern crate futures_state_stream;
extern crate pretty_env_logger;
#[allow(unused_extern_crates)]
extern crate serde;
extern crate serde_json;
extern crate shio;
extern crate tokio_postgres as postgres;
extern crate uuid;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

// TODO: Connection pooling

mod errors {
    error_chain! {
        foreign_links {
            PostgresConnect(::postgres::error::ConnectError);
            Postgres(::postgres::error::Error);
            Json(::serde_json::Error);
        }
    }
}

use futures::prelude::*;
use futures_state_stream::StateStream;
use postgres::{Connection, TlsMode};
use shio::prelude::*;
use errors::*;

#[derive(Serialize)]
struct Person {
    id: uuid::Uuid,
    name: String,
}

const DATABASE_URL: &'static str = "postgres://postgres@localhost/shio_dev_example";

#[async]
fn index(ctx: Context) -> Result<Response> {
    let conn = await!(Connection::connect(DATABASE_URL, TlsMode::None, &ctx.handle()))?;

    // Setup UUID extension and create table (if needed)

    let conn = await!(conn.batch_execute(r#"
        CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
        CREATE TABLE IF NOT EXISTS person (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
            name TEXT NOT NULL
        );
    "#))?;

    // Insert row into table

    let (stmt, conn) = await!(conn.prepare("INSERT INTO person (name) VALUES ($1)"))?;

    let name: String = ctx.get::<Parameters>()["name"].into();
    let (_, conn) = await!(conn.execute(&stmt, &[&name]))?;

    // Select all rows from table

    let (stmt, conn) = await!(conn.prepare("SELECT id, name FROM person"))?;
    let (results, _) = await!(conn.query(&stmt, &[]).map(|row| {
        Person {
            id: row.get("id"),
            name: row.get("name"),
        }
    }).collect())?;

    // Return rows as JSON

    let s = serde_json::to_string(&results)?;

    Ok(
        Response::build()
            .header(header::ContentType::json())
            .body(s),
    )
}

fn main() {
    pretty_env_logger::init().unwrap();

    Shio::default()
        .route((Method::Get, "/{name}", index))
        .run(":7878")
        .unwrap();
}
