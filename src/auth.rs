use std::arch::aarch64::ST;

use tide::{Request, Response, Redirect, Body, StatusCode};
use tide::http::Cookie;
use uuid::Uuid;
use time::{OffsetDateTime, Duration};
use serde::Deserialize;
use floppa_auth::{User, Session, Apps};
use crate::State;

#[derive(Deserialize)]
struct UserBody {
  username: String,
  password: String,
}
#[derive(Deserialize)]
#[serde(default)]
struct Query {
  app: String,
  secret: String,
}
impl Default for Query {
  fn default() -> Self {
    Self {
      app: "floppa-auth".to_string(),
      secret: "meow".to_string(),
    }
  }
}

#[derive(Deserialize)]
struct AuthQuery {
  session_id: String,
  app_secret: String,
  app_name: String,
}

pub async fn register(mut req: Request<State>) -> tide::Result {
  let user: UserBody = req.body_form().await?;
  let mut state = req.state().db.get_mut();
  let query: Query = req.query().unwrap_or_default();
  let mut url: String = "/".to_string();
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
      let expires = OffsetDateTime::now_utc() + Duration::day();
      match state.apps.get(&query.app) {
        Some(app) => {
          if query.secret == app.secret {
            url = app.url.to_owned();
            url += &("id=".to_string() + &id.to_string());
          } else {
            url = "/".to_string();
          }
        }
        None => println!("error"),
      };
      state.sessions.insert(
        id,
        Session {
          username: user.username,
          expires,
          ip: req.peer_addr().unwrap().into(),
          app: query.app,
        },
      );
      let mut res: Response = Redirect::new(url).into();
      res.insert_cookie(
        Cookie::build("session", id.to_string())
          .http_only(true)
          .path("/")
          .expires(expires)
          .finish(),
      );
      Ok(res)
    }
  }
}

pub async fn login(mut req: Request<State>) -> tide::Result {
  let user: UserBody = req.body_form().await?;
  let mut state = req.state().db.get_mut();
  let query: Query = req.query().unwrap_or_default();
  let mut url: String = "/".to_string();
  match state.users.get(&user.username) {
    Some(u) => {
      if bcrypt::verify(user.password, &u.password)? {
        let id = Uuid::new_v4();
        let expires = OffsetDateTime::now_utc() + Duration::day();

        match state.apps.get(&query.app) {
          Some(app) => {
            if query.secret == app.secret {
              url = app.url.to_owned();
              url += &("id=".to_string() + &id.to_string());
            } else {
              url = "/".to_string();
            }
          }
          None => println!("error"),
        };
        state.sessions.insert(
          id,
          Session {
            username: user.username,
            expires,
            ip: req
              .peer_addr()
              .unwrap()
              .to_string()
              .split(':')
              .next()
              .unwrap()
              .to_string(),
            app: query.app,
          },
        );

        let mut res: Response = Redirect::new(url).into();
        res.insert_cookie(
          Cookie::build("session", id.to_string())
            .http_only(true)
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

pub async fn logout(req: Request<State>) -> tide::Result {
  let mut state = req.state().db.get_mut();
  let mut res: Response = Redirect::new("/").into();
  if let Some(c) = req.cookie("session") {
    state.sessions.remove(&Uuid::parse_str(c.value())?);
    res.remove_cookie(Cookie::build("session", "").path("/").finish())
  }
  Ok(res)
}

pub async fn delete(req: Request<State>) -> tide::Result {
  let mut state = req.state().db.get_mut();
  let mut res: Response = Redirect::new("/").into();
  if let Some(c) = req.cookie("session") {
    let username = state
      .sessions
      .get(&Uuid::parse_str(c.value())?)
      .unwrap()
      .username
      .clone();
    state.users.remove(&username);
    state.sessions.remove(&Uuid::parse_str(c.value())?);
    res.remove_cookie(Cookie::build("session", "").path("/").finish())
  }
  Ok(res)
}

pub async fn delete_session(req: Request<State>) -> tide::Result {
  let mut state = req.state().db.get_mut();

  if let Some(c) = req
    .cookie("session")
    .and_then(|c| state.sessions.get(&Uuid::parse_str(c.value()).unwrap()))
  {
    let uid = Uuid::parse_str(req.param("id").unwrap())?;
    if state.sessions.get(&uid).unwrap().username == c.username {
      state.sessions.remove(&uid);
    }
  }

  Ok(Redirect::new("/sessions").into())
}

pub fn auth(req: &Request<State>) -> Option<(Uuid, Session, User)> {
  let state = req.state().db.get();
  req
    .cookie("session")
    .and_then(|c| Uuid::parse_str(c.value()).ok())
    .and_then(|u| state.sessions.get(&u).map(|s| (u, s)))
    .and_then(|s| {
      state
        .users
        .get(&s.1.username)
        .map(|u| (s.0, s.1.clone(), u.clone()))
    })
}

#[derive(Deserialize)]
struct Meow {
  name: String,
  url: String,
}
pub async fn add_app(req: Request<State>) -> tide::Result {
  let mut state = req.state().db.get_mut();
  let query: Meow = req.query()?;
  let id = Uuid::new_v4().to_string();
  println!(
    "App {} created with callback url {} and secret {}",
    &query.name, &query.url, &id
  );
  state.apps.insert(
    query.name,
    Apps {
      secret: id,
      url: query.url,
    },
  );
  Ok(Redirect::new("/").into())
}

//TODO third party callback authentication with session
pub async fn auth_session(mut req: Request<State>) -> tide::Result {
  let auth_query: AuthQuery = req.query()?;
  let mut conn = req.state().db.get_mut();

  Ok(match conn.sessions.get(auth_query.session_id) {
    Some(session) if session.expires > OffsetDateTime::now_utc() => {
      match conn.apps.get(&auth_query.app_name) {
        Some(app) if app.secret == auth_query.app_secret => Response::builder(StatusCode::Ok)
          .body(session.username)
          .build(),
        Some(_) => Response::new(StatusCode::Unauthorized),
        None => Response::new(StatusCode::NotFound),
      }
    }
    Some(_) => Response::new(StatusCode::Unauthorized),
    None => Response::new(StatusCode::NotFound),
  })
}
