use ormx::Table;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Table)]
#[ormx(table = "alerts", id = id, insertable)]
pub struct Alert {
    #[ormx(default)]
    pub id: u64,
    pub group_key: String,
    pub status: String,
    pub starts_at: chrono::DateTime<chrono::Utc>,
    pub ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub generator_url: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, FromRow, Table)]
#[ormx(table = "alerts_labels", id = id, insertable)]
pub struct AlertLabel {
    #[ormx(default)]
    pub id: u64,
    pub alert_id: u64,
    pub label_id: u64,
}

#[derive(Debug, Clone, FromRow)]
pub struct AlertAnnotation {
    pub alert_id: u64,
    pub annotation_id: u64,
}
