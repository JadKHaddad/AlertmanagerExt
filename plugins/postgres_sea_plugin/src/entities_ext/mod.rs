use crate::entities::sea_orm_active_enums::AlertStatus;
use models::Status as AlermanagerPushStatus;

impl From<&AlermanagerPushStatus> for AlertStatus {
    fn from(status: &AlermanagerPushStatus) -> Self {
        match status {
            AlermanagerPushStatus::Resolved => AlertStatus::Resolved,
            AlermanagerPushStatus::Firing => AlertStatus::Firing,
        }
    }
}
