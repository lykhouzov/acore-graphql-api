use async_graphql::Object;
use struct_field_names_as_array::FieldNamesAsArray;

#[derive(Clone, Debug, Default, sqlx::FromRow, FieldNamesAsArray)]
pub struct Access {
    #[sqlx(default)]
    id: u64,
    #[sqlx(default)]
    gmlevel: u8,
    #[sqlx(default)]
    realmid: i32,
    #[sqlx(default)]
    comment: Option<String>,
}

#[Object]
impl Access {
    async fn id(&self) -> u64 {
        self.id.clone()
    }
    async fn gmlevel(&self) -> u8 {
        self.gmlevel.clone()
    }
    async fn realmid(&self) -> i32 {
        self.realmid.clone()
    }
    async fn comment(&self) -> Option<String> {
        self.comment.clone()
    }
}
