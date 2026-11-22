use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifest {
    pub name: String,
    pub bucket: String,
    pub version: String,
    pub description: String,
    pub homepage: String,
    pub license: String,
    pub license_url: Option<String>,
    pub bins: Vec<String>,
    pub depends: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Deserialize)]
struct RawManifest {
    version: Option<String>,
    description: Option<Value>,
    homepage: Option<String>,
    license: Option<Value>,
    bin: Option<Value>,
    depends: Option<Value>,
    notes: Option<Value>,
}

impl PackageManifest {
    pub fn from_json(name: &str, bucket: &str, json_str: &str) -> Result<Self, serde_json::Error> {
        let raw: RawManifest = serde_json::from_str(json_str)?;

        let version = raw.version.unwrap_or_else(|| "Unknown".to_string());
        let description = match raw.description {
            Some(Value::String(s)) => s,
            Some(Value::Array(arr)) => arr
                .into_iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
                .join(" "),
            _ => String::new(),
        };
        let homepage = raw.homepage.unwrap_or_default();

        let (license, license_url) = match raw.license {
            Some(Value::String(s)) => (s, None),
            Some(Value::Object(obj)) => {
                let id = obj
                    .get("identifier")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let url = obj.get("url").and_then(|v| v.as_str()).map(str::to_owned);
                (id, url)
            }
            _ => ("Unknown".to_string(), None),
        };

        let mut bins = Vec::new();
        match raw.bin {
            Some(Value::String(s)) => {
                if !bins.contains(&s) {
                    bins.push(s);
                }
            }
            Some(Value::Array(arr)) => {
                for item in arr {
                    match item {
                        Value::String(s) => {
                            if !bins.contains(&s) {
                                bins.push(s);
                            }
                        }
                        Value::Array(inner) => {
                            // Format: ["target.exe", "alias.exe", ...]
                            for alias in inner {
                                if let Value::String(s) = alias {
                                    if !bins.contains(&s) {
                                        bins.push(s);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        let mut depends = Vec::new();
        match raw.depends {
            Some(Value::String(s)) => depends.push(s),
            Some(Value::Array(arr)) => {
                for item in arr {
                    if let Value::String(s) = item {
                        depends.push(s);
                    }
                }
            }
            _ => {}
        }

        let mut notes = Vec::new();
        match raw.notes {
            Some(Value::String(s)) => {
                if !s.trim().is_empty() {
                    notes.push(s);
                }
            }
            Some(Value::Array(arr)) => {
                for item in arr {
                    if let Value::String(s) = item {
                        notes.push(s);
                    }
                }
            }
            _ => {}
        }

        Ok(Self {
            name: name.to_string(),
            bucket: bucket.to_string(),
            version,
            description,
            homepage,
            license,
            license_url,
            bins,
            depends,
            notes,
        })
    }

    pub fn install_command(&self) -> String {
        if self.bucket.is_empty() {
            format!("scoop install {}", self.name)
        } else {
            format!("scoop install {}/{}", self.bucket, self.name)
        }
    }

    pub fn display_description(&self) -> &str {
        if self.description.trim().is_empty() {
            "暂无描述"
        } else {
            self.description.as_str()
        }
    }
}
