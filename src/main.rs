mod state;
mod web;

use tide::{Request, Response, Redirect};
use tide::security::CorsMiddleware;
use tide::http::headers::HeaderValue;
use tide::http::Cookie;
use uuid::Uuid;
use serde::Deserialize;
use state::{State, User};

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result {
  tide::log::start();
  let mut app = tide::with_state(State::load()?);
  app.with(CorsMiddleware::new().allow_methods("GET, POST".parse::<HeaderValue>()?));
  app.at("/").get(web::home);
  app.at("/register").get(web::register);
  app.at("/auth/register").post(register);
  app.at("/auth/login").post(login);
  app.at("/auth/logout").post(logout);
  app.at("/static").serve_dir("static")?;
  app.listen("127.0.0.1:8080").await?;
  Ok(())
}

#[derive(Deserialize)]
struct UserBody {
  username: String,
  password: String,
}

async fn register(mut req: Request<State>) -> tide::Result {
  let user: UserBody = req.body_form().await?;
  let mut state = req.state().lock();
  match state.users.get(&user.username) {
    Some(_) => Ok(Redirect::new("/register?err=exists").into()),
    None => {
      state.users.insert(
        user.username.clone(),
        User {
          password: bcrypt::hash(user.password, 10)?,
        },
      );
      let id = Uuid::new_v4();
      state.sessions.insert(id, user.username);
      let mut res: Response = Redirect::new("/").into();
      res.insert_cookie(
        Cookie::build("session", id.to_string())
          .http_only(true)
          .secure(true)
          .path("/")
          .finish(),
      );
      Ok(res)
    }
  }
}

async fn login(mut req: Request<State>) -> tide::Result {
  let user: UserBody = req.body_form().await?;
  let mut state = req.state().lock();
  match state.users.get(&user.username) {
    Some(u) => {
      if bcrypt::verify(user.password, &u.password)? {
        let id = Uuid::new_v4();
        state.sessions.insert(id, user.username);
        let mut res: Response = Redirect::new("/").into();
        res.insert_cookie(
          Cookie::build("session", id.to_string())
            .http_only(true)
            .secure(true)
            .path("/")
            .finish(),
        );
        Ok(res)
      } else {
        Ok(Redirect::new("/?err=incorrect").into())
      }
    }
    None => Ok(Redirect::new("/?err=notfound").into()),
  }
}

async fn logout(req: Request<State>) -> tide::Result {
  let mut state = req.state().lock();
  let mut res: Response = Redirect::new("/").into();
  if let Some(c) = req.cookie("session") {
    state.sessions.remove(&Uuid::parse_str(c.value())?);
    res.remove_cookie(Cookie::build("session", "").path("/").finish())
  }
  Ok(res)
}
