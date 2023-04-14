use tide::{Request, Response, Redirect};
use tide::http::Cookie;
use uuid::Uuid;
use serde::Deserialize;
use floppa_auth::User;
use crate::State;

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

pub async fn login(mut req: Request<State>) -> tide::Result {
  let user: UserBody = req.body_form().await?;
  let mut state = req.state().db.get_mut();
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

pub async fn logout(req: Request<State>) -> tide::Result {
  let mut state = req.state().db.get_mut();
  let mut res: Response = Redirect::new("/").into();
  if let Some(c) = req.cookie("session") {
    state.sessions.remove(&Uuid::parse_str(c.value())?);
    res.remove_cookie(Cookie::build("session", "").path("/").finish())
  }
  Ok(res)
}
