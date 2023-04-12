use bonsaidb::core::connection::Connection;
use bonsaidb::core::schema::{Collection, Schema};
use bonsaidb::local::config::{Builder, StorageConfiguration};
use bonsaidb::local::Database;
use bonsaidb_core::schema::SerializedCollection;
use tide::http::headers::HeaderValue;
use tide::http::Cookie;
use tide::prelude::*;
use tide::security::CorsMiddleware;
use tide::{Request, Response, StatusCode};
use uuid::Uuid;
pub type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[derive(Debug, Schema)]
#[schema(name = "userdb", collections = [User,Session])]
struct UserDB;

#[derive(Debug, Serialize, Deserialize, Collection, Eq, PartialEq)]
#[collection(name = "users",primary_key = String)]
struct User {
    password: String,
}

#[derive(Debug, Serialize, Deserialize, Collection, Eq, PartialEq)]
#[collection(name = "session",primary_key = Uuid)]
struct Session {
    uid: String,
}
#[derive(Deserialize, Serialize)]
struct LoginUser {
    username: String,
    password: String,
}
#[derive(Clone)]
struct State {
    db: bonsaidb_local::Database,
}

impl State {
    async fn new() -> Result<Self> {
        let dab = Database::open::<UserDB>(StorageConfiguration::new("based.bonsaidb"))?;
        Ok(Self { db: dab })
    }
}

#[async_std::main]
async fn main() -> Result {
    tide::log::start();
    let mut app = tide::with_state(State::new().await?);
    app.with(CorsMiddleware::new().allow_methods("GET, POST".parse::<HeaderValue>()?));
    app.at("/register").post(new);
    app.at("/login").post(login);
    app.at("/logout").post(logout);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn new(mut req: Request<State>) -> tide::Result {
    let mut user: LoginUser = req.body_form().await?;
    match User::get(&user.username, &req.state().db)? {
        Some(_) => Ok("uwer exists uwu".into()),
        None => {
            user.password = bcrypt::hash(user.password, 10)?;
            req.state().db.collection::<User>().insert(
                &user.username,
                &User {
                    password: user.password,
                },
            )?;
            Ok("user created".into())
        }
    }
}

async fn login(mut req: Request<State>) -> tide::Result {
    let user: LoginUser = req.body_form().await?;
    match User::get(&user.username, &req.state().db)? {
        Some(u) => {
            if bcrypt::verify(user.password, &u.contents.password)? {
                let mut res = Response::new(StatusCode::Ok);
                let fart = Uuid::new_v4();
                req.state()
                    .db
                    .collection::<Session>()
                    .insert(&fart, &Session { uid: user.username })?;
                res.insert_cookie(Cookie::new("session", fart.to_string()));
                Ok(res)
            } else {
                Ok("not fart".into())
            }
        }
        None => Ok("user no exist".into()),
    }
}

async fn logout(req: Request<State>) -> tide::Result {
    let db = req.state().db.collection::<Session>();
    match db.get(&Uuid::parse_str(req.cookie("session").unwrap().value())?)? {
        Some(doc) => {
            db.delete(&doc)?;
            let mut res = Response::new(StatusCode::Ok);
            res.remove_cookie(Cookie::named("session"));
            Ok(res)
        }
        None => Ok("not ok".into()),
    }
}
