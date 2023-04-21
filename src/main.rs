mod web;
mod auth;

use tide::security::CorsMiddleware;
use tide::http::headers::HeaderValue;
use floppadb::Database;
use floppa_auth::{Data, Result};

#[derive(Clone)]
pub struct State {
  db: Database<Data>,
}

#[tokio::main]
async fn main() -> Result {
  tide::log::start();
  let mut app = tide::with_state(State {
    db: Database::new("auth.db")?,
  });
  app.with(CorsMiddleware::new().allow_methods("GET, POST".parse::<HeaderValue>()?));
  app.at("/").get(web::home);
  app.at("/sessions").get(web::sessions);
  app.at("/register").get(web::register);
  app.at("/auth/register").post(auth::register);
  app.at("/auth/login").post(auth::login);
  app.at("/auth/logout").post(auth::logout);
  app.at("/auth/delete").post(auth::delete);
  app.at("/auth/sessions/:id").post(auth::delete_session);
  app.at("/static").serve_dir("static")?;
  app.listen("127.0.0.1:8080").await?;
  Ok(())
}
