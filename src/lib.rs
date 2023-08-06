use std::collections::HashMap;
use tide::http::Url;
use uuid::Uuid;
use time::OffsetDateTime;
use serde::{Serialize, Deserialize};

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Data {
  pub sessions: HashMap<Uuid, Session>,
  pub users: HashMap<String, User>,
  pub apps: HashMap<String, Apps>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
  pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
  pub username: String,
  pub expires: OffsetDateTime,
  pub ip: String,
  pub app: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Apps {
  pub secret: String,
  pub name: String,
  pub url: String,
}
