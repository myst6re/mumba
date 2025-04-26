use log::info;
use md5::{Digest, Md5};
use quick_xml::events::{BytesText, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn update_metadata<'a>(
    save_dir: &PathBuf,
    slot: u8,
    num: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata_path = save_dir.join("metadata.xml");
    if !metadata_path.exists() {
        info!("Creating {}", metadata_path.to_string_lossy());
        create_metadata(&metadata_path)?
    }
    let user_id = &save_dir.file_name().unwrap().to_string_lossy()[5..];
    let mut reader = Reader::from_file(&metadata_path)?;
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
    let mut buf = Vec::new();
    let mut is_choco_element = false;
    let mut is_ff8_element = false;
    let mut current_slot = 0;
    let mut current_num = 0;
    let mut is_timestamp_element = false;
    let mut is_signature_element = false;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .to_string();
    let mut hasher = Md5::new();
    let save_data = get_save_data(save_dir, slot, num)?;
    hasher.update([save_data.as_slice(), user_id.as_bytes()].concat());
    let result = hasher.finalize();
    let signature = String::from_utf8(result[..].to_vec()).unwrap();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"savefile" => {
                match e
                    .attributes()
                    .map(|attr| attr.unwrap())
                    .find(|e| e.key.as_ref() == b"type")
                    .map(|e| e.value)
                {
                    Some(t) if t.as_ref() == b"ff8" => {
                        is_ff8_element = true;
                        current_slot = e
                            .attributes()
                            .map(|attr| attr.unwrap())
                            .find(|e| e.key.as_ref() == b"slot")
                            .map(|e| String::from_utf8_lossy(&e.value).parse::<u8>().unwrap())
                            .unwrap_or(0);
                        current_num = e
                            .attributes()
                            .map(|attr| attr.unwrap())
                            .find(|e| e.key.as_ref() == b"num")
                            .map(|e| String::from_utf8_lossy(&e.value).parse::<u8>().unwrap())
                            .unwrap_or(0);
                    }
                    Some(t) if t.as_ref() == b"choco" => is_choco_element = true,
                    Some(_) => (),
                    None => (),
                };
                assert!(writer.write_event(Event::Start(e.borrow())).is_ok());
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"savefile" => {
                is_ff8_element = false;
                is_choco_element = false;
                assert!(writer.write_event(Event::End(e.borrow())).is_ok());
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"timestamp" => {
                is_timestamp_element = true;
                assert!(writer.write_event(Event::Start(e.borrow())).is_ok());
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"timestamp" => {
                is_timestamp_element = false;
                assert!(writer.write_event(Event::End(e.borrow())).is_ok());
            }
            Ok(Event::Start(e)) if e.name().as_ref() == b"signature" => {
                is_signature_element = true;
                assert!(writer.write_event(Event::Start(e.borrow())).is_ok());
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"signature" => {
                is_signature_element = false;
                assert!(writer.write_event(Event::End(e.borrow())).is_ok());
            }
            Ok(Event::Text(e)) => {
                if is_choco_element && slot == 3
                    || is_ff8_element && slot == current_slot && num == current_num
                {
                    if is_timestamp_element {
                        assert!(writer
                            .write_event(Event::Text(BytesText::new(&timestamp)))
                            .is_ok())
                    } else if is_signature_element {
                        assert!(writer
                            .write_event(Event::Text(BytesText::new(&signature)))
                            .is_ok())
                    } else {
                        assert!(writer.write_event(Event::Text(e.borrow())).is_ok())
                    }
                } else {
                    assert!(writer.write_event(Event::Text(e.borrow())).is_ok())
                }
            }
            Ok(Event::Eof) => break,
            Ok(e) => assert!(writer.write_event(e.borrow()).is_ok()),
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
        }
    }
    info!("Updating {}", metadata_path.to_string_lossy());
    let mut file = File::create(&metadata_path)?;
    file.write(writer.into_inner().into_inner().as_slice())?;
    Ok(())
}

fn get_save_data(save_dir: &PathBuf, slot: u8, num: u8) -> std::io::Result<Vec<u8>> {
    let file_name = if slot > 2 {
        String::from("chocorpg.ff8")
    } else {
        format!("slot{}_save{:02}.ff8", slot, num)
    };
    let mut file = File::open(save_dir.join(file_name))?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn create_metadata(metadata_path: &PathBuf) -> std::io::Result<()> {
    let contents = r#"
<?xml version="1.0" encoding="UTF-8"?>
<gamestatus>
  <savefile num="1" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="2" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="3" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="4" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="5" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="6" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="7" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="8" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="9" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="10" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="11" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="12" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="13" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="14" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="15" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="16" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="17" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="18" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="19" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="20" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="21" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="22" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="23" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="24" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="25" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="26" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="27" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="28" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="29" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="30" type="ff8" slot="1">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="1" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="2" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="3" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="4" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="5" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="6" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="7" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="8" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="9" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="10" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="11" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="12" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="13" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="14" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="15" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="16" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="17" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="18" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="19" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="20" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="21" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="22" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="23" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="24" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="25" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="26" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="27" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="28" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="29" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile num="30" type="ff8" slot="2">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
  <savefile type="choco">
    <timestamp></timestamp>
    <signature></signature>
  </savefile>
</gamestatus>
"#;
    std::fs::write(metadata_path, contents)
}
