use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ThemeAppearance {
    Dark,
    Light,
    Auto,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ThemeFile {
    #[serde(default, rename = "$schema")]
    pub(crate) schema: Option<String>,
    pub(crate) schema_version: u32,
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) author: Option<String>,
    #[serde(default)]
    pub(crate) version: Option<String>,
    pub(crate) appearance: ThemeAppearance,
    #[serde(default)]
    pub(crate) extends: Option<String>,
    #[serde(default)]
    pub(crate) palette: BTreeMap<String, ThemeValue>,
    #[serde(default)]
    pub(crate) tokens: BTreeMap<String, ThemeValue>,
    #[serde(default)]
    pub(crate) components: ComponentOverrides,
}

impl ThemeFile {
    pub(crate) fn invalid_placeholder(id: String) -> Self {
        Self {
            schema: None,
            schema_version: 1,
            id: id.clone(),
            name: id,
            description: None,
            author: None,
            version: None,
            appearance: ThemeAppearance::Auto,
            extends: None,
            palette: BTreeMap::new(),
            tokens: BTreeMap::new(),
            components: ComponentOverrides::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComponentOverrides {
    #[serde(default)]
    pub(crate) composer: BTreeMap<String, ThemeValue>,
    #[serde(default)]
    pub(crate) tool_call: BTreeMap<String, ThemeValue>,
    #[serde(default)]
    pub(crate) user_message: BTreeMap<String, ThemeValue>,
    #[serde(default)]
    pub(crate) assistant_message: BTreeMap<String, ThemeValue>,
    #[serde(default)]
    pub(crate) picker: BTreeMap<String, ThemeValue>,
    #[serde(default)]
    pub(crate) modal: BTreeMap<String, ThemeValue>,
    #[serde(flatten)]
    pub(crate) unknown: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum ThemeValue {
    String(String),
    Style(ThemeStyleObject),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ThemeStyleObject {
    #[serde(default)]
    pub(crate) foreground: Option<String>,
    #[serde(default)]
    pub(crate) background: Option<String>,
    #[serde(default)]
    pub(crate) modifiers: Vec<String>,
}
