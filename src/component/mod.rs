pub mod context_list;
pub mod highlight;
pub mod password;
pub mod pop_windows;
pub mod request_ui;
pub mod toggle;
pub mod template_tools;

pub use password::password;
pub use toggle::toggle;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_chrono() {
        let now = Local::now();
        println!("{}", now.timestamp_millis());
    }
}
