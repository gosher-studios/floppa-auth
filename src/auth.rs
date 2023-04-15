use tide::{Request, Response, Redirect};
use tide::http::Cookie;
use uuid::Uuid;
use serde::Deserialize;
use floppa_auth::User;
use floppa_auth::Session;
use crate::State;
use time::OffsetDateTime;
use time::Duration;
#[derive(Deserialize)]
struct UserBody {
  username: String,
  password: String,
}

pub async fn register(mut req: Request<State>) -> tide::Result {
  let user: UserBody = req.body_form().await?;
  let mut state = req.state().db.get_mut();
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
      let time = OffsetDateTime::now_utc() + Duration::day();
      state.sessions.insert(
        id,
        Session {
          username: user.username,
          expires: time,
          ip: req.peer_addr().unwrap().into(),
        },
      );
      let mut res: Response = Redirect::new("/").into();
      res.insert_cookie(
        Cookie::build("session", id.to_string())
          .http_only(true)
          .secure(true)
          .path("/")
          .expires(time)
          .finish(),
      );
      Ok(res)
    }
  }
}

pub async fn login(mut req: Request<State>) -> tide::Result {
  let user: UserBody = req.body_form().await?;
  let mut state = req.state().db.get_mut();
  match state.users.get(&user.username) {
    Some(u) => {
      if bcrypt::verify(user.password, &u.password)? {
        let id = Uuid::new_v4();
        let time = OffsetDateTime::now_utc() + Duration::day();
        state.sessions.insert(
          id,
          Session {
            username: user.username,
            expires: time,
            ip: req.peer_addr().unwrap().into(),
          },
        );
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
    .map(|c| state.sessions.get(&Uuid::parse_str(c.value()).unwrap()))
    .flatten()
  {
    let uid = Uuid::parse_str(req.param("id").unwrap())?;
    if state.sessions.get(&uid).unwrap().username == c.username {
      state.sessions.remove(&uid);
    }
  }

  Ok(Redirect::new("/sessions").into())
}
