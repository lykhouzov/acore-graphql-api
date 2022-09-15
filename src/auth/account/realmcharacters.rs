use async_graphql::Object;
use struct_field_names_as_array::FieldNamesAsArray;

#[derive(Clone, Debug, Default, sqlx::FromRow, FieldNamesAsArray)]
pub struct RealmCharacter {
    #[sqlx(default)]
    realmid: u64,
    #[sqlx(default)]
    acctid: u64,
    #[sqlx(default)]
    numchars: u8,
    #[sqlx(default)]
    realmname: String,
}

#[Object]
impl RealmCharacter {
    async fn realmid(&self) -> u64 {
        self.realmid.clone()
    }
    async fn acctid(&self) -> u64 {
        self.acctid.clone()
    }
    async fn numchars(&self) -> u8 {
        self.numchars.clone()
    }
    async fn realmname(&self) -> String {
        self.realmname.clone()
    }
}
