#![forbid(unsafe_code)]

use std::{collections::BTreeMap, net::SocketAddr};

#[derive(Clone, Debug)]
pub struct WebConfig {
    pub bind: String,
    pub api_http_base: Option<String>,
    pub database_url: Option<String>,
}

impl WebConfig {
    pub fn from_env() -> Self {
        Self::from_map(&std::env::vars().collect())
    }

    pub fn from_map(environment: &BTreeMap<String, String>) -> Self {
        Self {
            bind: environment
                .get("DECLMIG_WEB_BIND")
                .cloned()
                .unwrap_or_else(|| "127.0.0.1:8081".into()),
            api_http_base: environment.get("DECLMIG_API_HTTP_BASE").cloned(),
            database_url: environment.get("DECLMIG_DATABASE_URL").cloned(),
        }
    }
}

impl WebConfig {
    /// Reject malformed explicitly supplied endpoints before startup. Never echo
    /// connection strings because database URLs can contain credentials.
    pub fn validate(&self) -> Result<(), &'static str> {
        let bind: SocketAddr = self.bind.parse().map_err(|_| "DECLMIG_WEB_BIND")?;
        if bind.port() == 0 {
            return Err("DECLMIG_WEB_BIND");
        }
        if let Some(api) = &self.api_http_base {
            let rest = api
                .strip_prefix("https://")
                .or_else(|| api.strip_prefix("http://"))
                .ok_or("DECLMIG_API_HTTP_BASE must use http:// or https://")?;
            let authority = rest.split('/').next().unwrap_or("");
            if authority.is_empty()
                || authority.starts_with(':')
                || api.chars().any(char::is_whitespace)
                || api.contains(['@', '#', '?', '\\'])
            {
                return Err("DECLMIG_API_HTTP_BASE must be a valid base without credentials or query");
            }
        }
        if let Some(db) = &self.database_url {
            let rest = db
                .strip_prefix("postgres://")
                .or_else(|| db.strip_prefix("postgresql://"))
                .ok_or("DECLMIG_DATABASE_URL must use postgres:// or postgresql://")?;
            if rest.is_empty() || rest.chars().any(char::is_whitespace) || rest.contains('#') {
                return Err("DECLMIG_DATABASE_URL must contain an authority");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod endpoint_tests {
    use super::*;

    #[test]
    fn accepts_valid_web_and_database_endpoints() {
        let env = BTreeMap::from([
            ("DECLMIG_WEB_BIND".into(), "[::1]:8081".into()),
            ("DECLMIG_API_HTTP_BASE".into(), "https://api.example.test/v1".into()),
            ("DECLMIG_DATABASE_URL".into(), "postgresql://user:secret@db.local:5432/declmig".into()),
        ]);
        assert!(WebConfig::from_map(&env).validate().is_ok());
        assert!(WebConfig::from_map(&BTreeMap::new()).validate().is_ok());
    }

    #[test]
    fn rejects_malformed_explicit_configuration_without_leaking_values() {
        let cases = [
            ("DECLMIG_WEB_BIND", "localhost:8081"),
            ("DECLMIG_WEB_BIND", "127.0.0.1:0"),
            ("DECLMIG_API_HTTP_BASE", "ftp://api.example.test"),
            ("DECLMIG_API_HTTP_BASE", "https://"),
            ("DECLMIG_API_HTTP_BASE", "https://user:secret@api.example.test"),
            ("DECLMIG_API_HTTP_BASE", "http://api.example.test?token=secret"),
            ("DECLMIG_DATABASE_URL", "https://db.example.test"),
            ("DECLMIG_DATABASE_URL", "postgresql://"),
            ("DECLMIG_DATABASE_URL", "postgres://bad host"),
        ];
        for (name, value) in cases {
            let env = BTreeMap::from([(name.to_owned(), value.to_owned())]);
            let error = WebConfig::from_map(&env).validate().expect_err(name);
            assert!(!error.contains(value), "configuration values must not appear in errors");
        }
    }
}
