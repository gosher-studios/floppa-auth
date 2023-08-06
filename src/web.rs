use tide::{Request, Redirect, Body};
use askama::Template;
use uuid::Uuid;
use serde::Deserialize;
use crate::State;
use crate::auth::auth;

#[derive(Template)]
#[template(path = "home.html")]
struct Home {
  username: String,
}

#[derive(Template, Deserialize)]
#[template(path = "login.html")]
struct Login {
  #[serde(default)]
  err: String,
  appid: String,
  secret: String,
}

pub async fn home(req: Request<State>) -> tide::Result {
  let mut body = Body::from_string(match auth(&req) {
    Some((_, s, _)) => Home {
      username: s.username,
    }
    .render()?,
    None => {
      let t: Login = req.query().unwrap_or(Login {
        err: "".to_string(),
        appid: "floppa-auth".to_string(),
        secret: "meow".to_string(),
      });
      t.render()?
    }
  });
  body.set_mime(Home::MIME_TYPE);
  Ok(body.into())
}

#[derive(Template, Deserialize)]
#[template(path = "register.html")]
struct Register {
  #[serde(default)]
  err: String,
  url: String,
}

pub async fn register(req: Request<State>) -> tide::Result {
  let t: Register = req.query().unwrap_or(Register {
    err: "".to_string(),
    url: "/".to_string(),
  });
  let mut body = Body::from_string(t.render()?);
  body.set_mime(Home::MIME_TYPE);
  Ok(body.into())
}

#[derive(Template)]
#[template(path = "sessions.html")]
struct Sessions {
  sessions: Vec<Session>,
}

struct Session {
  expiry: String,
  ip: String,
  id: Uuid,
  current: bool,
  app: String,
}

pub async fn sessions(req: Request<State>) -> tide::Result {
  Ok(match auth(&req) {
    Some((u, s, _)) => {
      let state = req.state().db.get();
      let sessions = state
        .sessions
        .iter()
        .filter(|e| e.1.username == s.username)
        .map(|f| Session {
          expiry: f.1.expires.format("%D %R"),
          ip: f.1.ip.clone(),
          id: *f.0,
          current: *f.0 == u,
          app: f.1.app.clone(),
        })
        .collect::<Vec<Session>>();
      let mut body = Body::from_string(Sessions { sessions }.to_string());
      body.set_mime(Sessions::MIME_TYPE);
      body.into()
    }
    None => Redirect::new("/").into(),
  })
}
