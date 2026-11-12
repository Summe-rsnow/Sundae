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
    pub bins: Vec<String>,
    pub depends: Vec<String>,
}

#[derive(Deserialize)]
struct RawManifest {
    version: Option<String>,
    description: Option<String>,
    homepage: Option<String>,
    license: Option<Value>,
    bin: Option<Value>,
    depends: Option<Value>,
}

impl PackageManifest {
    pub fn from_json(name: &str, bucket: &str, json_str: &str) -> Result<Self, serde_json::Error> {
        let raw: RawManifest = serde_json::from_str(json_str)?;

        let version = raw.version.unwrap_or_else(|| "Unknown".to_string());
        let description = raw.description.unwrap_or_default();
        let homepage = raw.homepage.unwrap_or_default();

        let license = match raw.license {
            Some(Value::String(s)) => s,
            Some(Value::Object(obj)) => obj
                .get("identifier")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            _ => "Unknown".to_string(),
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

        Ok(Self {
            name: name.to_string(),
            bucket: bucket.to_string(),
            version,
            description,
            homepage,
            license,
            bins,
            depends,
        })
    }
}
