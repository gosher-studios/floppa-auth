use floppadb::Database;
use floppa_auth::{Data, Result};

fn main() -> Result {
  let db = Database::<Data>::new("auth.db")?;
  println!("Opened 'auth.db'.");
  for (name, user) in db.get().users.iter() {
    println!("{} | {}", name, user.password);
  }
  for (name, meow) in db.get().apps.iter() {
    println!("{} | {} | {}", name, meow.secret, meow.url,);
  }
  Ok(())
}
