//! Thin blocking HTTP layer shared by every doc source, with transparent
//! disk caching and `--offline` support baked in.

use crate::cache::Cache;
use crate::error::{Result, RsHelpError};
use std::time::Duration;

pub struct HttpCtx {
    client: reqwest::blocking::Client,
    cache: Cache,
    offline: bool,
    no_cache: bool,
}

/// Outcome of trying a single candidate URL: either it resolved (with the
/// body, whether it came from cache, and the final URL after redirects), or
/// it clearly doesn't exist (404) so the caller should try the next
/// candidate, or a hard error occurred (network/timeout/DNS) that should
/// stop the whole lookup.
pub enum FetchOutcome {
    Found { body: String, from_cache: bool },
    NotFound,
}

impl HttpCtx {
    pub fn new(timeout_secs: u64, cache_ttl_secs: u64, offline: bool, no_cache: bool) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(concat!("rshelp/", env!("CARGO_PKG_VERSION"), " (+https://github.com/cumulus13/rshelp)"))
            .redirect(reqwest::redirect::Policy::limited(8))
            .build()
            .map_err(RsHelpError::Network)?;

        Ok(HttpCtx {
            client,
            cache: Cache::new(cache_ttl_secs),
            offline,
            no_cache,
        })
    }

    /// Fetch `url`, transparently consulting/populating the cache. Returns
    /// `FetchOutcome::NotFound` for a clean 404 so resolvers can try the
    /// next candidate URL; any other non-success status or transport
    /// failure is a hard [`RsHelpError`].
    pub fn get(&self, url: &str) -> Result<FetchOutcome> {
        if !self.no_cache {
            if let Some(body) = self.cache.get(url, self.offline) {
                return Ok(FetchOutcome::Found { body, from_cache: true });
            }
        }

        if self.offline {
            return Err(RsHelpError::OfflineMiss(url.to_string()));
        }

        let resp = self
            .client
            .get(url)
            .send()
            .map_err(|source| RsHelpError::Fetch { url: url.to_string(), source })?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(FetchOutcome::NotFound);
        }
        if !resp.status().is_success() {
            let status = resp.status();
            return Err(RsHelpError::Parse(format!(
                "unexpected HTTP status {status} fetching {url}"
            )));
        }

        let body = resp
            .text()
            .map_err(|source| RsHelpError::Fetch { url: url.to_string(), source })?;

        if !self.no_cache {
            let _ = self.cache.put(url, &body);
        }

        Ok(FetchOutcome::Found { body, from_cache: false })
    }
}
