pub fn to_duration(seconds: i32, plus_prefix: bool) -> String {
    let hours = seconds.abs() / 3600;
    let minutes = (seconds.abs() % 3600) / 60;

    if hours == 0 && minutes == 0 {
        return "0h".to_string();
    }

    let mut duration = String::new();

    if seconds < 0 {
        duration += "-";
    }

    if seconds > 0 && plus_prefix {
        duration += "+";
    }

    if hours > 0 {
        duration += &format!("{}h", hours);
    }

    if minutes > 0 {
        duration += &format!("{}m", minutes);
    }

    duration
}
