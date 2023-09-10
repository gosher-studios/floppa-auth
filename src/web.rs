use tide::{Request, Redirect, Body, Response, StatusCode};
use askama::Template;
use std::str::FromStr;
use time::{OffsetDateTime, Duration};
use uuid::Uuid;
use serde::Deserialize;
use crate::State;
use crate::auth::auth;
#[derive(Template)]
#[template(path = "home.html")]
struct Home {
  username: String,
  redirect: bool,
  url: String,
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
    Some((_, s, _)) => {
      let mut state = req.state().db.get_mut();
      let query: Login = req.query().unwrap_or(Login {
        err: "".to_string(),
        appid: "".to_string(),
        secret: "".to_string(),
      });
      match state.clone().apps.get(&query.appid) {
        Some(d) => {
          let secret = &d.clone().to_owned().secret;
          if &query.secret == secret {
            let id = Uuid::new_v4();
            let expires = OffsetDateTime::now_utc() + Duration::day();
            state.sessions.insert(
              id,
              floppa_auth::Session {
                username: "meow".to_string(),
                expires,
                ip: req
                  .peer_addr()
                  .unwrap()
                  .to_string()
                  .split(':')
                  .next()
                  .unwrap()
                  .to_string(),
                app: query.appid,
              },
            );
            let url: String = format!("{}id={}", &d.url, id);
            let t: Home = Home {
              username: s.username,
              redirect: true,
              url,
            };
            t.render()?
          } else {
            let t: Home = Home {
              username: s.username,
              redirect: false,
              url: "".to_string(),
            };
            t.render()?
          }
        }
        None => {
          let t: Home = Home {
            username: s.username,
            redirect: false,
            url: "".to_string(),
          };
          t.render()?
        }
      }
    }
    None => {
      let t: Login = req.query().unwrap_or(Login {
        err: "".to_string(),
        appid: "floppa-auth".to_string(),
        secret: "mrrow".to_string(),
      });
      t.render()?
    }
  });
  body.set_mime(Home::MIME_TYPE);
  return Ok(body.into());
}

#[derive(Template, Deserialize)]
#[template(path = "register.html")]
struct Register {
  #[serde(default)]
  err: String,
  appid: String,
  secret: String,
}

pub async fn register(req: Request<State>) -> tide::Result {
  let t: Register = req.query().unwrap_or(Register {
    err: "".to_string(),
    appid: "floppa-auth".to_string(),
    secret: "mrrow".to_string(),
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
