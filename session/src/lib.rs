#![no_std]

use gstd::{collections::HashMap, msg, prelude::*, ActorId, exec, MessageId, debug};
use gstd::ext::debug;
use session_io::*;

const WORD_LENGTH:usize = 5;
pub struct Session {
    wordle: ActorId,
    player_game_status:HashMap<ActorId, GameStatus>,
    //记录一个session用户参加次数
    player_times:HashMap<ActorId,Vec<String>>,
    player_start_games:HashMap<ActorId,bool>,
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
            player_times: HashMap::new(),
            player_start_games: Default::default(),
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

    let player_game_status = session.player_game_status.get(&user_id);

    debug!("session.player_game_status.get(&msg::source()) is:{:?}",session.player_game_status.get(&user_id));
    if player_game_status.is_none(){
        let user_action:Action = msg::load().expect("Failed to load payload");
        match user_action.clone() {
            Action::StartGame { user } => {
                let send_msg_id = msg::send(session.wordle, user_action, 0).expect("Failed to send");
                let origin_id = msg::id();
                session.player_game_status.insert(user_id, GameStatus::StartGameMessageSend {
                    origin_id,
                    send_id: send_msg_id
                });
                debug!("origin_id is:{:?}",origin_id);
                exec::wait();
            },

            Action::CheckWord { user, word } => {
                let player_start_game = session.player_start_games.get(&user_id).expect("get player_start_games error").clone();
                if !player_start_game {
                    debug!("player_start_game is false");
                    return;
                }
                //检查word不超过六个数字
                debug!("word.capacity() is:{}",word.capacity());
                assert_eq!(word.capacity(), WORD_LENGTH, "The length of the word exceeds 6");
                session.player_times.entry(user_id).or_insert_with(Vec::<String>::new).push(word);
                debug!("check world session.player_times is:{:?}",session.player_times);
                debug!("user_action is:{:?}",user_action);
                debug!("session.wordle is:{:?}",session.wordle);
                let send_msg_id = msg::send(session.wordle, user_action, 0).expect("Failed to send");
                debug!("start check word send_msg_id is:{:?}",send_msg_id);
                let origin_id = msg::id();
                session.player_game_status.insert(user_id, GameStatus::CheckWordMessageSend {
                    origin_id,
                    send_id: send_msg_id
                });
                exec::wait();
            },
        }
    }else{
        let msg_status = player_game_status.expect("player status is empty").clone();
        debug!("received msg_status is:{:?}",msg_status);
        match msg_status{
            GameStatus::StartGameMessageReceived {event} => {
                // 获取用户id
                let game_status = player_game_status.expect("Failed to get status");
                debug!("received game_status is:{:?}",game_status);
                session.player_game_status.remove(&user_id);
                session.player_start_games.insert(user_id,true);
                msg::reply(event,0).expect("Failed to reply");

            }
            GameStatus::CheckWordMessageReceived{event} => {

                debug!("received check word message id is:{:?}",msg_id);
                // 获取用户id
                debug!("received checked user id is :{:?}",user_id);
                let game_status = player_game_status.expect("Failed to get status").clone();
                debug!("check word game_status is:{:?}",game_status);
                debug!("check word event is:{:?}",event);
                //清空用户的状态
                session.player_game_status.remove(&user_id);
                //检查用户是否结束了游戏
                match event.clone() {
                    Event::WordChecked{ user,
                        correct_positions,
                        contained_in_word,} => {
                        debug!("session.player_times.get(&user_id).expect(\"Failed to get times\").len() is :{:?}",session.player_times.get(&user_id));
                        if !(correct_positions.contains(&0)||session.player_times.get(&user_id).expect("Failed to get times").len() as u32==session.max_play_times){//游戏结束
                            session.player_times.remove(&user_id);
                            session.player_start_games.remove(&user_id);
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