use cpu_express::platform::{
    DesktopAction as Action, DesktopState as State, DesktopTransitionError,
};

#[test]
fn hiding_requires_a_working_recovery_path_in_every_live_state() {
    for state in [State::Visible, State::Minimized, State::HiddenByUser] {
        assert_eq!(
            state.after(Action::Hide, false),
            Err(DesktopTransitionError::NoRecoveryTray)
        );
        assert_eq!(state.after(Action::Hide, true), Ok(State::HiddenByUser));
    }
}

#[test]
fn visible_minimized_hidden_and_closed_are_distinct() {
    let visible = State::Visible;
    let minimized = visible.after(Action::Minimize, false).unwrap();
    assert_eq!(minimized, State::Minimized);
    let hidden = minimized.after(Action::Hide, true).unwrap();
    assert_eq!(hidden, State::HiddenByUser);
    assert_eq!(hidden.after(Action::Show, false), Ok(State::Visible));
    assert_eq!(hidden.after(Action::Close, false), Ok(State::Closed));
}

#[test]
fn closing_always_works_and_is_idempotent_even_without_a_tray() {
    for state in [
        State::Visible,
        State::Minimized,
        State::HiddenByUser,
        State::Closed,
    ] {
        for tray in [false, true] {
            assert_eq!(state.after(Action::Close, tray), Ok(State::Closed));
        }
    }
}

#[test]
fn closed_is_terminal_and_cannot_be_mistaken_for_hidden() {
    for action in [Action::Show, Action::Hide, Action::Minimize] {
        for tray in [false, true] {
            assert_eq!(
                State::Closed.after(action, tray),
                Err(DesktopTransitionError::AlreadyClosed)
            );
        }
    }
}

#[test]
fn losing_the_tray_recovers_only_hidden_windows() {
    for state in [
        State::Visible,
        State::Minimized,
        State::HiddenByUser,
        State::Closed,
    ] {
        assert_eq!(state.needs_recovery(false), state == State::HiddenByUser);
        assert!(!state.needs_recovery(true));
    }
}

#[test]
fn inspecting_a_failed_hide_does_not_commit_the_state() {
    let state = State::Visible;
    let proposal = state.after(Action::Hide, false);
    assert!(proposal.is_err());
    assert_eq!(state, State::Visible);
    assert_eq!(state.after(Action::Show, false), Ok(State::Visible));
}

#[test]
fn startup_invisibility_allows_first_paint_but_user_hiding_suspends() {
    use cpu_express::platform::DesktopState;

    assert!(!DesktopState::HiddenByUser.suspends_animation(false));
    assert!(DesktopState::HiddenByUser.suspends_animation(true));
    assert!(!DesktopState::Visible.suspends_animation(true));
    assert!(DesktopState::Minimized.suspends_animation(false));
    assert!(DesktopState::Closed.suspends_animation(false));
}
