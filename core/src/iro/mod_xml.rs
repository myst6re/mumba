use quick_xml::de as xml;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct ModInfoPartial {
    // Only metadata
    #[serde(rename = "ID")]
    pub id: String,
    pub name: String,
    pub category: String,
    pub author: String,
    #[serde(default = "mod_info_version_default")]
    pub version: String,
    pub description: String,
    pub release_date: String,
    pub link: String,
    pub preview_file: String,
    pub release_notes: String,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct ModInfo {
    #[serde(rename = "ID")]
    pub id: String,
    pub name: String,
    pub category: String,
    pub author: String,
    #[serde(default = "mod_info_version_default")]
    pub version: String,
    pub description: String,
    pub contains_music: String,
    pub release_date: String,
    pub link: String,
    pub preview_file: String,
    pub release_notes: String,
    pub mod_folder: Vec<ModFolder>,
    pub conditional: Vec<ConditionalFolder>,
    pub config_option: Vec<ConfigOption>,
    pub load_library: Vec<HashMap<String, String>>,
    pub load_assembly: Vec<HashMap<String, String>>,
    pub load_plugin: Vec<HashMap<String, String>>,
    #[serde(rename = "FFNxConfig")]
    pub ffnx_config: Vec<HashMap<String, HashMap<String, Vec<String>>>>,
    pub variable: Vec<Variable>,
    pub load_programs: LoadPrograms,
    pub order_constraints: Vec<OrderConstraints>,
    pub compatibility: Compatibility,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct ModFolder {
    #[serde(rename = "@Folder")]
    pub folder: String,
    #[serde(rename = "@ActiveWhen")]
    pub active_when_compat: Option<String>,
    pub active_when: ActiveWhen,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct ActiveWhen {
    #[serde(rename = "$value")]
    pub value: ActiveWhenValue,
}

#[derive(Deserialize, Debug, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub enum ActiveWhenValue {
    Or {
        #[serde(rename = "$value")]
        value: Vec<ActiveWhenValue>,
    },
    And {
        #[serde(rename = "$value")]
        value: Vec<ActiveWhenValue>,
    },
    Not {
        #[serde(rename = "$value")]
        value: Box<ActiveWhenValue>,
    },
    Option(String),
    #[serde(rename = "$text")]
    Text(String),
    #[serde(other)]
    #[default]
    Other,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct ConditionalFolder {
    #[serde(rename = "@Folder")]
    pub folder: String,
    #[serde(rename = "@ActiveWhen")]
    pub active_when_compat: Option<String>,
    pub active_when: ActiveWhen,
    pub or: Vec<Conditional>,
    pub and: Vec<Conditional>,
    pub not: Vec<Conditional>,
    pub runtime_var: Vec<RuntimeVar>,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct Conditional {
    #[serde(rename = "@ApplyTo")]
    pub apply_to: String,
    #[serde(rename = "$value")]
    pub value: ConditionalValue,
}

#[derive(Deserialize, Debug, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub enum ConditionalValue {
    Or {
        // https://github.com/tafia/quick-xml/issues/819
        #[serde(rename = "$value")]
        value: Vec<ConditionalValue>,
    },
    And {
        #[serde(rename = "$value")]
        value: Vec<ConditionalValue>,
    },
    Not {
        #[serde(rename = "$value")]
        value: Box<ConditionalValue>,
    },
    RuntimeVar(RuntimeVar),
    #[serde(rename = "$text")]
    Text(String),
    #[serde(other)]
    #[default]
    Other,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct RuntimeVar {
    #[serde(rename = "@Var")]
    pub var: String,
    #[serde(rename = "@Values")]
    pub values: String,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct ConfigOption {
    #[serde(rename = "Type")]
    pub ttype: String,
    pub default: i64,
    pub name: String,
    #[serde(rename = "ID")]
    pub id: String,
    pub description: String,
    pub option: Vec<OptionValue>,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct OptionValue {
    #[serde(rename = "@Value")]
    pub value: i64,
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@PreviewFile")]
    pub preview_file: String,
    #[serde(rename = "@PreviewAudio")]
    pub preview_audio: String,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct Variable {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct LoadPrograms {
    pub path_to_program: String,
    pub program_args: String,
    pub close_all_instances: bool,
    pub window_title: String,
    pub wait_for_window_to_show: bool,
    pub wait_time_out_in_seconds: u32,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct OrderConstraints {
    pub before: Vec<String>, // uuid
    pub after: Vec<String>,  // uuid
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct Compatibility {
    pub require: Vec<CompatEntry>,
    pub forbid: Vec<CompatEntry>,
    pub setting: Vec<CompatSetting>,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct CompatEntry {
    #[serde(rename = "@ModID")]
    pub mod_id: String, // guid
    #[serde(rename = "$text")]
    pub description: String,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct CompatSetting {
    #[serde(rename = "MyID")]
    pub my_id: String,
    pub my_value: i64,
    #[serde(rename = "ModID")]
    pub mod_id: String, // guid
    #[serde(rename = "TheirID")]
    pub their_id: String,
    pub require: i64,
    pub forbid: Vec<i64>,
}

fn mod_info_version_default() -> String {
    String::from("0.00")
}

pub fn deserialize_mod_xml_partial<R: std::io::BufRead>(
    reader: R,
) -> Result<ModInfoPartial, xml::DeError> {
    xml::from_reader(reader)
}

pub fn deserialize_mod_xml<R: std::io::BufRead>(reader: R) -> Result<ModInfo, xml::DeError> {
    xml::from_reader(reader)
}
