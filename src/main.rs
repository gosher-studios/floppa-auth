use tide::Request;
use tide::prelude::*;
use uuid::Uuid;
use bonsaidb::core::schema::{Collection, Schema};
use bonsaidb::local::config::{Builder, StorageConfiguration};
use bonsaidb::local::Database;
use bonsaidb::core::connection::Connection;
pub type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;


#[derive(Debug, Schema)]
#[schema(name = "userdb", collections = [User,Session])]
struct UserDB;



#[derive(Debug, Serialize, Deserialize, Collection, Eq, PartialEq)]
#[collection(name = "users",primary_key = Uuid)]
struct User {
    username: String,
    password: String,
    }

#[derive(Debug, Serialize, Deserialize, Collection, Eq, PartialEq)]
#[collection(name = "session",primary_key = Uuid)]
struct Session {
     uid: Uuid
}



#[derive(Clone)]
struct State {
 db: bonsaidb_local::Database,

}


impl State {
    async fn new() -> Result<Self> {
        let dab = Database::open::<UserDB>(StorageConfiguration::new("based.bonsaidb"))?;
        Ok(Self {
            db : dab
        })
    }
}



#[async_std::main]
async fn main() -> Result {
     tide::log::start();
    let mut app = tide::with_state(State::new().await?);
    app.at("/register").post(new);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}



async fn new(mut req: Request<State>) -> tide::Result<String> {
    let mut user: User = req.body_json().await?;
    println!("{:?}",user.password); 
    user.password = bcrypt::hash(user.password,12)?;
    req.state().db.collection::<User>().insert(&Uuid::new_v4(),&user)?; 
    Ok(format!("user created").into())
}



async fn login(mut req:Request<State>) -> tide::Result<> {
    
}