mod state;

use tide::{Request, Response, Redirect, Body};
use tide::security::CorsMiddleware;
use tide::http::headers::HeaderValue;
use tide::http::Cookie;
use uuid::Uuid;
use askama::Template;
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
  app.at("/").get(home);
  app.at("/register").get(reg);
  app.at("/login").get(log);
  app.at("/static").serve_dir("static")?;
  app.at("/auth/register").post(register);
  app.at("/auth/login").post(login);
  app.at("/auth/logout").post(logout);
  app.listen("127.0.0.1:8080").await?;
  Ok(())
}

#[derive(Template)]
#[template(path = "home.html")]
struct Home {
  username: String,
  num_users: usize,
}

async fn home(req: Request<State>) -> tide::Result {
  let state = req.state().lock();
  // let username = match req
  //   .cookie("session")
  //   .map(|c| state.sessions.get(&Uuid::parse_str(c.value()).unwrap()))
  //   .flatten()
  // {
  //   Some(s) => s.clone(),
  //   None => "no exist".to_string(),
  // };
  let username = "username".to_string();
  let mut body = Body::from_string(
    Home {
      username,
      num_users: state.users.len(),
    }
    .render()?,
  );
  body.set_mime(Home::MIME_TYPE);
  Ok(body.into())
}

#[derive(Template)]
#[template(path = "register.html")]
struct Register;

async fn reg(_: Request<State>) -> tide::Result {
  let mut body = Body::from_string(Register.render()?);
  body.set_mime(Home::MIME_TYPE);
  Ok(body.into())
}

#[derive(Template)]
#[template(path = "login.html")]
struct Login;

async fn log(req: Request<State>) -> tide::Result {
  match req.cookie("session") {
    Some(_) => Ok(Redirect::new("/").into()),
    None => {
      let mut body = Body::from_string(Login.render()?);
      body.set_mime(Home::MIME_TYPE);
      Ok(body.into())
    }
  }
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
      res.insert_cookie(Cookie::new("session", id.to_string()));
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
        res.insert_cookie(Cookie::new("session", id.to_string()));
        Ok(res)
      } else {
        Ok("incorrect pass".into())
      }
    }
    None => Ok("doesnt exist".into()),
  }
}

async fn logout(req: Request<State>) -> tide::Result {
  // let mut state = req.state().lock();
  let mut res: Response = Redirect::new("/").into();
  // if let Some(c) = req.cookie("session") {
  //   state.sessions.remove(&Uuid::parse_str(c.value())?);
  //   res.remove_cookie(Cookie::named("session"));
  // }
  Ok(res)
}
