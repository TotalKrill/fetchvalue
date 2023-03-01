use std::time::{Duration, Instant};

use ehttp;
use poll_promise::Promise;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Serialize)]
///  Defines an URL to continously fetch values in JSON form from
pub struct FetchValue<T>
where
    T: std::fmt::Debug + Default + DeserializeOwned + Clone + Send + 'static,
{
    value: T,
    url: String,
    max_update_rate: Option<Duration>,

    #[serde(skip)]
    value_promise: Option<Promise<ehttp::Result<T>>>,
    #[serde(skip)]
    last_update: Option<Instant>,
}

impl<T> FetchValue<T>
where
    T: std::fmt::Debug + DeserializeOwned + Default + Clone + Send + 'static,
{
    /// Create a new fetcher value, it will use the set URL to fetch information, parse it as json each time it is fetched
    pub fn new<S: ToString>(url: S) -> Self {
        Self {
            value: Default::default(),
            value_promise: None,
            url: url.to_string(),
            max_update_rate: None,
            last_update: None,
        }
    }
    /// Create a new fetcher value, it will use the set URL to fetch information, parse it as json each time it is fetched
    pub fn new_rate_limited<S: ToString>(url: S, max_rate: Duration) -> Self {
        Self {
            value: Default::default(),
            value_promise: None,
            url: url.to_string(),
            max_update_rate: Some(max_rate),
            last_update: None,
        }
    }
    // limits how often there will be new updates
    pub fn max_rate(mut self, max_rate: Duration) -> Self {
        self.max_update_rate = Some(max_rate);
        self
    }
    // sets the starting value
    pub fn starting_value(mut self, value: T) -> Self {
        self.value = value;
        self
    }
    // just kickstart the value, so that it will start updating directly
    pub fn start_now(mut self) -> Self {
        self.value = self.value();
        self
    }
    // shows when the value was last updated
    pub fn last_update(self) -> Option<Instant> {
        self.last_update
    }
    /// each time this is called, it will check if there are new information available, or try to fetch new information if there isnt
    pub fn value(&mut self) -> T {
        let update = match (self.last_update, self.max_update_rate) {
            (None, Some(_)) => true,
            (Some(last_update), Some(max_rate)) => {
                if Instant::now().duration_since(last_update) > max_rate {
                    true
                } else {
                    false
                }
            }
            (_, None) => true,
        };

        match &self.value_promise {
            None => {
                if update {
                    // Start a fetch
                    // let ctx = ctx.clone();
                    let (sender, promise) = Promise::<ehttp::Result<T>>::new();
                    let request = ehttp::Request::get(self.url.clone());
                    ehttp::fetch(request, move |response| {
                        // ctx.request_repaint(); // wake up UI thread
                        let response = response
                            .map(|resp| {
                                let text = resp.text().unwrap_or_default();
                                let respval = serde_json::from_str(&text);
                                let resp: ehttp::Result<T> = respval.map_err(|e| format!("{e}"));
                                // must fix the json
                                resp
                            })
                            .unwrap();
                        sender.send(response);
                    });
                    self.value_promise = Some(promise);
                }
            }
            Some(promise) => {
                // If the promise is ready, we reset it, so next update will generate a new one
                match promise.ready() {
                    Some(result) => {
                        match result {
                            Ok(res) => {
                                self.value = res.clone();
                                //tracing::debug!("{result:?}");
                            }
                            Err(e) => {
                                tracing::error!("{e}");
                            }
                        }
                        self.last_update = Some(Instant::now());
                        self.value_promise = None;
                    }
                    None => {
                        // otherwise we wait for ready
                    }
                }
            }
        }

        self.value.clone()
    }
}
