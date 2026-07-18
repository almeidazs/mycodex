use super::format_session_timer;
use pretty_assertions::assert_eq;
use std::time::Duration;

#[test]
fn session_timer_formats_elapsed_duration() {
    assert_eq!(format_session_timer(Duration::from_secs(0)), "0s");
    assert_eq!(format_session_timer(Duration::from_secs(12)), "12s");
    assert_eq!(
        format_session_timer(Duration::from_secs(3 * 60 + 42)),
        "03m 42s"
    );
    assert_eq!(
        format_session_timer(Duration::from_secs(68 * 60 + 59)),
        "01h 08m"
    );
}
