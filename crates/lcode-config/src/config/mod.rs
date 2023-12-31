pub mod global;
pub mod read_config;
pub mod user_nest;

use std::{collections::VecDeque, env, path::PathBuf, str::FromStr};

use decrypt_cookies::{get_cookie, Browser, Cookies};
use miette::{IntoDiagnostic, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use user_nest::*;

use self::global::{APP_NAME, USER_CONFIG};
use crate::keymap::TuiKeyMap;

mod suffix_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    use super::user_nest::Suffix;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Suffix, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let res = match s.as_bytes() {
            b"cn" => Suffix::Cn,
            b"com" => Suffix::Com,
            _ => Suffix::Com,
        };
        Ok(res)
    }
    pub fn serialize<S>(v: &Suffix, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let res = match v {
            Suffix::Cn => "cn",
            Suffix::Com => "com",
        };
        serializer.serialize_str(res)
    }
}
/// config for user
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(skip)]
    pub urls:    Urls,
    #[serde(default)]
    pub config:  Config,
    #[serde(default)]
    pub cookies: Cookies,
    #[serde(default)]
    pub langs:   SupportLang,
    #[serde(default)]
    pub keymap:  TuiKeyMap,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub translate:   bool,
    #[serde(default, with = "suffix_serde")]
    pub url_suffix:  Suffix,
    #[serde(default)]
    pub column:      usize,
    #[serde(default)]
    pub num_sublist: u32,
    #[serde(default)]
    pub page_size:   usize,
    #[serde(default = "default_editor")]
    pub editor:      VecDeque<String>,
    #[serde(default)]
    pub lang:        String,
    #[serde(default = "default_code_dir")]
    pub code_dir:    PathBuf,
    #[serde(default)]
    pub browser:     String,
}

impl Config {
    pub fn new(suffix: Suffix) -> Self {
        let (url_suffix, translate) = match suffix {
            Suffix::Cn => (Suffix::Cn, true),
            Suffix::Com => (Suffix::Com, false),
        };
        Self {
            translate,
            url_suffix,
            ..Default::default()
        }
    }
}

/// "~/.local/share/leetcode-cn-en-cli"
fn default_code_dir() -> PathBuf {
    let mut code_dir = dirs::data_local_dir().expect("new data local dir failed");
    code_dir.push(APP_NAME);
    code_dir
}
/// Get user's editor from environment variable EDITOR and VISUAL
fn default_editor() -> VecDeque<String> {
    let editor = env::var("EDITOR").map_or_else(
        |_| env::var("VISUAL").map_or_else(|_| "vim".to_owned(), |editor| editor),
        |v| v,
    );
    VecDeque::from([editor])
}
impl Default for Config {
    fn default() -> Self {
        Self {
            translate:   false,
            column:      4,
            num_sublist: 16,
            page_size:   25,
            url_suffix:  Suffix::default(),
            editor:      default_editor(),
            lang:        "rust".to_owned(),
            code_dir:    default_code_dir(),
            browser:     String::new(),
        }
    }
}

impl User {
    ///  "cn"  "en"
    pub fn new(tongue: Suffix) -> Self {
        let config = Config::new(tongue);
        Self { config, ..Default::default() }
    }

    /// `start`, `end`, `inject_end`, `inject_end`
    pub fn get_lang_info(&self) -> (String, String, String, String) {
        macro_rules! return_info_macro {
            ($($struct_name:ident),*) => {
                match self.config.lang.as_str() {
                    $(
                        stringify!($struct_name) => self.langs.$struct_name.return_info(),
                    )*
                    _ => self.langs.rust.return_info(),
                }
            };
        }

        return_info_macro!(
            rust, bash, c, cpp, csharp, golang, java, javascript, kotlin, mysql, php, python,
            python3, ruby, scala, swift, typescript, racket, erlang, elixir, dart
        )
    }

    pub fn mod_all_pb_api(&self, category: &str) -> String {
        self.urls
            .all_problem_api
            .replace("$category", category)
    }

    pub fn mod_submit(&self, slug: &str) -> String {
        self.urls
            .submit
            .replace("$slug", slug)
    }

    pub fn mod_test(&self, slug: &str) -> String {
        self.urls
            .test
            .replace("$slug", slug)
    }

    pub fn mod_submissions(&self, id: &str) -> String {
        self.urls
            .submissions
            .replace("$id", id)
    }
    pub fn get_qsurl(&self, slug: &str) -> String {
        self.urls
            .question_url
            .replace("$slug", slug)
    }

    /// get code file suffix
    pub fn get_suffix(&self) -> &str {
        let sp_lang = &global::SUPPORT_LANGS;
        sp_lang
            .get(self.config.lang.as_str())
            .copied()
            .unwrap_or_default()
    }
}

/// headers for `LeetCode` reqwest
///
/// * `headers`: headers for reqwest
pub struct Headers {
    pub headers: HeaderMap,
}
const BROWSERS: [Browser; 4] = [
    Browser::Firefox,
    Browser::Edge,
    Browser::Chrome,
    Browser::Librewolf,
];
const LEETCODE_HOST: &str = "leetcode";

impl Headers {
    pub async fn new() -> Result<Self> {
        let default_headers = HeaderMap::new();
        let mut cookies = USER_CONFIG.cookies.clone();
        let host = format!("{}.{}", LEETCODE_HOST, USER_CONFIG.config.url_suffix);

        if !cookies.is_completion() {
            cookies = get_cookie(USER_CONFIG.config.browser.as_str(), &host).await?;
        }

        if !cookies.is_completion() {
            for i in BROWSERS {
                let pat = get_cookie(i, &host)
                    .await
                    .unwrap_or_default();
                if pat.is_completion() {
                    cookies = pat;
                    break;
                }
            }
        }

        let cookie = cookies.to_string();

        let kv_vec: Vec<(&str, &str)> = vec![
            ("Cookie", &cookie),
            ("x-csrftoken", &cookies.csrf),
            ("x-requested-with", "XMLHttpRequest"),
            ("Origin", &USER_CONFIG.urls.origin),
        ];
        let default_headers = Self::mod_headers(default_headers, kv_vec)?;

        Ok(Self { headers: default_headers })
    }

    /// new or modify headers
    ///
    /// * `headers`: be modified headers
    /// * `kv_vec`: added content
    pub fn mod_headers(mut headers: HeaderMap, kv_vec: Vec<(&str, &str)>) -> Result<HeaderMap> {
        for (k, v) in kv_vec {
            let name = HeaderName::from_str(k).into_diagnostic()?;
            let value = HeaderValue::from_str(v).into_diagnostic()?;

            headers.insert(name, value);
        }
        Ok(headers)
    }
}
