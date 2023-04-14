use tide::{Request, Body};
use askama::Template;
use uuid::Uuid;
use serde::Deserialize;
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
        username: u.clone(),
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
