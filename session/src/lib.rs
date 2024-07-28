#![no_std]

use gstd::{collections::HashMap, msg, prelude::*, ActorId, exec, MessageId};
use template_io::*;

pub struct Session {
    wordle: ActorId,
    player_game_status:HashMap<ActorId, GameStatus>,
    // player_event:HashMap<ActorId,Event>,
    player_message_id:HashMap<MessageId,ActorId>
}

pub enum GameStatus {
    StartGameIdle,
    CheckWordIdle,
    StartGameMessageSend {
        origin_id:MessageId,
        send_id:MessageId,
    },
    StartGameMessageReceived{
        event:Event
    },
    CheckWordMessageSend {
        origin_id:MessageId,
        send_id:MessageId,
    },
    CheckWordMessageReceived{
        event:Event
    },
}

static mut SESSION: Option<Session> = None;

// The `init()` entry point.
#[no_mangle]
extern fn init(wordle_address: ActorId) {
    unsafe {
        SESSION = Some(Session {
            wordle: wordle_address,
            player_game_status: HashMap::new(),
            player_message_id: HashMap::new(),
        });
    }
}

// The `handle()` entry point.
#[no_mangle]
extern fn handle() {
    let user_id = msg::source();
    let session = unsafe {SESSION.as_mut().expect("State isn't initialized")};
    let msg_status = session.player_game_status.get(&msg::source()).unwrap_or(&GameStatus::StartGameIdle);
    match msg_status{
        GameStatus::StartGameIdle => {
            let user_action:Action = msg::load().expect("Failed to load payload");
            match user_action {
                Action::StartGame => {
                    let send_msg_id = msg::send(session.wordle, user_action, 0).expect("Failed to send");
                    let origin_id = msg::id();
                    session.player_message_id.insert(send_msg_id,user_id);
                    session.player_game_status.insert(user_id, GameStatus::StartGameMessageSend {
                        origin_id ,send_id:send_msg_id});
                    exec::wait();
                },
                _ => {panic!("Invalid action");}
            }
        }
        GameStatus::CheckWordIdle => {
            let user_action:Action = msg::load().expect("Failed to load payload");
            match user_action {
                Action::CheckWord =>{
                    let send_msg_id = msg::send(session.wordle, user_action, 0).expect("Failed to send");
                    session.player_message_id.insert(send_msg_id,user_id);
                    session.player_game_status.insert(msg::source(), GameStatus::CheckWordMessageSend {
                        origin_id:msg::id() ,send_id:send_msg_id});
                    exec::wait();
                },
                _=>{
                    panic!("Invalid action");
                }
            }

        }
        GameStatus::StartGameMessageSend => {}
        GameStatus::StartGameMessageReceived {event} => {
            // 获取用户id
            let user_id = session.player_message_id.get(&msg::id()).expect("Failed to get id");
            let game_status =session.player_game_status.get(&user_id).expect("Failed to get status");
            match game_status {
                GameStatus::StartGameMessageReceived {event} => {
                    msg::reply(event,0).expect("Failed to reply");
                }
                _ => {panic!("Invalid status");}
            }
        }
        GameStatus::CheckWordMessageReceived{event} => {
            // 获取用户id
            let user_id = session.player_message_id.get(&msg::id()).expect("Failed to get id");
            let game_status =session.player_game_status.get(&user_id).expect("Failed to get status");
            match game_status {
                GameStatus::CheckWordMessageReceived {event} => {
                    //检查用户是否结束了游戏
                    match event {
                        Event::WordChecked{ user,
                        correct_positions,
                        contained_in_word,} => {
                            if !correct_positions.contains(&0){//游戏结束
                                msg::reply(Event::UserWin {user:*user},0).expect("Failed to reply");
                                //清空用户的状态
                                session.player_game_status.remove(user);
                                let origin_id = msg::id();
                                session.player_message_id.remove(&origin_id);
                                return;
                            }
                        }
                        _ => {}
                    }
                    msg::reply(event,0).expect("Failed to reply");
                }
                _ => {panic!("Invalid status");}
            }
        }
        GameStatus::CheckWordMessageSend { .. } => {}
    }
}

#[no_mangle]
extern fn handle_reply() {
    let reply:Event = msg::load().expect("Failed to load payload");
    let session = unsafe {SESSION.as_mut().expect("State isn't initialized")};
    let reply_to = msg::reply_to().expect("Failed to get reply_to");
    match reply {
        Event::GameStarted { user } => {
            let msg_status = session.player_game_status.get(&user).expect("Failed to get status");
            match msg_status {
                GameStatus::StartGameMessageSend {origin_id,send_id} => {
                    if reply_to == *origin_id {
                        session.player_game_status.insert(user, GameStatus::StartGameMessageReceived{event:reply});
                        exec::wake(*origin_id).expect("Failed to wake");
                    }

                }
                _ => panic!("Invalid reply"),
            }
        }
        Event::WordChecked { user, .. } => {
            let msg_status = session.player_game_status.get(&user).expect("Failed to get status");
            match msg_status {
                GameStatus::CheckWordMessageSend {origin_id,send_id} => {
                    if reply_to == *origin_id {
                        session.player_game_status.insert(user, GameStatus::CheckWordMessageReceived{event:reply});
                        exec::wake(*origin_id).expect("Failed to wake");
                    }

                }
                _ => panic!("Invalid reply"),
            }
        }
        _=>{panic!("Invalid reply");}
    }
}

// // The `state()` entry point.
// #[no_mangle]
// extern fn state() {
//     let state = unsafe { SESSION.take().expect("State isn't initialized") };
//     msg::reply(State::from_iter(state), 0).expect("Failed to reply from `state()`");
// }
