use gstd::prelude::*;
use gtest::{Log, Program, System};
use wordle_io::{Action, Event};

#[test]
fn test_start_game() {
    let system = System::new();

    system.init_logger();

    let program = Program::current_opt(&system);

    let result = program.send_bytes(2, []);

    assert!(!result.main_failed(), "Program failed: {:?}", result);

    let start_game_result = program.send(2, Action::StartGame { user: 2.into() });

    assert!(
        !start_game_result.main_failed(),
        "Program failed: {:?}",
        start_game_result
    );

    start_game_result.contains(&Log::builder().payload(Action::StartGame { user: 2.into() }));
}
#[test]
fn test_wordle_game_success() {
    let system = System::new();

    system.init_logger();

    let program = Program::current_opt(&system);

    let result = program.send_bytes(2, []);

    assert!(!result.main_failed(), "Program failed: {:?}", result);

    let start_game_result = program.send(2, Action::StartGame { user: 2.into() });

    assert!(
        !start_game_result.main_failed(),
        "Program failed: {:?}",
        start_game_result
    );

    assert!(
        start_game_result.contains(&Log::builder().payload(Action::StartGame { user: 2.into() }))
    );

    let wordle_result = program.send(
        2,
        Action::CheckWord {
            user: 2.into(),
            word: "house".to_string(),
        },
    );
    assert!(
        !wordle_result.main_failed(),
        "Program failed: {:?}",
        wordle_result
    );
    assert!(
        wordle_result.contains(&Log::builder().payload(Event::WordChecked {
            user: 2.into(),
            correct_positions: vec![0, 1, 2, 3, 4],
            contained_in_word: vec![],
        }))
    );
    let result_event = wordle_result.decoded_log::<Event>();
    println!("word result is:{:?}", result_event);
}

#[test]
fn test_wordle_game_more_times() {
    let system = System::new();

    system.init_logger();

    let program = Program::current_opt(&system);

    let result = program.send_bytes(2, []);

    assert!(!result.main_failed(), "Program failed: {:?}", result);

    let start_game_result = program.send(2, Action::StartGame { user: 2.into() });

    assert!(
        !start_game_result.main_failed(),
        "Program failed: {:?}",
        start_game_result
    );

    assert!(
        start_game_result.contains(&Log::builder().payload(Action::StartGame { user: 2.into() }))
    );

    let wordle_result = program.send(
        2,
        Action::CheckWord {
            user: 2.into(),
            word: "human".to_string(),
        },
    );
    let result_event = wordle_result.decoded_log::<Event>();
    println!("word result is:{:?}", result_event);
    assert!(
        !wordle_result.main_failed(),
        "Program failed: {:?}",
        wordle_result
    );
    assert!(
        wordle_result.contains(&Log::builder().payload(Event::WordChecked {
            user: 2.into(),
            correct_positions: vec![0],
            contained_in_word: vec![1],
        }))
    );

    let wordle_result = program.send(
        2,
        Action::CheckWord {
            user: 2.into(),
            word: "heuan".to_string(),
        },
    );
    let result_event = wordle_result.decoded_log::<Event>();
    println!("word result is:{:?}", result_event);
    assert!(
        !wordle_result.main_failed(),
        "Program failed: {:?}",
        wordle_result
    );
    assert!(
        wordle_result.contains(&Log::builder().payload(Event::WordChecked {
            user: 2.into(),
            correct_positions: vec![0, 2],
            contained_in_word: vec![1],
        }))
    );
    let wordle_result = program.send(
        2,
        Action::CheckWord {
            user: 2.into(),
            word: "house".to_string(),
        },
    );
    let result_event = wordle_result.decoded_log::<Event>();
    println!("word result is:{:?}", result_event);
    assert!(
        !wordle_result.main_failed(),
        "Program failed: {:?}",
        wordle_result
    );
    assert!(
        wordle_result.contains(&Log::builder().payload(Event::WordChecked {
            user: 2.into(),
            correct_positions: vec![0, 1, 2, 3, 4],
            contained_in_word: vec![],
        }))
    );
}
