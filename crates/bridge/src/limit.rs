//! In-memory token-bucket rate limiting.
//!
//! Behind Cloudflare and nginx the peer address is always the proxy, so keys
//! are identities the bridge issued itself rather than IPs.

use std::{
   collections::HashMap,
   sync::Mutex,
   time::{
      Duration,
      Instant,
   },
};

/// Bounded, or the map is itself the unbounded allocation.
const MAX_TRACKED_KEYS: usize = 1 << 16;

struct Bucket {
   tokens:  f64,
   updated: Instant,
}

pub struct RateLimiter {
   capacity:   f64,
   per_second: f64,
   buckets:    Mutex<HashMap<String, Bucket>>,
}

impl RateLimiter {
   /// `burst` requests may arrive at once, refilling to full over `per`.
   pub fn new(burst: u32, per: Duration) -> Self {
      let capacity = f64::from(burst.max(1));
      Self {
         capacity,
         per_second: capacity / per.as_secs_f64().max(1.0),
         buckets: Mutex::new(HashMap::new()),
      }
   }

   /// Consumes one token, returning false when the key is over its budget.
   pub fn acquire(&self, key: &str) -> bool {
      let now = Instant::now();
      let mut buckets = self.buckets.lock().expect("rate limiter lock poisoned");
      if buckets.len() >= MAX_TRACKED_KEYS && !buckets.contains_key(key) {
         self.evict(&mut buckets, now);
      }
      let bucket = buckets.entry(key.to_owned()).or_insert(Bucket {
         tokens:  self.capacity,
         updated: now,
      });
      let available = refill(bucket, now, self.per_second, self.capacity);
      let granted = available >= 1.0;
      if granted {
         bucket.tokens = available - 1.0;
         bucket.updated = now;
      }
      drop(buckets);
      granted
   }

   /// Dropping recovered buckets alone frees nothing under a flood of distinct
   /// keys, so the oldest go too and the bound stays hard. That forgives their
   /// debt, but it needs [`MAX_TRACKED_KEYS`] bridge-issued identities to
   /// reach. Freed in one batch because each pass sorts the whole map.
   fn evict(&self, buckets: &mut HashMap<String, Bucket>, now: Instant) {
      buckets
         .retain(|_, bucket| refill(bucket, now, self.per_second, self.capacity) < self.capacity);
      let target = MAX_TRACKED_KEYS - MAX_TRACKED_KEYS / 8;
      if buckets.len() < target {
         return;
      }
      let excess = buckets.len() - target;
      let mut by_age = buckets
         .iter()
         .map(|(key, bucket)| (bucket.updated, key.clone()))
         .collect::<Vec<_>>();
      by_age.sort_unstable();
      for (_, key) in by_age.into_iter().take(excess) {
         buckets.remove(&key);
      }
   }
}

fn refill(bucket: &mut Bucket, now: Instant, per_second: f64, capacity: f64) -> f64 {
   let elapsed = now.saturating_duration_since(bucket.updated).as_secs_f64();
   let refilled = elapsed.mul_add(per_second, bucket.tokens).min(capacity);
   bucket.tokens = refilled;
   bucket.updated = now;
   refilled
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn burst_is_spent_then_refused() {
      let limiter = RateLimiter::new(3, Duration::from_secs(3600));
      assert!(limiter.acquire("a"));
      assert!(limiter.acquire("a"));
      assert!(limiter.acquire("a"));
      assert!(!limiter.acquire("a"));
      // Budgets are per key, so one noisy install cannot starve another.
      assert!(limiter.acquire("b"));
   }

   /// Keys seen exactly once leave every bucket partly spent, so dropping only
   /// recovered ones frees nothing.
   #[test]
   fn a_flood_of_distinct_keys_stays_bounded() {
      let limiter = RateLimiter::new(4, Duration::from_secs(3600));
      for index in 0..MAX_TRACKED_KEYS + 4096 {
         assert!(limiter.acquire(&format!("key-{index}")));
         assert!(limiter.buckets.lock().unwrap().len() <= MAX_TRACKED_KEYS);
      }
   }
}
