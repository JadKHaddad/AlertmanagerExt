use crate::{Alert, AlertmanagerPush, Status};
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use std::collections::BTreeMap;

fn generate_random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(25)
        .map(char::from)
        .collect()
}

fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn generate_random_btreemap() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let count = rand::thread_rng().gen_range(1..=5);

    for _ in 0..count {
        map.insert(generate_random_string(), generate_random_string());
    }

    map
}

fn generate_random_naive_date_time() -> chrono::NaiveDateTime {
    let now = Utc::now();
    let random_days: i64 = rand::thread_rng().gen_range(-30..=30);
    let random_timestamp = now + chrono::Duration::days(random_days);
    random_timestamp.naive_utc()
}

fn generate_option_random_naive_date_time() -> Option<chrono::NaiveDateTime> {
    if rand::random() {
        Some(generate_random_naive_date_time())
    } else {
        None
    }
}

fn generate_random_alert(n: usize) -> Alert {
    Alert {
        status: if rand::random() {
            Status::Resolved
        } else {
            Status::Firing
        },
        labels: generate_random_btreemap(),
        annotations: generate_random_btreemap(),
        starts_at: generate_random_naive_date_time(),
        ends_at: generate_option_random_naive_date_time(),
        generator_url: generate_random_string(),
        fingerprint: format!("{n}-{}", generate_uuid()),
    }
}

fn generate_random_alertmanager_push(n: usize) -> AlertmanagerPush {
    let count = rand::thread_rng().gen_range(1..=5);
    AlertmanagerPush {
        version: generate_random_string(),
        group_key: format!("{n}-{}", generate_uuid()),
        truncated_alerts: rand::thread_rng().gen(),
        status: if rand::random() {
            Status::Resolved
        } else {
            Status::Firing
        },
        receiver: generate_random_string(),
        group_labels: generate_random_btreemap(),
        common_labels: generate_random_btreemap(),
        common_annotations: generate_random_btreemap(),
        external_url: generate_random_string(),
        alerts: (0..count).map(generate_random_alert).collect(),
    }
}

pub fn generate_random_alertmanager_pushes(n: usize) -> Vec<AlertmanagerPush> {
    (0..n).map(generate_random_alertmanager_push).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn print_random_alertmanager_pushes() {
        let alertmanager_push = generate_random_alertmanager_pushes(10);
        println!("{:#?}", alertmanager_push);
    }
}
