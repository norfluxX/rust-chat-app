use actix::{Actor, ActorContext, StreamHandler, Handler, Message, Addr, Recipient, Context, AsyncContext};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix_files as fs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use serde_json;

// Message structure for chat
#[derive(Serialize, Deserialize, Clone, Message)]
#[rtype(result = "()")]
struct ChatMessage {
    room_id: String,
    user: String,
    message: String,
    #[serde(rename = "type")]
    message_type: String,
}

// Request structure for creating a room
#[derive(Deserialize)]
struct CreateRoomRequest {
    username: String,
}

// Response structure for room creation/joining
#[derive(Serialize)]
struct RoomResponse {
    room_id: String,
    room_name: String,
}

// Chat room structure
struct ChatRoom {
    id: String,
    name: String,
    participants: HashMap<String, Recipient<ChatMessage>>,
}

// Chat session actor
struct ChatSession {
    room_id: String,
    user: String,
    addr: Addr<ChatServer>,
}

// Chat server actor
struct ChatServer {
    rooms: HashMap<String, ChatRoom>,
}

impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Register this session with the server
        let addr = ctx.address().recipient();
        self.addr.do_send(Join {
            room_id: self.room_id.clone(),
            user: self.user.clone(),
            addr,
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.addr.do_send(Leave {
            room_id: self.room_id.clone(),
            user: self.user.clone(),
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let message_data: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                let message_type = message_data.get("type").and_then(|t| t.as_str()).unwrap_or("chat");
                
                match message_type {
                    "chat" => {
                        let message = message_data.get("message").and_then(|m| m.as_str()).unwrap_or("");
                        let chat_msg = ChatMessage {
                            room_id: self.room_id.clone(),
                            user: self.user.clone(),
                            message: message.to_string(),
                            message_type: "chat".to_string(),
                        };
                        self.addr.do_send(chat_msg);
                    },
                    "typing" => {
                        let typing_msg = ChatMessage {
                            room_id: self.room_id.clone(),
                            user: self.user.clone(),
                            message: "".to_string(),
                            message_type: "typing".to_string(),
                        };
                        self.addr.do_send(typing_msg);
                    },
                    _ => (),
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl Actor for ChatServer {
    type Context = actix::Context<Self>;
}

// Message for joining a room
#[derive(Message)]
#[rtype(result = "()")] 
struct Join {
    room_id: String,
    user: String,
    addr: Recipient<ChatMessage>,
}

// Message for leaving a room
#[derive(Message)]
#[rtype(result = "()")] 
struct Leave {
    room_id: String,
    user: String,
}

impl Handler<Join> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        if let Some(room) = self.rooms.get_mut(&msg.room_id) {
            // Send join notification
            let notification = ChatMessage {
                room_id: msg.room_id.clone(),
                user: msg.user.clone(),
                message: format!("{} has joined the chat", msg.user),
                message_type: "notification".to_string(),
            };
            
            for (_, recipient) in &room.participants {
                let _ = recipient.do_send(notification.clone());
            }
            
            room.participants.insert(msg.user, msg.addr);
        }
    }
}

impl Handler<Leave> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: Leave, _: &mut Context<Self>) {
        if let Some(room) = self.rooms.get_mut(&msg.room_id) {
            room.participants.remove(&msg.user);
            
            // Send leave notification
            let notification = ChatMessage {
                room_id: msg.room_id.clone(),
                user: msg.user.clone(),
                message: format!("{} has left the chat", msg.user),
                message_type: "notification".to_string(),
            };
            
            for (_, recipient) in &room.participants {
                let _ = recipient.do_send(notification.clone());
            }
        }
    }
}

impl Handler<ChatMessage> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: ChatMessage, _: &mut Context<Self>) {
        if let Some(room) = self.rooms.get_mut(&msg.room_id) {
            for (_, recipient) in &room.participants {
                let _ = recipient.do_send(msg.clone());
            }
        }
    }
}

impl Handler<ChatMessage> for ChatSession {
    type Result = ();
    fn handle(&mut self, msg: ChatMessage, ctx: &mut ws::WebsocketContext<Self>) {
        let json = serde_json::to_string(&msg).unwrap();
        ctx.text(json);
    }
}

impl ChatServer {
    fn new() -> Self {
        ChatServer {
            rooms: HashMap::new(),
        }
    }
}

// WebSocket handler
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, Error> {
    let room_id = req.match_info().get("room_id").unwrap_or("default").to_string();
    let user = req.match_info().get("user").unwrap_or("anonymous").to_string();

    ws::start(
        ChatSession {
            room_id,
            user,
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

// Create new chat room
async fn create_room(
    req: web::Json<CreateRoomRequest>,
    srv: web::Data<Addr<ChatServer>>,
) -> HttpResponse {
    let room_id = Uuid::new_v4().to_string();
    let room_name = format!("Room {}", Uuid::new_v4().to_string()[..8].to_string());
    let server = srv.get_ref().clone();

    // Create the room
    server.do_send(CreateRoom { 
        room_id: room_id.clone(),
        room_name: room_name.clone(),
    });

    HttpResponse::Ok().json(RoomResponse {
        room_id,
        room_name,
    })
}

// Join route handler
async fn join_route(
    req: HttpRequest,
    srv: web::Data<Addr<ChatServer>>,
) -> HttpResponse {
    let room_id = req.match_info().get("room_id").unwrap_or("default").to_string();
    
    // Check if this is an API request (for room info)
    if req.headers().get("accept").map_or(false, |h| h.to_str().unwrap_or("").contains("application/json")) {
        let server = srv.get_ref().clone();
        let room_info = server.send(GetRoomInfo { room_id: room_id.clone() }).await.unwrap_or(None);
        
        match room_info {
            Some(info) => HttpResponse::Ok()
                .content_type("application/json")
                .json(info),
            None => HttpResponse::NotFound()
                .content_type("application/json")
                .json("Room not found")
        }
    } else {
        // Serve the HTML page for direct browser access
        HttpResponse::Ok()
            .content_type("text/html")
            .body(include_str!("../static/index.html"))
    }
}

// Message for getting room info
#[derive(Message)]
#[rtype(result = "Option<RoomResponse>")]
struct GetRoomInfo {
    room_id: String,
}

// Message for creating a room
#[derive(Message)]
#[rtype(result = "()")]
struct CreateRoom {
    room_id: String,
    room_name: String,
}

impl Handler<GetRoomInfo> for ChatServer {
    type Result = Option<RoomResponse>;
    fn handle(&mut self, msg: GetRoomInfo, _: &mut Context<Self>) -> Option<RoomResponse> {
        self.rooms.get(&msg.room_id).map(|room| RoomResponse {
            room_id: room.id.clone(),
            room_name: room.name.clone(),
        })
    }
}

impl Handler<CreateRoom> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: CreateRoom, _: &mut Context<Self>) {
        self.rooms.insert(
            msg.room_id.clone(),
            ChatRoom {
                id: msg.room_id,
                name: msg.room_name,
                participants: HashMap::new(),
            },
        );
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let chat_server = ChatServer::new().start();
    println!("Server running on port 8087");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(chat_server.clone()))
            .route("/ws/{room_id}/{user}", web::get().to(chat_route))
            .route("/create_room", web::post().to(create_room))
            .route("/join/{room_id}", web::get().to(join_route))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("0.0.0.0:8087")?
    .run()
    .await
} 