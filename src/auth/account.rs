use async_graphql::{Context, Object, Result, Schema, Subscription};

use futures_util::Stream;
use log::debug;
use sqlx::types::chrono::{DateTime, Utc};
use struct_field_names_as_array::FieldNamesAsArray;

use self::{access::Access, realmcharacters::RealmCharacter};

use super::db::Storage;

pub mod access;
pub mod realmcharacters;

pub type ID = u64;

#[derive(Clone, Debug, Default, sqlx::FromRow, FieldNamesAsArray)]
pub struct Account {
    #[sqlx(default)]
    pub id: ID,
    #[sqlx(default)]
    pub username: String,
    #[sqlx(default)]
    pub salt: Vec<u8>,
    #[sqlx(default)]
    pub verifier: Vec<u8>,
    #[sqlx(default)]
    pub session_key: Option<Vec<u8>>,
    #[sqlx(default)]
    pub totp_secret: Option<Vec<u8>>,
    #[sqlx(default)]
    pub email: String,
    #[sqlx(default)]
    pub reg_mail: String,
    #[sqlx(default)]
    pub joindate: DateTime<Utc>,
    #[sqlx(default)]
    pub last_ip: String,
    #[sqlx(default)]
    pub last_attempt_ip: String,
    #[sqlx(default)]
    pub failed_logins: u32,
    #[sqlx(default)]
    pub locked: u8,
    #[sqlx(default)]
    pub lock_country: String,
    #[sqlx(default)]
    pub last_login: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub online: u8,
    #[sqlx(default)]
    pub expansion: u8,
    #[sqlx(default)]
    pub mutetime: i64,
    #[sqlx(default)]
    pub mutereason: String,
    #[sqlx(default)]
    pub muteby: String,
    #[sqlx(default)]
    pub locale: u8,
    #[sqlx(default)]
    pub os: String,
    #[sqlx(default)]
    pub recruiter: u32,
    #[sqlx(default)]
    pub totaltime: u32,
}

#[Object]
impl Account {
    async fn id(&self) -> u64 {
        self.id.clone()
    }
    async fn username(&self) -> String {
        self.username.clone()
    }

    async fn email(&self) -> String {
        self.email.clone()
    }
    async fn salt(&self) -> Vec<u8> {
        Vec::from(self.salt.as_slice())
    }

    async fn verifier(&self) -> Vec<u8> {
        Vec::from(self.verifier.as_slice())
    }
    async fn session_key(&self) -> Option<Vec<u8>> {
        self.session_key.clone()
    }
    async fn totp_secret(&self) -> Option<Vec<u8>> {
        self.totp_secret.clone()
    }

    async fn joindate(&self) -> String {
        self.joindate.to_rfc3339()
    }

    async fn last_ip(&self) -> String {
        self.last_ip.clone()
    }

    async fn last_attempt_ip(&self) -> String {
        self.last_attempt_ip.clone()
    }

    async fn failed_logins(&self) -> u32 {
        self.failed_logins.clone()
    }

    async fn locked(&self) -> u8 {
        self.locked.clone()
    }

    async fn lock_country(&self) -> String {
        self.lock_country.clone()
    }

    async fn last_login(&self) -> String {
        self.last_login
            .map_or_else(|| "".to_string(), |d| d.to_rfc3339())
    }

    async fn online(&self) -> u8 {
        self.online.clone()
    }

    async fn expansion(&self) -> u8 {
        self.expansion.clone()
    }

    async fn os(&self) -> String {
        self.os.clone()
    }
    async fn recruiter(&self) -> u32 {
        self.recruiter.clone()
    }
    async fn totaltime(&self) -> u32 {
        self.totaltime.clone()
    }
    async fn access(&self, ctx: &Context<'_>) -> Result<Vec<Access>, String> {
        let db = ctx.data_unchecked::<Storage>().lock().await;
        let fields = ctx
            .field()
            .selection_set()
            .map(|field| field.name())
            .collect::<Vec<_>>();
        db.access_by_user_id(self.id, &fields).await
    }
    async fn realmcharacters(&self, ctx: &Context<'_>) -> Result<Vec<RealmCharacter>, String> {
        let db = ctx.data_unchecked::<Storage>().lock().await;
        db.realmcharacters_by_user_id(self.id).await
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn accounts(&self, ctx: &Context<'_>) -> Vec<Account> {
        let db = ctx.data_unchecked::<Storage>().lock().await;
        let fields = ctx
            .field()
            .selection_set()
            .map(|field| field.name())
            .collect::<Vec<_>>();
        debug!("ctx.field = {:?}", &fields);
        debug!(
            "Account::FIELD_NAMES_AS_ARRAY = {:?}",
            Account::FIELD_NAMES_AS_ARRAY
        );

        let accounts = db.get_accounts_with_fields(&fields).await;
        accounts
    }
    async fn account(&self, ctx: &Context<'_>, id: u64) -> Result<Account, String> {
        let db = ctx.data_unchecked::<Storage>().lock().await;
        let fields = ctx
            .field()
            .selection_set()
            .map(|field| field.name())
            .collect::<Vec<_>>();

        let accounts = db.get_account_by_id(id, &fields).await;
        accounts
    }
    async fn check_username(&self, ctx: &Context<'_>, username: String) -> Result<bool, String> {
        let db = ctx.data_unchecked::<Storage>().lock().await;

        db.has_account_username(&username).await
    }

    async fn account_set_password(
        &self,
        ctx: &Context<'_>,
        username: String,
        password: String,
    ) -> Result<bool, String> {
        let db = ctx.data_unchecked::<Storage>().lock().await;

        db.set_username_password(username, password).await
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_account(
        &self,
        ctx: &Context<'_>,
        username: String,
        password: String,
        email: String,
    ) -> Result<u64, String> {
        let auth_db = ctx.data_unchecked::<Storage>().lock().await;
        auth_db.create_account(&username, &password, &email).await
    }

    async fn delete_account(&self, ctx: &Context<'_>, id: u64) -> Result<bool> {
        let auth_db = ctx.data_unchecked::<Storage>().lock().await;
        Ok(auth_db.delete_account(id).await)
    }
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn values(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = i32>> {
        println!("{:?}", ctx.data::<Account>());
        // if ctx.data::<Account>()?.0 != "123456" {
        //     return Err("Forbidden".into());
        // }
        Ok(futures_util::stream::once(async move { 10 }))
    }
}
pub type AccountSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
