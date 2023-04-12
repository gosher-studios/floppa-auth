mod state;

use tide::{Request, Response, StatusCode};
use tide::security::CorsMiddleware;
use tide::http::headers::HeaderValue;
use tide::http::Cookie;
use uuid::Uuid;
use serde::Deserialize;
use state::{State, User};

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
struct UserBody {
  username: String,
  password: String,
}

#[tokio::main]
async fn main() -> Result {
  tide::log::start();
  let mut app = tide::with_state(State::load()?);
  app.with(CorsMiddleware::new().allow_methods("GET, POST".parse::<HeaderValue>()?));
  app.at("/register").post(new);
  app.at("/login").post(login);
  app.at("/logout").post(logout);
  app.listen("127.0.0.1:8080").await?;
  Ok(())
}

async fn new(mut req: Request<State>) -> tide::Result {
  let user: UserBody = req.body_form().await?;
  let mut state = req.state().lock();
  match state.users.get(&user.username) {
    Some(_) => Ok("uwer exists uwu".into()),
    None => {
      state.users.insert(
        user.username,
        User {
          password: bcrypt::hash(user.password, 10)?,
        },
      );
      Ok("user created".into())
    }
  }
}

async fn login(mut req: Request<State>) -> tide::Result {
  let user: UserBody = req.body_form().await?;
  let mut state = req.state().lock();
  match state.users.get(&user.username) {
    Some(u) => {
      if bcrypt::verify(user.password, &u.password)? {
        let mut res = Response::new(StatusCode::Ok);
        let id = Uuid::new_v4();
        state.sessions.insert(id, user.username);
        res.insert_cookie(Cookie::new("session", id.to_string()));
        Ok(res)
      } else {
        Ok("not fart".into())
      }
    }
    None => Ok("user no exist".into()),
  }
}

async fn logout(req: Request<State>) -> tide::Result {
  let mut state = req.state().lock();
  let id = Uuid::parse_str(req.cookie("session").unwrap().value())?;
  match state.sessions.get(&id) {
    Some(_) => {
      state.sessions.remove(&id);
      let mut res = Response::new(StatusCode::Ok);
      res.remove_cookie(Cookie::named("session"));
      Ok(res)
    }
    None => Ok("not ok".into()),
  }
}
