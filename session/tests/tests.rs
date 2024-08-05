use gtest::{Log, Program, ProgramBuilder, System};
use session_io::Action::{CheckWord, StartGame};
use session_io::{GameSessionInit, SessionEvent};
use wordle_io::Event;

const GAME_SESSION_PROGRAM_ID: u64 = 1;
const GAME_WORDLE_PROGRAM_ID: u64 = 2;
const USER: u64 = 20;

fn init_game(system: &System) -> (Program, Program) {
    let session_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/session.opt.wasm")
            .with_id(GAME_SESSION_PROGRAM_ID)
            .with_meta_file("../target/wasm32-unknown-unknown/debug/session.meta.txt")
            .build(&system);
    let wordle_program =
        ProgramBuilder::from_file("../target/wasm32-unknown-unknown/debug/wordle.opt.wasm")
            .with_id(GAME_WORDLE_PROGRAM_ID)
            .with_meta_file("../target/wasm32-unknown-unknown/debug/wordle.meta.txt")
            .build(&system);

    let wordle_init_result = wordle_program.send::<u64, [u8; 0]>(USER, []);
    assert!(!wordle_init_result.main_failed(), "wordle init failed");

    let session_init_result = session_program.send(
        USER,
        GameSessionInit {
            wordle_address: GAME_WORDLE_PROGRAM_ID.into(),
            max_play_times: 3,
        },
    );
    assert!(!session_init_result.main_failed(), "session init success");
    (session_program, wordle_program)
}
#[test]
pub fn test_init() {
    let system = System::new();
    system.init_logger();

    let (session_program, wordle_program) = init_game(&system);
}
#[test]
pub fn test_play_success() {
    let system = System::new();
    system.init_logger();

    let (session_program, wordle_program) = init_game(&system);
    let start_result = session_program.send(USER, StartGame { user: USER.into() });
    assert!(!start_result.main_failed(), "start run failed");
    let start_logs = start_result.log();
    println!("start logs is:{:?}", start_logs);
    assert!(
        start_result.contains(&Log::builder().payload(Event::GameStarted { user: USER.into() })),
        "receive log error!"
    );

    //start test wordle
    let wordle_result = session_program.send(
        USER,
        CheckWord {
            user: USER.into(),
            word: "house".to_string(),
        },
    );
    assert!(!wordle_result.main_failed(), "wordle run failed");
    assert!(
        wordle_result.contains(&Log::builder().payload(SessionEvent::WordChecked {
            user: USER.into(),
            correct_positions: "0,1,3,4,".to_string(),
            contained_in_word: "".to_string(),
        }))
    );

    let success_wordle_result = session_program.send(
        USER,
        CheckWord {
            user: USER.into(),
            word: "horse".to_string(),
        },
    );

    assert!(!success_wordle_result.main_failed(), "wordle run failed");
    assert!(
        success_wordle_result.contains(&Log::builder().payload(SessionEvent::WordChecked {
            user: USER.into(),
            correct_positions: "0,1,2,3,4,".to_string(),
            contained_in_word: "".to_string(),
        }))
    );
}

#[test]
pub fn test_play_fail() {
    let system = System::new();
    system.init_logger();

    let (session_program, wordle_program) = init_game(&system);

    let wordle_result = session_program.send(
        USER,
        CheckWord {
            user: USER.into(),
            word: "house".to_string(),
        },
    );
    assert!(wordle_result.main_failed(), "wordle run failed");
}
