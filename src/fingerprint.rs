use actix_web::HttpRequest;
use lru_time_cache::LruCache;

#[derive(Clone)]
pub struct FingerPrint {
    cache: LruCache<String, u32>,
}

impl FingerPrint {
    pub fn new() -> FingerPrint {
        let time_to_live = ::std::time::Duration::from_secs(60);
        let lru_cache = LruCache::<String, u32>::with_expiry_duration(time_to_live);

        FingerPrint { cache: lru_cache }
    }

    pub fn add_request(&mut self, req: HttpRequest) -> u32 {
        if let Some(user_agent) = req.headers().get(actix_web::http::header::USER_AGENT) {
            let user_agent = user_agent.to_str();

            if let Ok(user_agent) = user_agent {
                let count: u32 = if self.cache.get_mut(user_agent).is_some() {
                    *self.cache.get_mut(user_agent).unwrap()
                } else {
                    0_u32
                };
                let count = count + 1;
                self.cache.insert(user_agent.to_string(), count);
                return count;
            }
        }
        1000 // If no user agent erro force Captcha
    }
}
