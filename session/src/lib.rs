#![no_std]

use gstd::{collections::HashMap, msg, prelude::*, ActorId, exec, MessageId, debug};
use gstd::ext::debug;
use session_io::*;

const WORD_LENGTH:usize = 5;
pub struct Session {
    wordle: ActorId,
    player_game_status:HashMap<ActorId, GameStatus>,
    player_message_id:HashMap<MessageId,ActorId>,
    //记录一个session用户参加次数
    player_times:HashMap<ActorId,Vec<String>>,
    max_play_times:u32,
}

#[derive(Clone,Debug)]
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
pub extern "C" fn init() {
    let game_session_init: GameSessionInit =
        msg::load().expect("Unable to decode GameSessionInit");
    unsafe {
        SESSION = Some(Session {
            wordle: game_session_init.wordle_address,
            player_game_status: HashMap::new(),
            player_message_id: HashMap::new(),
            player_times: HashMap::new(),
            max_play_times: game_session_init.max_play_times
        });
    }
}

// The `handle()` entry point.
#[no_mangle]
extern fn handle() {
    let user_id = msg::source();
    let session = unsafe {SESSION.as_mut().expect("State isn't initialized")};
    let msg_id = msg::id();

    let action = session.player_game_status.get(&user_id);

    debug!("session.player_game_status.get(&msg::source()) is:{:?}",session.player_game_status.get(&user_id));
    if action.is_none(){
        let user_action:Action = msg::load().expect("Failed to load payload");
        match user_action.clone() {
            Action::StartGame { user } => {
                let send_msg_id = msg::send(session.wordle, user_action, 0).expect("Failed to send");
                let origin_id = msg::id();
                session.player_message_id.insert(origin_id, user_id);
                session.player_game_status.insert(user_id, GameStatus::StartGameMessageSend {
                    origin_id,
                    send_id: send_msg_id
                });
                debug!("origin_id is:{:?}",origin_id);
                debug!("session.player_message_id is:{:?}",&session.player_message_id);
                exec::wait();
            },

            Action::CheckWord { user, word } => {
                //检查word不超过六个数字
                debug!("word.capacity() is:{}",word.capacity());
                assert_eq!(word.capacity(), WORD_LENGTH, "The length of the word exceeds 6");
                session.player_times.entry(user_id).or_insert_with(Vec::<String>::new).push(word);
                let send_msg_id = msg::send(session.wordle, user_action, 0).expect("Failed to send");
                debug!("start check word send_msg_id is:{:?}",send_msg_id);
                let origin_id = msg::id();
                session.player_message_id.insert(origin_id, user_id);
                session.player_game_status.insert(user_id, GameStatus::CheckWordMessageSend {
                    origin_id,
                    send_id: send_msg_id
                });
                exec::wait();
            },
        }
    }else{
        let msg_status = action.expect("player status is empty").clone();
        debug!("received msg_status is:{:?}",msg_status);
        match msg_status{
            GameStatus::StartGameMessageReceived {event} => {
                // 获取用户id
                let game_status = action.expect("Failed to get status");
                debug!("received game_status is:{:?}",game_status);
                msg::reply(event,0).expect("Failed to reply");
                session.player_game_status.remove(&user_id);
            }
            GameStatus::CheckWordMessageReceived{event} => {

                debug!("received check word message id is:{:?}",msg_id);
                // 获取用户id
                debug!("received checked user id is :{:?}",user_id);
                let game_status = action.expect("Failed to get status").clone();
                debug!("check word game_status is:{:?}",game_status);
                debug!("check word event is:{:?}",event);
                //清空用户的状态
                session.player_game_status.remove(&user_id);
                session.player_message_id.remove(&msg_id);
                session.player_times.remove(&user_id);
                //检查用户是否结束了游戏
                match event.clone() {
                    Event::WordChecked{ user,
                        correct_positions,
                        contained_in_word,} => {
                        if !correct_positions.contains(&0)||session.player_times.get(&user_id).unwrap().len() as u32==session.max_play_times{//游戏结束
                            msg::reply(Event::UserWin {user},0).expect("Failed to reply");
                            return;
                        }
                    }
                    _ => {}
                }
                msg::reply(event,0).expect("Failed to reply");
            }
            _=> {
                panic!("Invalid status");
            }
        }
    }
}

#[no_mangle]
extern fn handle_reply() {
    let reply:Event = msg::load().expect("Failed to load payload");
    let session = unsafe {SESSION.as_mut().expect("State isn't initialized")};
    let reply_to = msg::reply_to().expect("Failed to get reply_to");
    debug!("handle reply_to is:{:?}",reply_to);
    match reply {
        Event::GameStarted { user } => {
            let msg_status = session.player_game_status.get(&user).expect("Failed to get status").clone();
            debug!("msg_status reply is:{:?}",msg_status);
            match msg_status {
                GameStatus::StartGameMessageSend {origin_id,send_id} => {
                    if reply_to == send_id {
                        session.player_game_status.insert(user, GameStatus::StartGameMessageReceived{event:reply});
                        exec::wake(origin_id).expect("Failed to wake");
                    }

                }
                _ => panic!("Invalid reply"),
            }
        }
        Event::WordChecked { user, .. } => {
            let player_game_status = &mut session.player_game_status;
            let game_status = player_game_status.get(&user).expect("Failed to get status").clone();
            debug!("reply word game_status is:{:?}",game_status);
            match game_status {
                GameStatus::CheckWordMessageSend {origin_id,send_id} => {
                    if reply_to == send_id {
                        player_game_status.insert(user, GameStatus::CheckWordMessageReceived{event:reply});
                        debug!("reply word player_game_status is:{:?}",player_game_status);
                        exec::wake(origin_id).expect("Failed to wake");
                    }

                }
                _ => panic!("Invalid reply"),
            }
        }
        _=>{panic!("Invalid reply");}
    }
}