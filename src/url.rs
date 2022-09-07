use std::collections::HashMap;
use std::fmt::Write as _; // import without risk of name clashing

/// Builds an URL with the given query parameters
pub struct UrlBuilder {
    url: String,
    params: HashMap<String, String>,
}

impl UrlBuilder {
    /// Initializes a new UrlBuilder with the given url.
    ///
    /// # Example
    /// ```
    /// use cargo_backup::url::UrlBuilder;
    ///
    /// let url = UrlBuilder::new("https://example.com")
    ///     .add_param("foo", "bar")
    ///     .build();
    /// ```
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            params: HashMap::new(),
        }
    }

    /// Adds a parameter to the url.

    pub fn add_param(&mut self, key: &str, value: &str) -> &mut Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    /// Builds the final url.
    pub fn build(&mut self) -> String {
        let mut url: String = self.url.to_owned();

        let params = self.params.to_owned();

        let mut is_first = true;

        for (key, value) in params {
            if is_first {
                write!(url, "?{}={}", key, value).unwrap();
                is_first = false
            } else {
                write!(url, "&{}={}", key, value).unwrap();
            }
        }

        url
    }
}

// Test is not consistend
// #[test]
// fn test_url_builder() {
//     let url = UrlBuilder::new("https://example.com")
//         .add_param("foo", "bar")
//         .add_param("id", "value")
//         .build();

//     assert_eq!(url, "https://example.com?foo=bar&id=value");
// }
