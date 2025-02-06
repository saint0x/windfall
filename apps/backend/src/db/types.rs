use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(transparent)]
pub struct DbDateTime(pub NaiveDateTime);

impl Serialize for DbDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        DateTime::<Utc>::from_naive_utc_and_offset(self.0, Utc).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DbDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let dt = DateTime::<Utc>::deserialize(deserializer)?;
        Ok(Self(dt.naive_utc()))
    }
}

impl DbDateTime {
    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(dt.naive_utc())
    }

    pub fn now() -> Self {
        Self(Utc::now().naive_utc())
    }

    pub fn into_datetime(self) -> DateTime<Utc> {
        DateTime::from_naive_utc_and_offset(self.0, Utc)
    }
}

impl From<DateTime<Utc>> for DbDateTime {
    fn from(dt: DateTime<Utc>) -> Self {
        Self::new(dt)
    }
}

impl From<DbDateTime> for DateTime<Utc> {
    fn from(dt: DbDateTime) -> Self {
        dt.into_datetime()
    }
}

impl From<NaiveDateTime> for DbDateTime {
    fn from(dt: NaiveDateTime) -> Self {
        Self(dt)
    }
} 