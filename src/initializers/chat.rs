use crate::models::_entities::users;
use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use socketioxide::{
    extract::{Data, Extension, SocketRef, State},
    SocketIo,
};
use std::sync::{atomic::AtomicUsize, Arc};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::initializers::chat;
use regex::Regex;

#[allow(clippy::module_name_repetitions)]
pub struct ChatInitializer;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(transparent)]
struct Username(String);

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", untagged)]
enum Res {
    Login {
        #[serde(rename = "numUsers")]
        num_users: usize,
    },
    UserEvent {
        #[serde(rename = "numUsers")]
        num_users: usize,
        username: Username,
    },
    Message {
        username: Username,
        message: String,
    },
    Username {
        username: Username,
    },
}

#[derive(Clone)]
struct ChatRoomState {
    pub count: Arc<AtomicUsize>,
    pub loco_ctx: loco_rs::app::AppContext,
}

impl ChatRoomState {
    fn new(ctx: loco_rs::app::AppContext) -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(0)),
            loco_ctx: ctx,
        }
    }
    fn add_user(&self) -> usize {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    }
    fn remove_user(&self) -> usize {
        self.count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) - 1
    }
}

#[async_trait]
impl Initializer for ChatInitializer {
    fn name(&self) -> String {
        "axum-session".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        let (layer, io) = SocketIo::builder()
            .with_state(ChatRoomState::new(ctx.clone()))
            .build_layer();

        io.ns("/", |s: SocketRef| {
            s.on(
                "new message",
                |s: SocketRef, Data::<String>(msg), Extension::<Username>(username)| {
                    let a = msg.clone();
                    let msg = &Res::Message {
                        username,
                        message: msg,
                    };

                    if a.contains("instagram.com") {
                        let re: Regex = Regex::new("instagram.com/(?<profile>.+)$").unwrap();
                        let caps = re.captures(a.as_str()).unwrap();
                        let profile = &caps["profile"];

                        let u: Username = chat::Username(String::from("instagram download"));
                        let m: String = String::from("Begin Instagram profile download...");
                        let cmdmsg = &Res::Message {
                            username: u,
                            message: m,
                        };
                        s.broadcast().emit("new message", cmdmsg).ok();

                        let homedir = match dirs::home_dir() {
                            Some(d) => d.display().to_string(),
                            None => "/home/yonas".into(),
                        };
                        let output = std::process::Command::new(homedir + "/bin/instagram-profile-download")
                            .arg("--no-metadata-json")
                            .arg("--no-compress-json")
                            .arg("--no-captions")
                            .arg("--no-video-thumbnails")
                            .arg("--load-cookies=\"firefox\"")
                            .arg(profile)
                            .output()
                            .expect("failed to execute process");

                        println!("status: {}", output.status);
                        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

                        let u2: Username = chat::Username(String::from("instagram download"));
                        let m2: String = match output.status.success() {
                            true => String::from("Download complete"),
                            false => (String::from_utf8_lossy(&output.stdout) + String::from_utf8_lossy(&output.stderr)).to_string(),
                        };

                        let cmdmsg2 = &Res::Message {
                            username: u2,
                            message: m2,
                        };
                        s.broadcast().emit("new message", cmdmsg2).ok();
                    }

                    s.broadcast().emit("new message", msg).ok();
                },
            );

            s.on(
                "add user",
                |s: SocketRef, Data::<String>(username), user_cnt: State<ChatRoomState>| async move {
                    let users = users::Entity::find()
                        .all(&user_cnt.loco_ctx.db)
                        .await
                        .unwrap();
                    println!("list users {users:#?}");

                    if s.extensions.get::<Username>().is_some() {
                        return;
                    }
                    let num_users = user_cnt.add_user();
                    s.extensions.insert(Username(username.clone()));
                    s.emit("login", &Res::Login { num_users }).ok();

                    let res = &Res::UserEvent {
                        num_users,
                        username: Username(username),
                    };
                    s.broadcast().emit("user joined", res).ok();
                },
            );

            s.on("typing", |s: SocketRef, Extension::<Username>(username)| {
                s.broadcast()
                    .emit("typing", &Res::Username { username })
                    .ok();
            });

            s.on(
                "stop typing",
                |s: SocketRef, Extension::<Username>(username)| {
                    s.broadcast()
                        .emit("stop typing", &Res::Username { username })
                        .ok();
                },
            );

            s.on_disconnect(
                |s: SocketRef, user_cnt: State<ChatRoomState>, Extension::<Username>(username)| {
                    let num_users = user_cnt.remove_user();
                    let res = &Res::UserEvent {
                        num_users,
                        username,
                    };
                    s.broadcast().emit("user left", res).ok();
                },
            );
        });

        let router = router.layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(layer),
        );

        Ok(router)
    }
}
