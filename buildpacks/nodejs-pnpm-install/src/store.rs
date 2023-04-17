const CACHE_PRUNE_INTERVAL: i64 = 40;
const CACHE_USE_KEY: &str = "cache_use_count";

/// Reads and returns the cache use count as i64. This expects the value to
/// stored as a float, since CNB erroneously converts toml integers to toml floats.
pub(crate) fn read_cache_use_count(metadata: &toml::Table) -> i64 {
    #[allow(clippy::cast_possible_truncation)]
    metadata.get(CACHE_USE_KEY).map_or(0, |v| {
        v.as_float().map_or(CACHE_PRUNE_INTERVAL, |f| f as i64)
    })
}

/// Sets the cache use count in toml metadata as a float. It's stored as a
/// float since CNB erroneously converts toml integers to toml floats anyway.
pub(crate) fn set_cache_use_count(metadata: &mut toml::Table, cache_use_count: i64) {
    #[allow(clippy::cast_precision_loss)]
    metadata.insert(
        CACHE_USE_KEY.to_owned(),
        toml::Value::from(cache_use_count as f64),
    );
}

/// Given a cache use count returns whether the cache should be pruned
pub(crate) fn should_prune_cache(cache_use_count: i64) -> bool {
    cache_use_count.rem_euclid(CACHE_PRUNE_INTERVAL) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_cache_use_count_valid() {
        let mut md = toml::Table::default();
        md.insert(CACHE_USE_KEY.to_owned(), toml::Value::from(42f64));
        assert_eq!(read_cache_use_count(&md), 42i64);
    }

    #[test]
    fn read_cache_use_count_empty() {
        let md = toml::Table::default();
        assert_eq!(read_cache_use_count(&md), 0i64);
    }

    #[test]
    fn read_cache_use_count_invalid() {
        let mut md = toml::Table::default();
        md.insert(CACHE_USE_KEY.to_owned(), toml::Value::from("12b"));
        assert_eq!(read_cache_use_count(&md), CACHE_PRUNE_INTERVAL);
    }

    #[test]
    fn test_set_cache_use_count() {
        let margin = std::f64::EPSILON;
        let mut md = toml::Table::default();
        set_cache_use_count(&mut md, 3);
        let expected = 3f64;
        let actual = md
            .get(CACHE_USE_KEY)
            .expect("expected key to be set")
            .as_float()
            .expect("expected key to be a float");
        assert!((actual - expected).abs() < margin);
    }
}
