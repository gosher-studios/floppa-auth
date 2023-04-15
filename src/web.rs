use tide::{Request, Body};
use askama::Template;
use uuid::Uuid;
use serde::Deserialize;
use reqwest;
use crate::State;

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
}

pub async fn home(req: Request<State>) -> tide::Result {
  let state = req.state().db.get();
  let mut body = Body::from_string(
    match req
      .cookie("session")
      .map(|c| state.sessions.get(&Uuid::parse_str(c.value()).unwrap()))
      .flatten()
    {
      Some(u) => Home {
        username: u.clone().username,
      }
      .render()?,
      None => {
        let t: Login = req.query()?;
        t.render()?
      }
    },
  );
  body.set_mime(Home::MIME_TYPE);
  Ok(body.into())
}

#[derive(Template, Deserialize)]
#[template(path = "register.html")]
struct Register {
  #[serde(default)]
  err: String,
}

pub async fn register(req: Request<State>) -> tide::Result {
  let t: Register = req.query()?;
  let mut body = Body::from_string(t.render()?);
  body.set_mime(Home::MIME_TYPE);
  Ok(body.into())
}

#[derive(Template)]
#[template(path = "sessions.html")]
struct Sessions {
  ses: Vec<Session>,
}

struct Session {
  expiry: String,
  ip: String,
  sid: Uuid,
  location: IpInfo,
}

#[derive(Deserialize, Default)]
struct IpInfo {
  city: String,
}

pub async fn sessions(req: Request<State>) -> tide::Result {
  let state = req.state().db.get();
  let sessions = req
    .cookie("session")
    .map(|c| {
      state
        .sessions
        .get(&Uuid::parse_str(c.value()).ok().unwrap())
    })
    .flatten()
    .unwrap();
  println!("{}", req.peer_addr().unwrap());
  let new = state
    .sessions
    .iter()
    .filter(|e| e.1.username == sessions.username)
    .map(|f| Session {
      expiry: f.1.expires.format("%D %R").to_string(),
      ip: f.1.ip.clone(),
      sid: *f.0,
      location: reqwest::blocking::get("https://ipinfo.io/".to_string() + &f.1.ip.clone())
        .unwrap()
        .json()
        .unwrap_or_default(),
    })
    .collect::<Vec<Session>>();
  let mut body = Body::from_string(Sessions { ses: new }.to_string());
  body.set_mime(Sessions::MIME_TYPE);
  Ok(body.into())
}
