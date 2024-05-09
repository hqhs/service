use diesel::prelude::*;

use serde::Serialize;

#[derive(Default, Serialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    pub id: Option<i32>,
    pub title: String,
    pub body: String,
}
