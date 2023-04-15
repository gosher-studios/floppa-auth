use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Data {
  pub sessions: HashMap<Uuid, Session>,
  pub users: HashMap<String, User>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
  pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
  pub username: String,
  pub expires: time::OffsetDateTime,
  pub ip: String,
}
