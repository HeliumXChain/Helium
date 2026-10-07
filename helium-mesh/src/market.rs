//! S3 market: resource offers/requests + local credits ledger (SQLite).
//!
//! Single node, no blockchain: `faucet` mints demo credits, `offer` /
//! `request` record intents, `find_matches` ports the helium-core Matcher
//! score, `accept` settles the transfer in the ledger.
//!
//! Reachability: offers carry the provider `endpoint` + `wg_pubkey` so a
//! borrower (or the daemon) can configure the tunnel without side channels.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

const DB_FILE: &str = "market.db";

#[derive(Debug, Clone, serde::Serialize)]
pub struct Offer {
    pub id: String,
    pub provider: String,
    pub rtype: String,
    pub amount: u32,
    pub price_per_hour: f64,
    pub endpoint: String,
    pub wg_pubkey: String,
    /// Sustained bench score (GFLOPS, `helium bench`). 0 = unmeasured.
    #[serde(default)]
    pub bench_gflops: f64,
    pub status: String,
}

/// Groups the 7 offer fields so add_offer stays a 2-arg call.
#[derive(Debug, Clone)]
pub struct NewOffer {
    pub provider: String,
    pub rtype: String,
    pub amount: u32,
    pub price: f64,
    pub endpoint: String,
    pub wg_pubkey: String,
    pub bench_gflops: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Request {
    pub id: String,
    pub requester: String,
    pub rtype: String,
    pub amount: u32,
    pub max_price: f64,
    pub hours: u32,
    pub wg_pubkey: String,
    pub endpoint: String,
    /// S2: container image wanted for the job ("" = node default, see Q5 templates).
    pub image: String,
    /// S2: GPUs wanted (0 = unset, node decides).
    pub gpus: u32,
    /// Bench floor: offers below this sustained GFLOPS never match (0 = any).
    #[serde(default)]
    pub min_bench: f64,
    pub status: String,
}

/// S4: groups the 9 request fields so add_request stays a 2-arg call.
#[derive(Debug, Clone)]
pub struct NewRequest {
    pub requester: String,
    pub rtype: String,
    pub amount: u32,
    pub max_price: f64,
    pub hours: u32,
    pub wg_pubkey: String,
    pub endpoint: String,
    pub image: String,
    pub gpus: u32,
    pub min_bench: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MatchRow {
    pub id: String,
    pub offer_id: String,
    pub request_id: String,
    pub score: f64,
    pub est_cost: f64,
    pub status: String,
}

pub struct Market {
    conn: Connection,
}

fn helium_db_path() -> Result<std::path::PathBuf> {
    let dir = crate::identity::helium_dir()?;
    std::fs::create_dir_all(&dir).context("cannot create helium dir")?;
    Ok(dir.join(DB_FILE))
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS offers (
    id TEXT PRIMARY KEY, provider TEXT NOT NULL, rtype TEXT NOT NULL,
    amount INTEGER NOT NULL, price_per_hour REAL NOT NULL,
    endpoint TEXT NOT NULL DEFAULT '', wg_pubkey TEXT NOT NULL DEFAULT '',
    bench_gflops REAL NOT NULL DEFAULT 0.0,
    status TEXT NOT NULL DEFAULT 'open', created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS requests (
    id TEXT PRIMARY KEY, requester TEXT NOT NULL, rtype TEXT NOT NULL,
    amount INTEGER NOT NULL, max_price REAL NOT NULL, hours INTEGER NOT NULL,
    wg_pubkey TEXT NOT NULL DEFAULT '', endpoint TEXT NOT NULL DEFAULT '',
    image TEXT NOT NULL DEFAULT '', gpus INTEGER NOT NULL DEFAULT 0,
    min_bench REAL NOT NULL DEFAULT 0.0,
    status TEXT NOT NULL DEFAULT 'open', created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS matches (
    id TEXT PRIMARY KEY, offer_id TEXT NOT NULL, request_id TEXT NOT NULL,
    score REAL NOT NULL, est_cost REAL NOT NULL,
    status TEXT NOT NULL DEFAULT 'proposed', created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS ledger (
    id TEXT PRIMARY KEY, from_party TEXT NOT NULL, to_party TEXT NOT NULL,
    amount REAL NOT NULL, memo TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS allocs (
    ip TEXT PRIMARY KEY, peer_pubkey TEXT NOT NULL, match_id TEXT NOT NULL,
    created_at TEXT NOT NULL
);
";

/// Bring pre-S3 databases up to date. Idempotent: duplicate columns ignored.
fn migrate(conn: &Connection) -> Result<()> {
    for stmt in [
        "ALTER TABLE offers ADD COLUMN endpoint TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE offers ADD COLUMN wg_pubkey TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE requests ADD COLUMN wg_pubkey TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE requests ADD COLUMN endpoint TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE requests ADD COLUMN image TEXT NOT NULL DEFAULT ''",
        "ALTER TABLE requests ADD COLUMN gpus INTEGER NOT NULL DEFAULT 0",
        "ALTER TABLE requests ADD COLUMN min_bench REAL NOT NULL DEFAULT 0.0",
        "ALTER TABLE offers ADD COLUMN bench_gflops REAL NOT NULL DEFAULT 0.0",
    ] {
        match conn.execute_batch(stmt) {
            Ok(()) => {}
            Err(e) => {
                if !e.to_string().contains("duplicate column name") {
                    return Err(e).context("market migration failed");
                }
            }
        }
    }
    Ok(())
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Match score ported from helium-core Matcher (0.0-1.0, None = incompatible).
fn match_score(
    req_type: &str,
    req_amount: u32,
    req_max_price: f64,
    req_min_bench: f64,
    off_type: &str,
    off_amount: u32,
    off_price: f64,
    off_bench: f64,
) -> Option<f64> {
    if req_type != off_type {
        return None;
    }
    if off_bench < req_min_bench {
        return None;
    }
    if off_price > req_max_price {
        return None;
    }
    if off_amount < req_amount {
        return None;
    }
    if req_max_price <= 0.0 || req_amount == 0 {
        return None;
    }
    let price_score = 1.0 - (off_price / req_max_price);
    let capacity_score = (off_amount as f64 / req_amount as f64).min(2.0) / 2.0;
    Some(((price_score * 0.5) + (capacity_score * 0.5)).clamp(0.0, 1.0))
}

impl Market {
    pub fn open() -> Result<Self> {
        let path = helium_db_path()?;
        let conn = Connection::open(path).context("cannot open market.db")?;
        conn.execute_batch(SCHEMA).context("cannot init schema")?;
        migrate(&conn)?;
        Ok(Self { conn })
    }

    /// In-memory database: demos, tests, zero disk writes.
    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("cannot open memory db")?;
        conn.execute_batch(SCHEMA).context("cannot init schema")?;
        migrate(&conn)?;
        Ok(Self { conn })
    }

    pub fn add_offer(&self, off: &NewOffer) -> Result<String> {
        let id = format!("offer-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        self.conn.execute(
            "INSERT INTO offers (id, provider, rtype, amount, price_per_hour, endpoint, wg_pubkey, bench_gflops, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'open', ?9)",
            params![id, off.provider, off.rtype, off.amount as i64, off.price, off.endpoint, off.wg_pubkey, off.bench_gflops, now()],
        )?;
        Ok(id)
    }

    pub fn list_offers(&self) -> Result<Vec<Offer>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, provider, rtype, amount, price_per_hour, endpoint, wg_pubkey, bench_gflops, status FROM offers
             WHERE status = 'open' ORDER BY price_per_hour ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Offer {
                id: r.get(0)?,
                provider: r.get(1)?,
                rtype: r.get(2)?,
                amount: r.get::<_, i64>(3)? as u32,
                price_per_hour: r.get(4)?,
                endpoint: r.get(5)?,
                wg_pubkey: r.get(6)?,
                bench_gflops: r.get(7)?,
                status: r.get(8)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn get_offer(&self, id: &str) -> Result<Offer> {
        self.conn.query_row(
            "SELECT id, provider, rtype, amount, price_per_hour, endpoint, wg_pubkey, bench_gflops, status FROM offers WHERE id = ?1",
            params![id],
            |r| {
                Ok(Offer {
                    id: r.get(0)?,
                    provider: r.get(1)?,
                    rtype: r.get(2)?,
                    amount: r.get::<_, i64>(3)? as u32,
                    price_per_hour: r.get(4)?,
                    endpoint: r.get(5)?,
                    wg_pubkey: r.get(6)?,
                    bench_gflops: r.get(7)?,
                    status: r.get(8)?,
                })
            },
        ).with_context(|| format!("offer {id} not found"))
    }

    pub fn add_request(&self, req: &NewRequest) -> Result<String> {
        let id = format!("req-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        self.conn.execute(
            "INSERT INTO requests (id, requester, rtype, amount, max_price, hours, wg_pubkey, endpoint, image, gpus, min_bench, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'open', ?12)",
            params![id, req.requester, req.rtype, req.amount as i64, req.max_price, req.hours as i64, req.wg_pubkey, req.endpoint, req.image, req.gpus as i64, req.min_bench, now()],
        )?;
        Ok(id)
    }

    pub fn list_requests(&self) -> Result<Vec<Request>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, requester, rtype, amount, max_price, hours, wg_pubkey, endpoint, image, gpus, min_bench, status FROM requests
             WHERE status = 'open' ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Request {
                id: r.get(0)?,
                requester: r.get(1)?,
                rtype: r.get(2)?,
                amount: r.get::<_, i64>(3)? as u32,
                max_price: r.get(4)?,
                hours: r.get::<_, i64>(5)? as u32,
                wg_pubkey: r.get(6)?,
                endpoint: r.get(7)?,
                image: r.get(8)?,
                gpus: r.get::<_, i64>(9)? as u32,
                min_bench: r.get(10)?,
                status: r.get(11)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn get_request(&self, id: &str) -> Result<Request> {
        self.conn.query_row(
            "SELECT id, requester, rtype, amount, max_price, hours, wg_pubkey, endpoint, image, gpus, min_bench, status FROM requests WHERE id = ?1",
            params![id],
            |r| {
                Ok(Request {
                    id: r.get(0)?,
                    requester: r.get(1)?,
                    rtype: r.get(2)?,
                    amount: r.get::<_, i64>(3)? as u32,
                    max_price: r.get(4)?,
                    hours: r.get::<_, i64>(5)? as u32,
                    wg_pubkey: r.get(6)?,
                    endpoint: r.get(7)?,
                    image: r.get(8)?,
                    gpus: r.get::<_, i64>(9)? as u32,
                    min_bench: r.get(10)?,
                    status: r.get(11)?,
                })
            },
        ).with_context(|| format!("request {id} not found"))
    }

    pub fn find_matches(&self, request_id: &str) -> Result<Vec<MatchRow>> {
        let req = self.get_request(request_id)?;
        if req.status != "open" {
            anyhow::bail!("request {request_id} is not open (status: {})", req.status);
        }
        let offers = self.list_offers()?;
        let mut out = Vec::new();
        for o in offers {
            if let Some(score) = match_score(
                &req.rtype,
                req.amount,
                req.max_price,
                req.min_bench,
                &o.rtype,
                o.amount,
                o.price_per_hour,
                o.bench_gflops,
            ) {
                let est_cost = score * o.price_per_hour * req.hours as f64;
                let id = format!("match-{}", &uuid::Uuid::new_v4().to_string()[..8]);
                self.conn.execute(
                    "INSERT INTO matches (id, offer_id, request_id, score, est_cost, status, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, 'proposed', ?6)",
                    params![id, o.id, request_id, score, est_cost, now()],
                )?;
                out.push(MatchRow {
                    id,
                    offer_id: o.id,
                    request_id: request_id.to_string(),
                    score,
                    est_cost,
                    status: "proposed".to_string(),
                });
            }
        }
        out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(out)
    }

    pub fn get_match(&self, id: &str) -> Result<MatchRow> {
        self.conn.query_row(
            "SELECT id, offer_id, request_id, score, est_cost, status FROM matches WHERE id = ?1",
            params![id],
            |r| {
                Ok(MatchRow {
                    id: r.get(0)?,
                    offer_id: r.get(1)?,
                    request_id: r.get(2)?,
                    score: r.get(3)?,
                    est_cost: r.get(4)?,
                    status: r.get(5)?,
                })
            },
        ).with_context(|| format!("match {id} not found"))
    }

    /// Accept a proposed match: closes offer+request, settles credits.
    pub fn accept(&self, match_id: &str) -> Result<(String, String, f64)> {
        let (offer_id, request_id, est_cost, status): (String, String, f64, String) =
            self.conn.query_row(
                "SELECT offer_id, request_id, est_cost, status FROM matches WHERE id = ?1",
                params![match_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )?;
        if status != "proposed" {
            anyhow::bail!("match {match_id} is not proposed (status: {status})");
        }
        let offer = self.get_offer(&offer_id)?;
        let request = self.get_request(&request_id)?;
        if offer.status != "open" || request.status != "open" {
            anyhow::bail!("offer/request no longer open");
        }
        let bal = self.balance(&request.requester)?;
        if bal < est_cost {
            anyhow::bail!(
                "insufficient credits for {}: {bal:.2} < {est_cost:.2} (run 'helium faucet' for demo credits)",
                request.requester
            );
        }
        let tx = format!("tx-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        self.conn.execute(
            "INSERT INTO ledger (id, from_party, to_party, amount, memo, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![tx, request.requester, offer.provider, est_cost, format!("match:{match_id}"), now()],
        )?;
        self.conn.execute("UPDATE matches SET status = 'accepted' WHERE id = ?1", params![match_id])?;
        self.conn.execute("UPDATE offers SET status = 'matched' WHERE id = ?1", params![offer_id])?;
        self.conn.execute("UPDATE requests SET status = 'filled' WHERE id = ?1", params![request_id])?;
        Ok((request.requester, offer.provider, est_cost))
    }

    /// S3: withdraw an open request. No credits moved (only accept settles).
    pub fn cancel_request(&self, id: &str) -> Result<String> {
        let req = self.get_request(id)?;
        if req.status != "open" {
            anyhow::bail!("request {id} is not open (status: {})", req.status);
        }
        self.conn.execute(
            "UPDATE requests SET status = 'cancelled' WHERE id = ?1",
            params![id],
        )?;
        Ok("cancelled".to_string())
    }

    /// S3: expire open requests older than `ttl_hours`. Returns evicted count.
    pub fn expire_stale(&self, ttl_hours: u32) -> Result<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(ttl_hours as i64);
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at FROM requests WHERE status = 'open'",
        )?;
        let rows: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut evicted = 0;
        for (id, created_at) in rows {
            let stale = chrono::DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc) < cutoff)
                .unwrap_or(false);
            if stale {
                self.conn.execute(
                    "UPDATE requests SET status = 'expired' WHERE id = ?1",
                    params![id],
                )?;
                evicted += 1;
            }
        }
        Ok(evicted)
    }

    /// Allocate the next free 10.0.0.x tunnel IP for a peer (provider side).
    pub fn alloc_ip(&self, peer_pubkey: &str, match_id: &str) -> Result<String> {
        let mut stmt = self.conn.prepare("SELECT ip FROM allocs")?;
        let used: Vec<String> = stmt
            .query_map([], |r| r.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        for last in 3..=254u8 {
            let ip = format!("10.0.0.{last}");
            if !used.contains(&ip) {
                self.conn.execute(
                    "INSERT INTO allocs (ip, peer_pubkey, match_id, created_at) VALUES (?1, ?2, ?3, ?4)",
                    params![ip, peer_pubkey, match_id, now()],
                )?;
                return Ok(ip);
            }
        }
        anyhow::bail!("tunnel subnet exhausted")
    }

    pub fn balance(&self, party: &str) -> Result<f64> {
        let incoming: Option<f64> = self
            .conn
            .query_row(
                "SELECT SUM(amount) FROM ledger WHERE to_party = ?1",
                params![party],
                |r| r.get(0),
            )
            .unwrap_or(None);
        let outgoing: Option<f64> = self
            .conn
            .query_row(
                "SELECT SUM(amount) FROM ledger WHERE from_party = ?1",
                params![party],
                |r| r.get(0),
            )
            .unwrap_or(None);
        Ok(incoming.unwrap_or(0.0) - outgoing.unwrap_or(0.0))
    }

    /// Mint demo credits (no real money in MVP).
    pub fn faucet(&self, party: &str, amount: f64) -> Result<f64> {
        let id = format!("faucet-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        self.conn.execute(
            "INSERT INTO ledger (id, from_party, to_party, amount, memo, created_at)
             VALUES (?1, '__faucet__', ?2, ?3, 'demo credits', ?4)",
            params![id, party, amount, now()],
        )?;
        self.balance(party)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_rejects_mismatch() {
        assert!(match_score("gpu", 16, 1.0, 0.0, "ram", 32, 0.5, 0.0).is_none());
        assert!(match_score("gpu", 16, 1.0, 0.0, "gpu", 32, 2.0, 0.0).is_none());
        assert!(match_score("gpu", 64, 1.0, 0.0, "gpu", 32, 0.5, 0.0).is_none());
        let s = match_score("gpu", 16, 1.0, 0.0, "gpu", 32, 0.5, 0.0).unwrap();
        assert!((0.0..=1.0).contains(&s));
    }

    #[test]
    fn score_enforces_bench_floor() {
        // Unbenched offer (0.0) never matches a request demanding a floor.
        assert!(match_score("gpu", 16, 1.0, 10.0, "gpu", 32, 0.5, 0.0).is_none());
        assert!(match_score("gpu", 16, 1.0, 10.0, "gpu", 32, 0.5, 9.9).is_none());
        assert!(match_score("gpu", 16, 1.0, 10.0, "gpu", 32, 0.5, 10.0).is_some());
    }

    fn new_offer(provider: &str, endpoint: &str, wg_pubkey: &str) -> NewOffer {
        NewOffer {
            provider: provider.to_string(),
            rtype: "gpu".to_string(),
            amount: 32,
            price: 0.5,
            endpoint: endpoint.to_string(),
            wg_pubkey: wg_pubkey.to_string(),
            bench_gflops: 0.0,
        }
    }

    #[test]
    fn migrate_is_idempotent() -> Result<()> {
        let m = Market::open_memory()?;
        migrate(&m.conn)?;
        migrate(&m.conn)?;
        m.add_offer(&new_offer("bob", "1.2.3.4:51820", "PUB"))?;
        Ok(())
    }

    #[test]
    fn full_flow_in_memory() -> Result<()> {
        let m = Market::open_memory()?;
        m.faucet("alice", 100.0)?;
        let oid = m.add_offer(&new_offer("bob", "1.2.3.4:51820", "PUBBOB"))?;
        let rid = m.add_request(&NewRequest {
            requester: "alice".to_string(),
            rtype: "gpu".to_string(),
            amount: 16,
            max_price: 1.0,
            hours: 2,
            wg_pubkey: "PUBALICE".to_string(),
            endpoint: String::new(),
            image: "ubuntu:22.04".to_string(),
            gpus: 1,
            min_bench: 0.0,
        })?;
        let req = m.get_request(&rid)?;
        assert_eq!(req.image, "ubuntu:22.04");
        assert_eq!(req.gpus, 1);
        let offer = m.get_offer(&oid)?;
        assert_eq!(offer.endpoint, "1.2.3.4:51820");
        let matches = m.find_matches(&rid)?;
        assert_eq!(matches.len(), 1);
        let (from, to, cost) = m.accept(&matches[0].id)?;
        assert_eq!((from.as_str(), to.as_str()), ("alice", "bob"));
        assert!(cost > 0.0);
        assert!((m.balance("alice")? - (100.0 - cost)).abs() < 1e-6);
        assert!((m.balance("bob")? - cost).abs() < 1e-6);
        let ip1 = m.alloc_ip("PUBALICE", &matches[0].id)?;
        let ip2 = m.alloc_ip("PUBOTHER", "match-x")?;
        assert_ne!(ip1, ip2);
        assert!(ip1.starts_with("10.0.0."));
        assert!(m.find_matches(&rid).is_err()); // request filled
        Ok(())
    }

    #[test]
    fn cancel_and_expiry() -> Result<()> {
        let m = Market::open_memory()?;
        let new_req = |requester: &str| NewRequest {
            requester: requester.to_string(),
            rtype: "gpu".to_string(),
            amount: 16,
            max_price: 1.0,
            hours: 2,
            wg_pubkey: String::new(),
            endpoint: String::new(),
            image: String::new(),
            gpus: 0,
            min_bench: 0.0,
        };
        let rid = m.add_request(&new_req("alice"))?;
        assert_eq!(m.cancel_request(&rid)?, "cancelled");
        assert_eq!(m.get_request(&rid)?.status, "cancelled");
        assert!(m.cancel_request(&rid).is_err()); // no longer open
        assert!(m.find_matches(&rid).is_err()); // cannot match it anymore
        assert!(m.cancel_request("req-missing").is_err()); // unknown id
        let rid2 = m.add_request(&new_req("bob"))?;
        m.conn.execute(
            "UPDATE requests SET created_at = '2000-01-01T00:00:00+00:00' WHERE id = ?1",
            params![rid2],
        )?;
        assert_eq!(m.expire_stale(24)?, 1);
        assert_eq!(m.get_request(&rid2)?.status, "expired");
        assert_eq!(m.expire_stale(24)?, 0); // idempotent
        Ok(())
    }
}
