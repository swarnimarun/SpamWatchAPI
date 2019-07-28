#[macro_use]
extern crate postgres;
#[macro_use]
extern crate slog;

use std::process::exit;

use actix_web::{App, guard, HttpResponse, HttpServer, web};

use utils::BoxResult;

use crate::database::Database;

#[macro_use]
mod utils;
mod settings;
mod database;
mod errors;

mod routes;
#[cfg(test)]
mod tests;

fn setup_database() -> BoxResult<i32> {
    let logger = utils::logger();
    let mut db = match Database::new() {
        Ok(d) => d,
        Err(e) => {
            error!(logger, "A Error occured while connecting to PostgreSQL"; "error" => e.to_string());
            return Ok(1);
        }
    };
    db.setup_tables()?;
    db.create_genesis_token()?;
    Ok(0)
}

fn run() -> BoxResult<i32> {
    let logger = utils::logger();
    info!(logger, "Starting {}", env!("CARGO_PKG_NAME"); "version" => &env!("CARGO_PKG_VERSION"));
    if config!(masterid) == 777000 {
        warn!(logger, "MasterID not set. Defaulting to Telegrams id (777000). To avoid this set `masterid` under the `general` section in the config.")
    }
    info!(logger, "Master ID is {}", config!(masterid));
    let db_code = setup_database()?;
    if db_code > 0 {
        return Ok(db_code);
    }
    let location = format!("{}:{}", config!(server.host), config!(server.port));
    info!(logger, "Starting Server on {}", location);
    HttpServer::new(|| {
        App::new()
            .service(web::resource("/").route(
                web::route()
                    .guard(guard::Any(guard::Get()).or(guard::Head()))
                    .to(|| HttpResponse::MethodNotAllowed())
                    .to(routes::root::info)))
            .service(web::resource("/version").route(
                web::route()
                    .guard(guard::Get())
                    .to(|| HttpResponse::MethodNotAllowed())
                    .to(routes::root::version)))
            .service(web::resource("/tokens").route(
                web::route()
                    .guard(guard::Get())
                    .to(|| HttpResponse::MethodNotAllowed())
                    .to(routes::tokens::get_tokens)))
            .service(web::resource("/tokens/{id}").route(
                web::route()
                    .guard(guard::Get())
                    .to(|| HttpResponse::MethodNotAllowed())
                    .to(routes::tokens::get_token)))
    })
        .bind(location).unwrap()
        .run().unwrap();
    Ok(0)
}

fn main() -> BoxResult<()> {
    let exit_code = run()?;
    exit(exit_code);
}
