use crate::schema::*;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Insertable)]
#[table_name = "logs"]
pub struct NewLog {
    pub user_agent: String,
    pub response_time: i32,
    pub timestamp: NaiveDateTime,
}
