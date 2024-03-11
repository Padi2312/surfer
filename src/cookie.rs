#[derive(Debug)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub expires: Option<String>,
    pub max_age: Option<i64>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
    pub same_site: Option<String>,
}

impl Cookie {
    pub fn new(name: &str, value: &str) -> Cookie {
        Cookie {
            name: name.to_string(),
            value: value.to_string(),
            expires: None,
            max_age: None,
            domain: None,
            path: None,
            secure: None,
            http_only: None,
            same_site: None,
        }
    }

    pub fn with_expires(mut self, expires: &str) -> Cookie {
        self.expires = Some(expires.to_string());
        self
    }

    pub fn with_max_age(mut self, max_age: i64) -> Cookie {
        self.max_age = Some(max_age);
        self
    }

    pub fn with_domain(mut self, domain: &str) -> Cookie {
        self.domain = Some(domain.to_string());
        self
    }

    pub fn with_path(mut self, path: &str) -> Cookie {
        self.path = Some(path.to_string());
        self
    }

    pub fn with_secure(mut self, secure: bool) -> Cookie {
        self.secure = Some(secure);
        self
    }

    pub fn with_http_only(mut self, http_only: bool) -> Cookie {
        self.http_only = Some(http_only);
        self
    }

    pub fn with_same_site(mut self, same_site: &str) -> Cookie {
        self.same_site = Some(same_site.to_string());
        self
    }

    pub fn to_string(&self) -> String {
        let mut cookie = format!("{}={}", self.name, self.value);
        if let Some(expires) = &self.expires {
            cookie.push_str(&format!("; Expires={}", expires));
        }
        if let Some(max_age) = &self.max_age {
            cookie.push_str(&format!("; Max-Age={}", max_age));
        }
        if let Some(domain) = &self.domain {
            cookie.push_str(&format!("; Domain={}", domain));
        }
        if let Some(path) = &self.path {
            cookie.push_str(&format!("; Path={}", path));
        }
        if let Some(same_site) = &self.same_site {
            cookie.push_str(&format!("; SameSite={}", same_site));
        }
        if let Some(secure) = &self.secure {
            if *secure {
                cookie.push_str("; Secure");
            }
        }
        if let Some(http_only) = &self.http_only {
            if *http_only {
                cookie.push_str("; HttpOnly");
            }
        }
        cookie
    }
}
