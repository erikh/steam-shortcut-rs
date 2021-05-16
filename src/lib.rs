use std::{collections::HashMap, time::SystemTime};

#[derive(Debug, Clone)]
pub struct Shortcut {
    id: u32,
    app_name: String,
    exe: String,
    start_dir: String,
    is_hidden: bool,
    icon: String,
    launch_options: String,
    allow_desktop_config: bool,
    shortcut_path: String,
    last_play_time: SystemTime,
    open_vr: bool,
    tags: Vec<String>,
}

impl Default for Shortcut {
    fn default() -> Self {
        Self {
            id: 0,
            app_name: String::from("Default Shortcut"),
            exe: String::from("calc.exe"),
            start_dir: String::from("/"),
            is_hidden: false,
            icon: String::new(),
            launch_options: String::new(),
            allow_desktop_config: true,
            shortcut_path: String::new(),
            last_play_time: SystemTime::now(),
            open_vr: false,
            tags: Vec::new(),
        }
    }
}

impl Shortcut {
    pub fn new(
        id: u32,
        app_name: &str,
        exe: &str,
        start_dir: &str,
        is_hidden: bool,
        icon: String,
        launch_options: &str,
        allow_desktop_config: bool,
        shortcut_path: &str,
        last_play_time: SystemTime,
        open_vr: bool,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id,
            app_name: app_name.to_string(),
            exe: exe.to_string(),
            start_dir: start_dir.to_string(),
            is_hidden,
            icon: icon.to_string(),
            launch_options: launch_options.to_string(),
            allow_desktop_config,
            shortcut_path: shortcut_path.to_string(),
            last_play_time,
            open_vr,
            tags,
        }
    }
}

type LooseMap = HashMap<String, Box<dyn std::any::Any>>;

impl From<&LooseMap> for Shortcut {
    fn from(t: &LooseMap) -> Self {
        Self {
            id: 0,
            //id: *t.get("id").unwrap().clone().downcast_ref::<u32>().unwrap(),
            app_name: String::from(
                (**t.get("AppName").clone().unwrap())
                    .downcast_ref::<String>()
                    .unwrap(),
            ),
            exe: String::from(
                t.get("exe")
                    .unwrap()
                    .clone()
                    .downcast_ref::<String>()
                    .unwrap(),
            ),
            start_dir: String::from(t.get("StartDir").unwrap().downcast_ref::<String>().unwrap()),
            is_hidden: *t.get("IsHidden").unwrap().downcast_ref::<u32>().unwrap() == 1,
            icon: String::from(t.get("icon").unwrap().downcast_ref::<String>().unwrap()),
            launch_options: String::from(
                t.get("LaunchOptions")
                    .unwrap()
                    .downcast_ref::<String>()
                    .unwrap(),
            ),
            allow_desktop_config: *t
                .get("AllowDesktopConfig")
                .unwrap()
                .downcast_ref::<u32>()
                .unwrap()
                == 1,
            open_vr: *t.get("OpenVR").unwrap().downcast_ref::<u32>().unwrap() == 1,
            shortcut_path: String::from(
                t.get("ShortcutPath")
                    .unwrap()
                    .downcast_ref::<String>()
                    .unwrap(),
            ),
            last_play_time: SystemTime::UNIX_EPOCH
                + std::time::Duration::from_secs(
                    *t.get("LastPlayTime")
                        .unwrap()
                        .downcast_ref::<u32>()
                        .unwrap() as u64,
                ),
            tags: Vec::new(),
        }
    }
}

pub mod parser {
    use std::io::{Bytes, Read};
    use std::{any::Any, fs::File};

    use crate::{LooseMap, Shortcut};

    const TYPE_OBJECT: u8 = 0;
    const TYPE_STRING: u8 = 1;
    const TYPE_INT: u8 = 2;

    const TERMINATOR_SHORTCUT: u8 = 8;
    const TERMINATOR_STRING: u8 = 0; // probably could just use cstr handlers for this

    fn read_next_string(handle: &mut Bytes<File>) -> String {
        String::from_utf8(
            handle
                .take_while(|c| match *c {
                    Ok(c) => c != TERMINATOR_STRING,
                    _ => false,
                })
                .map(|c| c.unwrap())
                .collect::<Vec<u8>>(),
        )
        .expect("invalid UTF-8 in shortcut definition")
    }

    fn parse_object(handle: &mut Bytes<File>) -> Result<Box<LooseMap>, std::io::Error> {
        let mut loose_map = Box::new(LooseMap::new());

        while let Some(Ok(header)) = handle.next() {
            if header == TERMINATOR_SHORTCUT {
                return Ok(loose_map);
            }

            let property: String = read_next_string(handle);

            match header {
                TYPE_OBJECT => {
                    eprintln!("Type: object");
                    let obj = parse_object(handle)?;
                    loose_map.insert(property, obj);
                }
                TYPE_STRING => {
                    let val = read_next_string(handle);
                    loose_map.insert(property, Box::new(val.clone()));
                    eprintln!("Type: string: {}", val);
                }
                TYPE_INT => {
                    let mut target: u32 = 0;

                    let mut i = 0;
                    for x in handle.take(4) {
                        target |= (x? as u32) << i;
                        i += 1;
                    }

                    loose_map.insert(property, Box::new(target));
                    eprintln!("Type: int: {}", target);
                }
                _ => {
                    eprintln!("Unrecognized type {}", header);
                }
            };
        }

        Ok(loose_map)
    }

    pub struct Parser {
        filename: String,
        parsed_map: LooseMap,
        idx: u32,
    }

    impl Parser {
        pub fn new(filename: &str) -> Result<Self, std::io::Error> {
            let mut handle = std::fs::File::open(filename)?.bytes();
            let mut obj = parse_object(&mut handle)?;
            let s: Box<dyn Any + 'static> = obj.remove("shortcuts").unwrap();

            let parsed_map: LooseMap = *s.downcast::<LooseMap>().unwrap();

            Ok(Self {
                filename: filename.to_string(),
                parsed_map,
                idx: 0,
            })
        }

        pub fn filename(self) -> String {
            return self.filename;
        }
    }

    impl Iterator for Parser {
        type Item = crate::Shortcut;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(any) = self.parsed_map.get(&format!("{}", self.idx)) {
                let map = any.downcast_ref::<LooseMap>().unwrap();
                let mut sc = Shortcut::from(map);
                sc.id = self.idx;
                self.idx += 1;
                return Some(sc);
            }
            None
        }
    }
}
