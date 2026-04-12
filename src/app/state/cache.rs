use crate::constants::timeouts::{CACHE_EXPIRATION_TIME, CACHE_INVALIDATION_OFFSET_SECS};

/// Cache for device creation options to avoid repeated API calls.
/// This cache stores available device types, API levels, and runtimes.
/// It expires after 5 minutes to ensure fresh data.
#[derive(Debug, Clone)]
pub struct DeviceCache {
    /// Available Android device types as (id, display_name) tuples
    pub android_device_types: Vec<(String, String)>,
    /// Available Android API levels as (api_level, display_name) tuples
    pub android_api_levels: Vec<(String, String)>,
    /// Complete list of Android devices for category filtering
    pub android_device_cache: Option<Vec<(String, String)>>,
    /// Available iOS device types as (identifier, display_name) tuples
    pub ios_device_types: Vec<(String, String)>,
    /// Available iOS runtime versions as (identifier, display_name) tuples
    pub ios_runtimes: Vec<(String, String)>,
    /// Timestamp of last cache update
    pub last_updated: std::time::Instant,
    /// Flag indicating if cache is currently being loaded
    pub is_loading: bool,
}

impl Default for DeviceCache {
    fn default() -> Self {
        Self {
            android_device_types: Vec::new(),
            android_api_levels: Vec::new(),
            android_device_cache: None,
            ios_device_types: Vec::new(),
            ios_runtimes: Vec::new(),
            last_updated: std::time::Instant::now(),
            is_loading: false,
        }
    }
}

impl DeviceCache {
    /// Checks if the cache is stale (older than 5 minutes).
    /// Returns true if the cache should be refreshed.
    pub fn is_stale(&self) -> bool {
        self.last_updated.elapsed() > CACHE_EXPIRATION_TIME
    }

    /// Updates the Android device cache with new data.
    /// Resets the loading flag and updates the timestamp.
    pub fn update_android_cache(
        &mut self,
        device_types: Vec<(String, String)>,
        api_levels: Vec<(String, String)>,
    ) {
        self.android_device_types = device_types;
        self.android_api_levels = api_levels;
        self.last_updated = std::time::Instant::now();
        self.is_loading = false;
    }

    /// Updates the iOS device cache with new data.
    /// Resets the loading flag and updates the timestamp.
    pub fn update_ios_cache(
        &mut self,
        device_types: Vec<(String, String)>,
        runtimes: Vec<(String, String)>,
    ) {
        self.ios_device_types = device_types;
        self.ios_runtimes = runtimes;
        self.last_updated = std::time::Instant::now();
        self.is_loading = false;
    }

    /// Invalidates the Android cache by clearing API levels and marking as stale.
    /// This forces a cache refresh on the next device creation.
    pub fn invalidate_android_cache(&mut self) {
        self.android_api_levels.clear();
        self.last_updated = std::time::Instant::now()
            - std::time::Duration::from_secs(CACHE_INVALIDATION_OFFSET_SECS);
    }

    /// Invalidates the iOS cache by clearing runtimes and marking as stale.
    /// This forces a cache refresh on the next device creation.
    pub fn invalidate_ios_cache(&mut self) {
        self.ios_runtimes.clear();
        self.last_updated = std::time::Instant::now()
            - std::time::Duration::from_secs(CACHE_INVALIDATION_OFFSET_SECS);
    }
}
