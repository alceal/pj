use chrono::Utc;

const HOUR: i64 = 3600;
const DAY: i64 = 24 * HOUR;
const WEEK: i64 = 7 * DAY;

pub fn calculate_frecency(last_accessed: i64, access_count: u32) -> f64 {
    let now = Utc::now().timestamp();
    let age = now - last_accessed;

    let recency_weight = if age < HOUR {
        4.0
    } else if age < DAY {
        2.0
    } else if age < WEEK {
        1.0
    } else {
        0.5
    };

    let frequency_score = (access_count as f64).ln_1p();

    recency_weight * frequency_score + recency_weight
}
