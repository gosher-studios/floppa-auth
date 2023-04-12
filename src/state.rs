use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex, MutexGuard};
use std::ops::{Deref, DerefMut};
use tide::log::info;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::Result;

const PATH: &str = "auth.db";

#[derive(Clone)]
pub struct State(Arc<Mutex<StateInner>>);

impl State {
  pub fn load() -> Result<Self> {
    Ok(Self(Arc::new(Mutex::new(match File::open(PATH) {
      Ok(f) => bincode::deserialize_from(f)?,
      Err(_) => StateInner::default(),
    }))))
  }

  pub fn lock(&self) -> StateGuard {
    StateGuard(self.0.lock().unwrap())
  }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct StateInner {
  pub sessions: HashMap<Uuid, String>,
  pub users: HashMap<String, User>,
}

pub struct StateGuard<'a>(MutexGuard<'a, StateInner>);

impl Deref for StateGuard<'_> {
  type Target = StateInner;

  fn deref(&self) -> &StateInner {
    &self.0
  }
}

impl DerefMut for StateGuard<'_> {
  fn deref_mut(&mut self) -> &mut StateInner {
    &mut self.0
  }
}

impl Drop for StateGuard<'_> {
  fn drop(&mut self) {
    info!("Saving '{}'.", PATH);
    bincode::serialize_into(File::create(PATH).unwrap(), &*self.0).unwrap();
  }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
  pub password: String,
}
