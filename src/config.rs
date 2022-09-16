#[derive(Debug, Clone)]
pub struct Config {
    host: String,
    port: u16,
    is_secure: bool,
}

impl Config {
    const DEFAULT_HOST: &'static str = "127.0.0.1";
    const DEFAUTL_PORT: u16 = 8000;
    pub fn from_env() -> Self {
        let host =
            std::env::var("GQL_SERVER_HOST").unwrap_or_else(|_| Self::DEFAULT_HOST.to_string());
        let port = std::env::var("GQL_SERVER_PORT").map_or_else(
            |_| Self::DEFAUTL_PORT,
            |v| v.parse::<u16>().unwrap_or(Self::DEFAUTL_PORT),
        );
        let is_secure = std::env::var("GQL_SERVER_IS_SECURE")
            .map_or_else(|_| false, |v| v.parse::<bool>().unwrap_or(false));
        Self {
            host,
            port,
            is_secure,
        }
    }
    pub fn host(&self) -> String {
        self.host.clone()
    }
    pub fn port(&self) -> u16 {
        self.port.clone()
    }
    pub fn is_secure(&self) -> bool {
        self.is_secure.clone()
    }
}
