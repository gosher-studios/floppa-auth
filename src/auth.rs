use tide::{Request, Response, Redirect};
use tide::http::Cookie;
use uuid::Uuid;
use time::{OffsetDateTime, Duration};
use serde::Deserialize;
use floppa_auth::{User, Session};
use crate::State;

#[derive(Deserialize)]
struct UserBody {
  username: String,
  password: String,
}

pub async fn register(mut req: Request<State>) -> tide::Result {
  let user: UserBody = req.body_form().await?;
  let mut state = req.state().db.get_mut();

  let url = &req.query::<Link>()?.url;
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
      state.sessions.insert(
        id,
        Session {
          username: user.username,
          expires,
          ip: req.peer_addr().unwrap().into(),
        },
      );
      let mut link: String = "/".to_string();
      if url != "/" {
        link = url.to_owned() + "?id=" + &id.to_string();
      }
      let mut res: Response = Redirect::new(link).into();
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

#[derive(Deserialize)]
struct Link {
  url: String,
}
pub async fn login(mut req: Request<State>) -> tide::Result {
  let user: UserBody = req.body_form().await?;
  let mut state = req.state().db.get_mut();
  let url = &req.query::<Link>()?.url;
  match state.users.get(&user.username) {
    Some(u) => {
      if bcrypt::verify(user.password, &u.password)? {
        let id = Uuid::new_v4();
        let expires = OffsetDateTime::now_utc() + Duration::day();
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
          },
        );

        let mut link: String = "/".to_string();
        if url != "/" {
          link = url.to_owned() + "?id=" + &id.to_string();
        }
        let mut res: Response = Redirect::new(link).into();
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
