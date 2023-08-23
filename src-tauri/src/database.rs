use chrono::{DateTime, Utc};
use diesel::debug_query;
use diesel::dsl::sql;
use diesel::internal::table_macro::SelectStatement;
use diesel::prelude::*;
use diesel::query_builder::AsQuery;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_types::*;
use diesel::sqlite::{Sqlite, SqliteType};
use diesel::*;
use serde;
use std::fmt;
use uuid::Uuid;

use crate::database_types::*;
use crate::llm::{LLMHistoryItem, LLMSession, LLM};
use crate::request::UserRequest;
use crate::schema;
use crate::user;
use crate::user::User;
// ON db migration generation:
// %s/Timestamp/TimestamptzSqlite/g

pub fn get_llm(
    llm_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLM, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm::dsl::*;
    llm.filter(uuid.eq(DbUuid(llm_id)))
        .select(LLM::as_select())
        .first(conn)
}

pub fn count_llm_by_pub_id(
    llm_id: String,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<i64, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm::dsl::*;
    llm.filter(id.eq(llm_id))
        .select(diesel::dsl::count(uuid))
        .get_result(conn)
}

pub fn get_llm_pub_id(
    llm_id: String,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLM, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm::dsl::*;
    llm.filter(id.eq(llm_id))
        .select(LLM::as_select())
        .first(conn)
}

pub fn get_available_llms(
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<LLM>, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm::dsl::*;
    llm.select(LLM::as_select()).load(conn)
}

pub fn get_llm_session(
    llm_session_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMSession, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm_session::dsl::*;
    llm_session
        .filter(id.eq(DbUuid(llm_session_id)))
        .select(LLMSession::as_select())
        .first(conn)
}

pub fn get_llm_history(
    llm_history_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMHistoryItem, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm_history::dsl::*;
    llm_history
        .filter(id.eq(DbUuid(llm_history_id)))
        .select(LLMHistoryItem::as_select())
        .first(conn)
}

pub fn get_requests(
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<UserRequest>, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::requests::dsl::*;
    requests.select(UserRequest::as_select()).load(conn)
}

pub fn get_request(
    request_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<UserRequest, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::requests::dsl::*;
    requests
        .filter(id.eq(DbUuid(request_id)))
        .select(UserRequest::as_select())
        .first(conn)
}

pub fn get_user(
    user_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<User, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::user::dsl::*;
    user.filter(id.eq(DbUuid(user_id)))
        .select(User::as_select())
        .first(conn)
}

pub fn get_sessions_for_llm(
    llm_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<LLMSession>, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm_session::dsl::*;
    llm_session
        .filter(llm_uuid.eq(DbUuid(llm_id)))
        .select(LLMSession::as_select())
        .load(conn)
}

pub fn get_history_for_session(
    llm_session_id_val: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<LLMHistoryItem>, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm_history::dsl::*;
    llm_history
        .filter(llm_session_id.eq(DbUuid(llm_session_id_val)))
        .select(LLMHistoryItem::as_select())
        .load(conn)
}

// We should just do this when we update the session.
// pub fn update_llm_last_called(
//     llm: LLM,
//     pool: Pool<ConnectionManager<SqliteConnection>>,
// ) -> Result<LLM, diesel::result::Error> {
//     let conn = &mut pool.get().unwrap();
//     use schema::llm::dsl::*;
//     todo!()
// }

pub fn update_last_called(
    llm_session: LLMSession,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMSession, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm::dsl as llm_dsl;
    use schema::llm_session::dsl as session_dsl;
    diesel::update(llm_dsl::llm)
        .filter(llm_dsl::uuid.eq(llm_session.llm_uuid))
        .set(llm_dsl::last_called.eq(Utc::now()))
        .execute(conn)?;
    diesel::update(session_dsl::llm_session)
        .filter(session_dsl::id.eq(llm_session.id.clone()))
        .set(session_dsl::last_called.eq(Utc::now()))
        .execute(conn)?;
    get_llm_session(llm_session.id.0, pool)
}

// set complete bool at the same time
pub fn append_token(
    llm_history_item: LLMHistoryItem,
    t: String,
    prompt_complete: bool,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMHistoryItem, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    let llm_history_id = llm_history_item.id.0.clone();
    use schema::llm_history::dsl::*;
    diesel::update(llm_history)
        .filter(id.eq(llm_history_item.id))
        .set((
            output.eq(output.concat(t)),
            updated_timestamp.eq(Utc::now()),
            complete.eq(prompt_complete),
        ))
        .execute(conn)?;
    get_llm_history(llm_history_id, pool)
}

pub fn save_new_llm(
    new_llm: LLM,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLM, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm::dsl::*;
    let llm_id = new_llm.uuid.0.clone();
    diesel::insert_into(llm).values(&new_llm).execute(conn)?;
    get_llm(llm_id, pool)
}

pub fn save_new_llm_session(
    new_llm_session: LLMSession,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMSession, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm_session::dsl::*;
    let llm_session_id = new_llm_session.id.0.clone();
    diesel::insert_into(llm_session)
        .values(&new_llm_session)
        .execute(conn)?;
    get_llm_session(llm_session_id, pool)
}

pub fn save_new_llm_history(
    new_llm_history: LLMHistoryItem,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMHistoryItem, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm_history::dsl::*;
    let llm_history_id = new_llm_history.id.0.clone();
    diesel::insert_into(llm_history)
        .values(&new_llm_history)
        .execute(conn)?;
    get_llm_history(llm_history_id, pool)
    // Remember to write a smoooooooth update statement to update session last called
}

pub fn save_new_request(
    new_request: UserRequest,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<UserRequest, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::requests::dsl::*;
    let request_id = new_request.id.0.clone();
    diesel::insert_into(requests)
        .values(&new_request)
        .execute(conn)?;
    get_request(request_id, pool)
}

pub fn save_new_user(
    new_user: User,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<User, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::user::dsl::*;
    let user_id = new_user.id.0.clone();
    diesel::insert_into(user).values(&new_user).execute(conn)?;
    get_user(user_id, pool)
}

pub fn get_llm_sessions_user(
    user: User,
    llm_id: DbUuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<(LLMSession, Vec<LLMHistoryItem>)>, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm_history::dsl as history_dsl;
    use schema::llm_session::dsl as session_dsl;
    let sessions = session_dsl::llm_session
        .filter(session_dsl::llm_uuid.eq(llm_id))
        .filter(session_dsl::user_id.eq(user.id))
        .order(session_dsl::last_called.desc())
        .select(LLMSession::as_select())
        .load(conn)?;

    let history_items = LLMHistoryItem::belonging_to(&sessions)
        .select(LLMHistoryItem::as_select())
        .load(conn)?;

    Ok(history_items
        .grouped_by(&sessions)
        .into_iter()
        .zip(sessions)
        .map(|(items, session)| (session, items))
        .collect::<Vec<(LLMSession, Vec<LLMHistoryItem>)>>())
}

pub fn delete_llm(
    llm_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<usize, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm::dsl::*;
    diesel::delete(llm)
        .filter(uuid.eq(DbUuid(llm_id)))
        .execute(conn)
}

pub fn get_llm_by_url(
    reg_url: String,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLM, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::llm::dsl::*;
    llm.filter(url.eq(reg_url))
        .select(LLM::as_select())
        .first(conn)
}

pub fn mark_request_complete(
    req_id: Uuid,
    accepted: bool,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<usize, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::requests::dsl;
    diesel::update(dsl::requests)
        .filter(dsl::id.eq(DbUuid(req_id)))
        .set((dsl::accepted.eq(accepted), dsl::complete.eq(true)))
        .execute(conn)
}

pub fn update_permissions(
    user_id: Uuid,
    perms: user::Permissions,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<usize, diesel::result::Error> {
    let conn = &mut pool.get().unwrap();
    use schema::user::dsl::*;
    println!("Updating to {:?}", perms);
    diesel::update(user)
        .filter(id.eq(DbUuid(user_id)))
        .set((
            perm_superuser.eq(perms.perm_superuser),
            perm_load_llm.eq(perms.perm_load_llm),
            perm_unload_llm.eq(perms.perm_unload_llm),
            perm_download_llm.eq(perms.perm_download_llm),
            perm_session.eq(perms.perm_session),
            perm_request_download.eq(perms.perm_request_download),
            perm_request_load.eq(perms.perm_request_load),
            perm_request_unload.eq(perms.perm_request_unload),
            perm_view_llms.eq(perms.perm_view_llms),
            perm_bare_model.eq(perms.perm_bare_model),
        ))
        .execute(conn)
}

// MAGIC SAUCE
//
// pub fn filter_prefer_llm_query_builder(
//     filter: LLMFilter,
//     preference: LLMPreference,
// ) -> schema::llm::BoxedQuery<Sqlite> {
//     use schema::llm::dsl::*;
//     let mut query = llm.select(LLM::as_select()).into_boxed();
//     if let Some(llm_uuid_filter) = filter.llm_uuid {
//         query = query.filter(uuid.eq(DbUuid(llm_uuid_filter)))
//     }
//     if let Some(llm_id_filter) = filter.llm_id {
//         query = query.filter(id.eq(llm_id_filter))
//     }
//     if let Some(family_id_filter) = filter.family_id {
//         query = query.filter(family_id.eq(family_id_filter))
//     }

//     if let Some(local_filter) = filter.local {
//         query = query.filter(local.eq(local_filter));
//     }

//     if let Some(capabilities_filter) = filter.minimum_capabilities {
//         for cap_fil in capabilities_filter.into_iter() {
//             let capability_name = cap_fil.capability;
//             let capability_min = cap_fil.value;
//             query = query.filter(
//                 sql::<Bool>("json_extract(capabilities, ")
//                     .bind::<diesel::sql_types::Text, String>(format!(
//                         "$.{}",
//                         capability_name.to_string()
//                     ))
//                     .sql(") > ")
//                     .bind::<diesel::sql_types::Integer, i32>(capability_min),
//             );
//         }
//     }

//     let debug = debug_query::<Sqlite, _>(&query);

//     println!("Built query: {:?}", debug);
//     query
// }

//pub fn filter_prefer_llm(
//    filter: LLMFilter,
//    preference: LLMPreference,
//    pool: Pool<ConnectionManager<SqliteConnection>>,
//) -> Result<LLM, diesel::result::Error> {
//    //filter in order
//    use schema::llm::dsl::*;

//    let conn = &mut pool.get().unwrap();

//    // let mut query: Box<dyn BoxableExpression<llm, Sqlite, SqlType = LLM>> = llm.select(LLM::as_select()).into_boxed();
//    let mut query = llm.select(LLM::as_select()).into_boxed();
//    if let Some(llm_uuid_filter) = filter.llm_uuid {
//        query = query.filter(uuid.eq(DbUuid(llm_uuid_filter)))
//    }
//    if let Some(llm_id_filter) = filter.llm_id {
//        query = query.filter(id.eq(llm_id_filter))
//    }
//    if let Some(family_id_filter) = filter.family_id {
//        query = query.filter(family_id.eq(family_id_filter))
//    }

//    if let Some(local_filter) = filter.local {
//        query = query.filter(local.eq(local_filter));
//    }

//    if let Some(capabilities_filter) = filter.minimum_capabilities {
//        for cap_fil in capabilities_filter.into_iter() {
//            let capability_name = cap_fil.capability;
//            let capability_min = cap_fil.value;
//            query = query.filter(
//                sql::<Bool>("json_extract(capabilities, ")
//                    .bind::<diesel::sql_types::Text, String>(format!(
//                        "$.{}",
//                        capability_name.to_string()
//                    ))
//                    .sql(") > ")
//                    .bind::<diesel::sql_types::Integer, i32>(capability_min),
//            );
//        }
//    }

//    let debug = debug_query::<Sqlite, _>(&query);
//    println!("Built query: {:?}", debug);

//    let raw_count: i64 = query.count().get_result(conn)?;
//    if raw_count == 1 {
//        println!("only one left, sending it out!");
//        return query.clone().first(conn);
//    } else if raw_count == 0 {
//        return Err(diesel::result::Error::NotFound);
//    }

//    //llm filter
//    //We dont' make this permanent, since it exists or it doesn't
//    if let Some(uuid_pref) = preference.llm_uuid {
//        let query_uuid_pref = query.filter(uuid.eq(DbUuid(uuid_pref)));
//        if let Ok(out) = query_uuid_pref.first(conn) {
//            return Ok(out);
//        }
//    }

//    if let Some(id_pref) = preference.llm_id {
//        let query_id_pref = query.clone().filter(id.eq(id_pref));
//        let llm_id_count: i64 = query_id_pref.clone().count().get_result(conn)?;
//        if llm_id_count == 1 {
//            return query_id_pref.first(conn);
//        } else if llm_id_count > 1 {
//            //We have more than one, so we can apply the preference and go down the stack
//            query = query_id_pref;
//        }
//    }

//    if let Some(local_pref) = preference.local {
//        let query_local_pref = query.clone().filter(local.eq(local_pref));
//        let llm_local_count: i64 = query_local_pref.clone().count().get_result(conn)?;
//        if llm_local_count == 1 {
//            return query_local_pref.first(conn);
//        } else if llm_local_count > 1 {
//            //We have more than one, so we can apply the preference and go down the stack
//            query = query_local_pref;
//        }
//    }

//    if let Some(family_pref) = preference.family_id {
//        let query_family_pref = query.clone().filter(family_id.eq(family_pref));
//        let llm_family_count: i64 = query_family_pref.clone().count().get_result(conn)?;
//        if llm_family_count == 1 {
//            return query_family_pref.first(conn);
//        } else if llm_family_count > 1 {
//            //We have more than one, so we can apply the preference and go down the stack
//            query = query_family_pref;
//        }
//    }

//    let capability_pref = match preference.capability_type {
//        Some(cap) => cap,
//        None => CapabilityType::General,
//    };

//    query
//        .order(
//            sql::<Bool>("json_extract(capabilities, ")
//                .bind::<diesel::sql_types::Text, String>(format!(
//                    "$.{}",
//                    capability_pref.to_string()
//                ))
//                .sql(")"),
//        )
//        .first(conn)
//}
