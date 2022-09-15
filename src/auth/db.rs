use super::account::access::Access;
use super::account::realmcharacters::RealmCharacter;
use super::account::Account;
use futures::executor::block_on;
use futures::lock::Mutex;
use log::debug;
use log::error;
use sqlx::MySqlPool;
use sqlx::Row;
use std::env;
use std::sync::Arc;
use wow_srp::normalized_string::NormalizedString;
use wow_srp::server::SrpVerifier;

pub type Storage = Arc<Mutex<AuthDB>>;
pub async fn get_storage() -> Storage {
    let conn = AuthDB::new().await;
    Storage::new(Mutex::new(conn))
}
#[derive(Debug)]
pub struct AuthDB {
    pool: MySqlPool,
}
impl Default for AuthDB {
    fn default() -> Self {
        Self::new_sync()
    }
}
#[allow(dead_code)]
impl AuthDB {
    pub fn new_sync() -> Self {
        block_on(Self::new())
    }
    pub async fn new() -> Self {
        let pool = MySqlPool::connect(&env::var("AUTH_DB").unwrap())
            .await
            .unwrap();
        Self { pool }
    }

    pub async fn get_accounts(&self) -> Vec<Account> {
        match sqlx::query_as::<_, Account>("SELECT * FROM account")
            .fetch_all(&self.pool)
            .await
        {
            Ok(accs) => accs,
            Err(e) => {
                error!("{:?}", e);
                Vec::new()
            }
        }
    }
    pub async fn get_accounts_with_fields(&self, fields: &Vec<&str>) -> Vec<Account> {
        let columns = get_columns(&Account::FIELD_NAMES_AS_ARRAY.to_vec(), fields);
        let sql = format!("SELECT {} FROM account", &columns);
        match sqlx::query_as::<_, Account>(sql.as_str())
            .fetch_all(&self.pool)
            .await
        {
            Ok(accs) => accs,
            Err(e) => {
                error!("{:?}", e);
                Vec::new()
            }
        }
    }
    pub async fn get_account_by_id(&self, id: u64, fields: &Vec<&str>) -> Result<Account, String> {
        let columns = get_columns(&Account::FIELD_NAMES_AS_ARRAY.to_vec(), fields);
        let sql = format!("SELECT {} FROM account where id = ?", &columns);
        match sqlx::query_as::<_, Account>(sql.as_str())
            .bind(id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(acc) => Ok(acc),
            Err(e) => {
                error!("{:?}", e);
                Err("Account not found".to_string())
            }
        }
    }
    pub async fn has_account_id(&self, id: u64) -> bool {
        sqlx::query("SELECT id from account where id = ?")
            .bind(id)
            .map(|row| {
                let id: u64 = row.get(0);
                id
            })
            .fetch_one(&self.pool)
            .await
            .unwrap_or_default()
            > 0
    }
    pub async fn has_account_username(&self, username: &str) -> Result<bool, String> {
        match sqlx::query("SELECT id from account where username = ?")
            .bind(username)
            .map(|row| {
                let id: u64 = row.get(0);
                id
            })
            .fetch_one(&self.pool)
            .await
        {
            Ok(id) => Ok(id > 0),
            Err(e) => {
                error!("{:?}", e);
                Err("An error when checking an account".to_string())
            }
        }
    }

    pub async fn create_account(
        &self,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<u64, String> {
        let username = NormalizedString::new(username).unwrap();
        let email = NormalizedString::new(email)
            .unwrap()
            .to_string()
            .to_lowercase();
        let (salt, verifier, username) = {
            let password = NormalizedString::new(password).unwrap();

            let verifier = SrpVerifier::from_username_and_password(username, password);
            // Salt is randomly chosen and password_verifier depends on salt so we can't assert_eq
            // Store these values in the database for future authentication
            // let password_verifier = verifier.password_verifier();
            let salt = verifier.salt();
            (
                salt.to_vec(),
                verifier.password_verifier().to_vec(),
                verifier.username().to_string(),
            )
        };
        let sql ="INSERT INTO account(username, email, salt, verifier, expansion, joindate) VALUES(?, ?, ?, ?, ?, NOW())";

        match sqlx::query(sql)
            .bind(username)
            .bind(email)
            .bind(&salt)
            .bind(&verifier)
            .bind(2)
            .execute(&self.pool)
            .await
        {
            Ok(result) => {
                let new_account_id = result.last_insert_id();
                sqlx::query(
                    "INSERT INTO realmcharacters (realmid, acctid, numchars) SELECT realmlist.id, account.id, 0 FROM realmlist, account LEFT JOIN realmcharacters ON acctid=account.id WHERE acctid IS NULL"
                    ).execute(&self.pool).await.unwrap();
                Ok(new_account_id)
            }

            Err(e) => {
                error!("{:?}", e);
                match e {
                    sqlx::Error::Database(er) => {
                        if let Some(code) = er.code() {
                            if code.eq("23000") {
                                return Err("Account already exist".to_string());
                            }
                        }

                        return Err("Account cannot be created".to_string());
                    }
                    _ => Err("Account cannot be created".to_string()),
                }
            }
        }
    }
    pub async fn delete_account(&self, id: u64) -> bool {
        match sqlx::query("DELETE FROM account WHERE id = ? LIMIT 1")
            .bind(id)
            .execute(&self.pool)
            .await
        {
            Ok(r) => r.rows_affected() > 0,
            Err(e) => {
                error!("{:?}", e);
                false
            }
        }
    }
    pub async fn access_by_user_id(
        &self,
        user_id: u64,
        fields: &Vec<&str>,
    ) -> Result<Vec<Access>, String> {
        debug!("incomming filds {:?}", &fields);
        debug!(
            "Access::FIELD_NAMES_AS_ARRAY {:?}",
            &Access::FIELD_NAMES_AS_ARRAY
        );
        let columns = get_columns(&Access::FIELD_NAMES_AS_ARRAY.to_vec(), fields);
        let sql = format!("SELECT {} FROM account_access where id = ?", &columns);
        match sqlx::query_as::<_, Access>(sql.as_str())
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
        {
            Ok(res) => {
                debug!("Accesses found {:?}", &res);
                Ok(res)
            }
            Err(e) => {
                error!("{:?}", e);
                Err(format!("Access not found for account id {}", user_id))
            }
        }
    }
    pub async fn realmcharacters_by_user_id(
        &self,
        user_id: u64,
    ) -> Result<Vec<RealmCharacter>, String> {
        let sql = format!(
            r#"SELECT 
        rc.realmid as realmid, 
        rc.acctid as acctid,
        rc.numchars as numchars, 
        rl.name as realmname 
        FROM realmcharacters rc  
        JOIN realmlist rl  ON rl.id = rc.realmid 
        WHERE rc.acctid = ?"#
        );
        match sqlx::query_as::<_, RealmCharacter>(sql.as_str())
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
        {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("{:?}", e);
                Err(format!("No characters found for the user #{}", user_id))
            }
        }
    }
}

fn get_columns(struct_fields: &Vec<&str>, fields: &Vec<&str>) -> String {
    if fields.len() > 0 {
        use std::collections::HashSet;
        let known_fields = HashSet::<&&str>::from_iter(struct_fields);
        let search_fields = HashSet::<&&str>::from_iter(fields);
        let intersection: Vec<&str> = known_fields
            .intersection(&search_fields)
            .map(|f| **f)
            .collect();
        if intersection.len() > 0 {
            return intersection.join(",");
        }
    }
    "*".to_string()
}
