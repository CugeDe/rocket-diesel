#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub firstname: String,
}