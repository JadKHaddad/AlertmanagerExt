use models::Status as AlermanagerPushStatus;
use sqlx::Type;

#[derive(Clone, Debug, PartialEq, PartialOrd, Type)]
#[sqlx(type_name = "alert_status", rename_all = "lowercase")]
pub enum AlertStatusModel {
    Resolved,
    Firing,
}

impl From<&AlermanagerPushStatus> for AlertStatusModel {
    fn from(status: &AlermanagerPushStatus) -> Self {
        match status {
            AlermanagerPushStatus::Resolved => AlertStatusModel::Resolved,
            AlermanagerPushStatus::Firing => AlertStatusModel::Firing,
        }
    }
}

impl From<AlertStatusModel> for AlermanagerPushStatus {
    fn from(status: AlertStatusModel) -> Self {
        match status {
            AlertStatusModel::Resolved => AlermanagerPushStatus::Resolved,
            AlertStatusModel::Firing => AlermanagerPushStatus::Firing,
        }
    }
}
