use mumba_core::iro::mod_xml::{
    ActiveWhen, ActiveWhenValue, CompatEntry, CompatSetting, Compatibility, Conditional,
    ConditionalFolder, ConditionalValue, ConfigOption, LoadPrograms, ModFolder, ModInfo,
    OptionValue, OrderConstraints, RuntimeVar,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[test]
#[cfg_attr(not(feature = "iro"), ignore)]
fn it_parses_a_valid_mod_xml() {
    let file = BufReader::new(
        File::open(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/mod.xml")).unwrap(),
    );

    assert_eq!(
        ModInfo::from_reader(file).unwrap(),
        ModInfo {
            id: String::from("foobar-id"),
            name: String::from("Fake mod"),
            category: String::from("Media"),
            author: String::from("myst6re"),
            version: String::from("1.00"),
            description: String::from("My super mod"),
            contains_music: String::from("false"),
            release_date: String::from("2020-10-19T22:12:35.8393223+02:00"),
            link: String::from("https://www.example.org"),
            preview_file: String::from("preview.jpg"),
            release_notes: String::new(),
            mod_folder: vec![
                ModFolder {
                    folder: String::from("data/foo/bar"),
                    active_when_compat: Some(String::from("1")),
                    active_when: ActiveWhen {
                        value: ActiveWhenValue::Other
                    }
                },
                ModFolder {
                    folder: String::from("MyModSubFolder1"),
                    active_when_compat: None,
                    active_when: ActiveWhen {
                        value: ActiveWhenValue::Not {
                            value: Box::new(ActiveWhenValue::And {
                                value: vec![ActiveWhenValue::Option(String::from("MySetting = 1"))]
                            })
                        }
                    }
                },
                ModFolder {
                    folder: String::from("MyModSubFolder2"),
                    active_when_compat: None,
                    active_when: ActiveWhen {
                        value: ActiveWhenValue::Or {
                            value: vec![ActiveWhenValue::Option(String::from("MySetting = 2"))]
                        }
                    }
                }
            ],
            conditional: vec![
                ConditionalFolder {
                    folder: String::from("sewer"),
                    active_when_compat: None,
                    active_when: ActiveWhen {
                        value: ActiveWhenValue::Other
                    },
                    or: vec![],
                    and: vec![],
                    not: vec![Conditional {
                        apply_to: String::from("apply"),
                        value: ConditionalValue::Or {
                            value: vec![ConditionalValue::RuntimeVar(RuntimeVar {
                                var: String::from("Test"),
                                values: String::from("Value")
                            })]
                        }
                    }],
                    runtime_var: vec![RuntimeVar {
                        var: String::from("Short:0xCC15D0:2"),
                        values: String::from("0xD5,0x160")
                    }]
                },
                ConditionalFolder {
                    folder: String::from("foobar"),
                    active_when_compat: None,
                    active_when: ActiveWhen {
                        value: ActiveWhenValue::Other
                    },
                    or: vec![],
                    and: vec![],
                    not: vec![],
                    runtime_var: vec![],
                }
            ],
            config_option: vec![ConfigOption {
                ttype: String::from("Bool"),
                default: 0,
                id: String::from("outfits"),
                name: String::from("Mod name"),
                description: String::from("Description"),
                option: vec![OptionValue {
                    value: 0,
                    name: String::from(""),
                    preview_file: String::from("preview\\hollyoff.jpg"),
                    preview_audio: String::new()
                }]
            }],
            load_library: vec![],
            load_assembly: vec![],
            load_plugin: vec![],
            ffnx_config: vec![HashMap::from([
                (
                    String::from("use_external_music"),
                    HashMap::from([(String::from("$text"), vec![String::from("true")]),])
                ),
                (
                    String::from("external_music_ext"),
                    HashMap::from([(
                        String::from("ext"),
                        vec![String::from("minipsf"), String::from("wav")]
                    )])
                ),
            ])],
            variable: vec![],
            load_programs: LoadPrograms {
                path_to_program: String::new(),
                program_args: String::new(),
                close_all_instances: false,
                window_title: String::new(),
                wait_for_window_to_show: false,
                wait_time_out_in_seconds: 0
            },
            order_constraints: vec![OrderConstraints {
                after: vec![String::from("0000-001")],
                before: vec![String::from("0000-002"), String::from("0000-003")],
            }],
            compatibility: Compatibility {
                require: vec![CompatEntry {
                    mod_id: String::from("0000-000"),
                    description: String::from("Name of Other Mod")
                }],
                forbid: vec![],
                setting: vec![CompatSetting {
                    my_id: String::from("MySettingVariable"),
                    my_value: 1,
                    mod_id: String::from("0000-003"),
                    their_id: String::from("Their id"),
                    require: 0,
                    forbid: vec![]
                }]
            },
        }
    );
}
