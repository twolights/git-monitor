//! Formatting helpers for git data.

/// Format a commit time relative to `now` (both in seconds since the Unix
/// epoch) as a short human string like "2 hours ago".
pub fn relative_date(commit_time_secs: i64, now_secs: i64) -> String {
    let diff = now_secs - commit_time_secs;
    if diff < 0 {
        return "in the future".to_string();
    }

    const MINUTE: i64 = 60;
    const HOUR: i64 = 60 * MINUTE;
    const DAY: i64 = 24 * HOUR;
    const WEEK: i64 = 7 * DAY;
    const MONTH: i64 = 30 * DAY;
    const YEAR: i64 = 365 * DAY;

    if diff <= 1 {
        return "just now".to_string();
    }
    if diff < MINUTE {
        return format!("{diff} seconds ago");
    }

    let (n, unit) = if diff < HOUR {
        (diff / MINUTE, "minute")
    } else if diff < DAY {
        (diff / HOUR, "hour")
    } else if diff < WEEK {
        (diff / DAY, "day")
    } else if diff < MONTH {
        (diff / WEEK, "week")
    } else if diff < YEAR {
        (diff / MONTH, "month")
    } else {
        (diff / YEAR, "year")
    };

    if n == 1 {
        format!("1 {unit} ago")
    } else {
        format!("{n} {unit}s ago")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn just_now() {
        assert_eq!(relative_date(1000, 1000), "just now");
        assert_eq!(relative_date(1000, 1001), "just now");
    }

    #[test]
    fn seconds() {
        assert_eq!(relative_date(1000, 1030), "30 seconds ago");
    }

    #[test]
    fn minutes() {
        assert_eq!(relative_date(0, 60), "1 minute ago");
        assert_eq!(relative_date(0, 125), "2 minutes ago");
    }

    #[test]
    fn hours() {
        assert_eq!(relative_date(0, 2 * 3600), "2 hours ago");
    }

    #[test]
    fn days_weeks_months_years() {
        assert_eq!(relative_date(0, 3 * 86_400), "3 days ago");
        assert_eq!(relative_date(0, 14 * 86_400), "2 weeks ago");
        assert_eq!(relative_date(0, 60 * 86_400), "2 months ago");
        assert_eq!(relative_date(0, 800 * 86_400), "2 years ago");
    }

    #[test]
    fn future() {
        assert_eq!(relative_date(100, 50), "in the future");
    }
}
