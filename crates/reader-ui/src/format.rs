//! Presentation formatting keeps Reader's compact counters separate from model totals.
pub fn unread(count: usize) -> String {
    if count >= 1000 {
        "1000+".into()
    } else {
        count.to_string()
    }
}
pub fn unread_badge(count: usize) -> String {
    if count == 0 {
        String::new()
    } else {
        format!("({})", unread(count))
    }
}
