use anyhow::Context;
use anyhow::Result;
use opentelemetry::{
    global,
    trace::{FutureExt, TraceContextExt, Tracer},
};
use serde::Deserialize;
use serde::Serialize;
use shared::Analytics;
use std::net::IpAddr;
use tokio_rusqlite::Connection;
use tracing::info;

/// Allows geo-locating IPs and keeps analytics
#[derive(Debug)]
pub struct Locat {
    geoip: maxminddb::Reader<Vec<u8>>,
    analytics: Db,
}

#[derive(Deserialize, Serialize)]
struct AnalyticsKey {
    ip: String,
    path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("maxminddb error: {0}")]
    MaxMindDb(#[from] maxminddb::MaxMindDBError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("rusqlite error: {0}")]
    Rusqlite(#[from] tokio_rusqlite::Error),

    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl Locat {
    pub async fn new(
        geoip_country_db_path: &str,
        analytics_db_path: String,
    ) -> Result<Self, Error> {
        info!(
            "Reading Geoip Country DB at path: {}",
            geoip_country_db_path
        );
        let geoip_data = tokio::fs::read(geoip_country_db_path).await?;
        info!("Successfully read Geoip Country DB.");
        Ok(Self {
            geoip: maxminddb::Reader::from_source(geoip_data)?,
            analytics: Db::create(analytics_db_path).await?,
        })
    }

    pub async fn get_iso_code(&self, addr: IpAddr) -> anyhow::Result<&str> {
        let tracer = global::tracer("");
        tracer.in_span("maxminddb::geoip2::Country::lookup", |_| {
            self.geoip
                .lookup::<maxminddb::geoip2::Country>(addr)?
                .country
                .context("MaxMindDB missing country for ip")?
                .iso_code
                .context("MaxMindDB missing ISO code")
        })
    }

    pub async fn record_request(
        &self,
        ip: IpAddr,
        iso_code: String,
        path: String,
    ) -> Result<(), Error> {
        let tracer = global::tracer("");
        let ip = ip.to_string();
        self.analytics
            .increment(serde_json::to_string(&AnalyticsKey { ip, path })?, iso_code)
            .with_context(opentelemetry::Context::current_with_span(
                tracer.start("increment"),
            ))
            .await?;
        Ok(())
    }

    /// Returns a map of country codes to number of requests
    pub async fn get_analytics(&self) -> Result<Vec<Analytics>> {
        let tracer = global::tracer("");
        Ok(self
            .analytics
            .list()
            .with_context(opentelemetry::Context::current_with_span(
                tracer.start("list"),
            ))
            .await?)
    }
}

#[derive(Debug)]
struct Db {
    connection: Connection,
}

impl Db {
    async fn create(path: String) -> Result<Self, tokio_rusqlite::Error> {
        info!("Reading Sqlite3 db at: {}", path);
        let connection = Connection::open(path).await?;
        connection
            .call(|connection| {
                // create analytics table
                connection.execute(
                    "CREATE TABLE IF NOT EXISTS analytics (
                key TEXT PRIMARY KEY,
                iso_code TEXT,
                count INTEGER NOT NULL
            )",
                    [],
                )?;

                Ok::<_, rusqlite::Error>(())
            })
            .await?;
        info!("Successfully created connection with Sqlite3 db.");
        Ok(Self { connection })
    }

    async fn list(&self) -> Result<Vec<Analytics>, tokio_rusqlite::Error> {
        self.connection
            .call(|connection| {
                let mut statement =
                    connection.prepare("SELECT key, iso_code, count FROM analytics")?;
                let mut rows = statement.query([])?;
                let mut analytics = Vec::new();
                while let Some(row) = rows.next()? {
                    let analytics_key: String = row.get(0)?;
                    let analytics_key: AnalyticsKey = serde_json::from_str(&analytics_key).unwrap();
                    let iso_code = row.get(1)?;
                    let count = row.get(2)?;
                    analytics.push(Analytics {
                        ip_address: analytics_key.ip,
                        path: analytics_key.path,
                        iso_code,
                        count,
                    });
                }
                Ok(analytics)
            })
            .await
    }

    async fn increment(&self, key: String, iso_code: String) -> Result<(), tokio_rusqlite::Error> {
        self.connection
            .call(move |connection| {
                let mut statement = connection.prepare(
                    "INSERT INTO analytics (key, iso_code, count)
                VALUES (?, ?, 1)
                ON CONFLICT (key)
                DO UPDATE SET count = count + 1",
                )?;
                statement.execute([key, iso_code])?;
                Ok(())
            })
            .await
    }
}
