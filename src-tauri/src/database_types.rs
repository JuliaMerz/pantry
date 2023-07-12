


use diesel::deserialize::{FromSql};

use diesel::query_builder::QueryId;
use diesel::serialize::{self, Output, ToSql};

use diesel::*;
use uuid::Uuid;


use diesel::sqlite::{Sqlite, SqliteValue};
use serde_json;
use serde_json::{Value};
use std::collections::HashMap;

use std::ops::Deref;
use std::path::PathBuf;


// Type conversions for diesel database usage.
//

// SerdeJsonIntMap(HashMap<String, isize>);
// SerdeJsonVec(Vec<String>);
// SerdeJsonHashMap(HashMap<String, Value>);
// SerdeJsonRwTimestamp(RwLock<Option<DateTime<Utc>>>);
// SerdePathBuf(PathBuf);

#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct DbHashMapInt(pub HashMap<String, isize>);

impl FromSql<diesel::sql_types::Text, Sqlite> for DbHashMapInt {
    fn from_sql(bytes: SqliteValue<'_, '_, '_>) -> diesel::deserialize::Result<Self> {
        let str = <String as FromSql<diesel::sql_types::Text, Sqlite>>::from_sql(bytes)?;
        let value: HashMap<String, isize> = serde_json::from_str(&str)?;
        Ok(DbHashMapInt(value))
    }
}

impl ToSql<DbHashMapInt, Sqlite> for DbHashMapInt {
    fn to_sql<'W>(&'W self, out: &mut Output<'W, '_, Sqlite>) -> serialize::Result {
        let str = serde_json::to_string(&self.0)?;
        // str.to_sql(out);
        out.set_value(str);
        Ok(serialize::IsNull::No)
    }
}

impl Deref for DbHashMapInt {
    type Target = HashMap<String, isize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// For Vec<String>
#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct DbVecString(pub Vec<String>);

impl FromSql<diesel::sql_types::Text, Sqlite> for DbVecString {
    fn from_sql(bytes: SqliteValue<'_, '_, '_>) -> diesel::deserialize::Result<Self> {
        let str = <String as FromSql<diesel::sql_types::Text, Sqlite>>::from_sql(bytes)?;
        let value: Vec<String> = serde_json::from_str(&str)?;
        Ok(DbVecString(value))
    }
}

impl ToSql<diesel::sql_types::Text, Sqlite> for DbVecString {
    fn to_sql<'W>(&'W self, out: &mut serialize::Output<'W, '_, Sqlite>) -> serialize::Result {
        let str = serde_json::to_string(&self.0)?;
        out.set_value(str);
        Ok(serialize::IsNull::No)
    }
}

impl Deref for DbVecString {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct DbVec<T>(pub Vec<T>);

impl<T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug>
    FromSql<diesel::sql_types::Text, Sqlite> for DbVec<T>
{
    fn from_sql(bytes: SqliteValue<'_, '_, '_>) -> diesel::deserialize::Result<Self> {
        let str = <String as FromSql<diesel::sql_types::Text, Sqlite>>::from_sql(bytes)?;
        let value: Vec<T> = serde_json::from_str(&str)?;
        Ok(DbVec(value))
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug>
    ToSql<diesel::sql_types::Text, Sqlite> for DbVec<T>
{
    fn to_sql<'W>(&'W self, out: &mut serialize::Output<'W, '_, Sqlite>) -> serialize::Result {
        let str = serde_json::to_string(&self.0)?;
        out.set_value(str);
        Ok(serialize::IsNull::No)
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug> Deref for DbVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// For HashMap<String, Value>
#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct DbHashMap(pub HashMap<String, Value>);

impl FromSql<diesel::sql_types::Text, Sqlite> for DbHashMap {
    fn from_sql(bytes: SqliteValue<'_, '_, '_>) -> diesel::deserialize::Result<Self> {
        let str = <String as FromSql<diesel::sql_types::Text, Sqlite>>::from_sql(bytes)?;
        let value: HashMap<String, Value> = serde_json::from_str(&str)?;
        Ok(DbHashMap(value))
    }
}

impl ToSql<diesel::sql_types::Text, Sqlite> for DbHashMap {
    fn to_sql<'W>(&self, out: &mut serialize::Output<'W, '_, Sqlite>) -> serialize::Result {
        let str = serde_json::to_string(&self.0)?;
        out.set_value(str);
        Ok(serialize::IsNull::No)
    }
}

impl Deref for DbHashMap {
    type Target = HashMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// // For RwLock<Option<DateTime<Utc>>>
// #[derive(serde::Deserialize, serde::Serialize, Debug, Clone, FromSqlRow, AsExpression)]
// #[diesel(sql_type = diesel::sql_types::Text)]
// pub struct DbRwDateTime(pub RwLock<Option<DateTime<Utc>>>);

// impl PartialEq for DbRwDateTime {
//     fn eq(&self, other: &Self) -> bool {
//         let self_value = self.0.read().expect("Failed to acquire read lock");
//         let other_value = other.0.read().expect("Failed to acquire read lock");
//         self_value == other_value
//     }
// }

// impl FromSql<diesel::sql_types::Nullable<diesel::sql_types::TimestamptzSqlite>, Sqlite>
//     for DbRwDateTime
// {
//     fn from_sql(bytes: SqliteValue<'_, '_, '_>) -> diesel::deserialize::Result<Self> {
//         let timestamp = <Option<DateTime<Utc>> as FromSql<
//             diesel::sql_types::Nullable<diesel::sql_types::TimestamptzSqlite>,
//             Sqlite,
//         >>::from_sql(bytes)?;
//         Ok(DbRwDateTime(RwLock::new(timestamp)))
//     }
// }

// impl ToSql<diesel::sql_types::Nullable<diesel::sql_types::TimestamptzSqlite>, Sqlite>
//     for DbRwDateTime
// {
//     fn to_sql<'W>(&self, out: &mut serialize::Output<'W, '_, Sqlite>) -> serialize::Result {
//         let value = self.0.read().unwrap().deref();
//         match value {
//             Some(inner) => <chrono::DateTime<chrono::Utc> as ToSql<
//                 diesel::sql_types::TimestamptzSqlite,
//                 Sqlite,
//             >>::to_sql(&inner.clone(), out),
//             None => Ok(serialize::IsNull::Yes),
//         }

//         // Ok(serialize::IsNull::No)
//     }
// }

// impl Deref for DbRwDateTime {
//     type Target = RwLock<Option<DateTime<Utc>>>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// // For RwLock<Option<DateTime<Utc>>>
// #[derive(serde::Deserialize, serde::Serialize, Debug, FromSqlRow, AsExpression)]
// #[diesel(sql_type = diesel::sql_types::Text)]
// pub struct DbDateTime(pub DateTime<Utc>);

// impl FromSql<diesel::sql_types::Text, Sqlite> for DbDateTime {
//     fn from_sql(bytes: SqliteValue<'_, '_, '_>) -> diesel::deserialize::Result<Self> {
//         let str = <String as FromSql<diesel::sql_types::Text, Sqlite>>::from_sql(bytes)?;
//         let value: Option<DateTime<Utc>> = serde_json::from_str(&str)?;
//         Ok(DbDateTime(RwLock::new(value)))
//     }
// }

// impl ToSql<diesel::sql_types::Text, Sqlite> for DbRwDateTime {
//     fn to_sql<'W>(&self, out: &mut serialize::Output<'W, '_, Sqlite>) -> serialize::Result {
//         let value = self.0.read().unwrap();
//         let str = serde_json::to_string(&*value)?;
//         out.set_value(str);
//         Ok(serialize::IsNull::No)
//     }
// }

// impl Deref for DbRwDateTime {
//     type Target = RwLock<Option<DateTime<Utc>>>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// For PathBuf
#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct DbPathBuf(pub PathBuf);

impl FromSql<diesel::sql_types::Text, Sqlite> for DbPathBuf {
    fn from_sql(bytes: SqliteValue<'_, '_, '_>) -> diesel::deserialize::Result<Self> {
        let str = <String as FromSql<diesel::sql_types::Text, Sqlite>>::from_sql(bytes)?;
        Ok(DbPathBuf(PathBuf::from(str)))
    }
}

impl ToSql<diesel::sql_types::Text, Sqlite> for DbPathBuf {
    fn to_sql<'W>(&self, out: &mut serialize::Output<'W, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.0.to_str().unwrap().clone().to_owned());
        Ok(serialize::IsNull::No)
    }
}

impl Deref for DbPathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct DbOptionPathbuf(pub Option<PathBuf>);

impl FromSql<diesel::sql_types::Nullable<diesel::sql_types::Text>, Sqlite> for DbOptionPathbuf {
    fn from_sql(bytes: SqliteValue<'_, '_, '_>) -> diesel::deserialize::Result<Self> {
        let str = <Option<String> as FromSql<
            diesel::sql_types::Nullable<diesel::sql_types::Text>,
            Sqlite,
        >>::from_sql(bytes)?;
        match str {
            Some(s) => Ok(DbOptionPathbuf(Some(PathBuf::from(s)))),
            None => Ok(DbOptionPathbuf(None)),
        }
    }
}

impl ToSql<diesel::sql_types::Nullable<diesel::sql_types::Text>, Sqlite> for DbOptionPathbuf {
    fn to_sql<'W>(&self, out: &mut serialize::Output<'W, '_, Sqlite>) -> serialize::Result {
        match self.0.clone() {
            Some(path) => {
                out.set_value(path.to_str().unwrap().clone().to_owned());
                Ok(serialize::IsNull::No)
            }
            None => Ok(serialize::IsNull::Yes),
        }
    }
}

impl Deref for DbOptionPathbuf {
    type Target = Option<PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// For Uuid
#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct DbUuid(pub Uuid);

impl FromSql<diesel::sql_types::Text, Sqlite> for DbUuid {
    fn from_sql(bytes: SqliteValue<'_, '_, '_>) -> diesel::deserialize::Result<Self> {
        let str = <String as FromSql<diesel::sql_types::Text, Sqlite>>::from_sql(bytes)?;
        Ok(DbUuid(Uuid::parse_str(&str)?))
    }
}

impl ToSql<diesel::sql_types::Text, Sqlite> for DbUuid {
    fn to_sql<'W>(&self, out: &mut serialize::Output<'W, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.0.to_string());
        Ok(serialize::IsNull::No)
    }
}

impl Deref for DbUuid {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
// Add QueryId implementation
impl QueryId for DbUuid {
    type QueryId = Self;

    const HAS_STATIC_QUERY_ID: bool = true;
}
