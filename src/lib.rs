use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Data {
  pub sessions: HashMap<Uuid, String>,
  pub users: HashMap<String, User>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
  pub password: String,
}
