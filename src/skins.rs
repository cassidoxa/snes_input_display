pub mod skin {
    use ggez::{
        graphics::{Image, Rect},
        Context,
    };
    use quick_xml::{
        events::{BytesStart, Event},
        reader::Reader,
    };
    use std::{collections::HashMap, error::Error, fs, io::Read, path::Path, path::PathBuf};

    use crate::controllers::controller::Pressed;

    // #[derive(Debug)]
    pub struct Skin {
        // pub metadata: HashMap<String, String>,
        pub background: Theme,
        pub buttons: HashMap<Pressed, Button>,
        pub directory: PathBuf,
        pub name: String,
        pub theme: String,
        pub piano_roll: PianoRoll,
    }

    impl Skin {
        pub fn new(
            path: &Path,
            name: &String,
            theme: &String,
            ctx: &mut Context,
        ) -> Result<Skin, Box<dyn Error>> {
            let skin_filename = "skin.xml";
            let file_path = path.join(name).join(skin_filename);
            let directory = file_path.parent().unwrap().to_owned();

            let (backgrounds, buttons) = Self::get_layout(file_path, name, ctx)?;
            let background = Self::parse_backgrounds(backgrounds, theme).unwrap();
            Ok(Self {
                // metadata,
                piano_roll: PianoRoll::new(&background),
                background,
                buttons: Skin::parse_buttons(buttons),
                directory,
                name: name.to_owned(),
                theme: theme.to_owned(),
            })
        }

        fn get_layout(
            file_path: PathBuf,
            name: &str,
            ctx: &mut Context,
        ) -> Result<(Vec<Theme>, Vec<Button>), Box<dyn Error>> {
            let file = Self::load_file(&file_path);
            let mut reader = Reader::from_str(&file);
            let mut _metadata: HashMap<String, String> = HashMap::new();
            let mut backgrounds: Vec<Theme> = Vec::new();
            let mut buttons: Vec<Button> = Vec::new();

            loop {
                match reader.read_event() {
                    // Ok(Event::Start(t)) => _metadata = parse_attributes(t),
                    Ok(Event::Empty(t)) => match t.name().as_ref() {
                        b"background" => {
                            let bg = Theme::new(t, name, ctx)?;
                            backgrounds.push(bg);
                        }
                        b"button" => {
                            let bt = Button::new(t, name, ctx)?;
                            buttons.push(bt);
                        }
                        _ => {}
                    },
                    Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                    Ok(Event::Eof) => break,
                    _ => (),
                }
            }
            Ok((backgrounds, buttons))
        }

        fn load_file(path: &Path) -> String {
            let mut file = fs::File::open(path).unwrap();
            let mut text = String::new();
            file.read_to_string(&mut text).unwrap();
            text
        }

        fn parse_backgrounds(backgrounds_vec: Vec<Theme>, theme: &String) -> Option<Theme> {
            for background in backgrounds_vec {
                dbg!(&background);
                dbg!(&theme);
                if background.theme.eq(theme) {
                    return Some(background);
                }
            }
            None
        }

        fn parse_buttons(buttons_vec: Vec<Button>) -> HashMap<Pressed, Button> {
            let mut buttons = HashMap::new();
            for button in buttons_vec {
                buttons.insert(button.name, button);
            }
            buttons
        }
    }

    #[derive(Debug, Clone)]
    pub struct Theme {
        pub theme: String,
        pub image: Image,
        pub width: f32,
        pub height: f32,
    }

    impl Theme {
        fn new(t: BytesStart, dir: &str, ctx: &mut Context) -> Result<Self, Box<dyn Error>> {
            let attributes = parse_attributes(t);
            let image_path = Path::new("/").join(dir).join(&attributes["image"]);
            let image = Image::from_path(ctx, image_path)?;
            let width = image.width() as f32;
            let height = image.height() as f32;

            Ok(Self {
                theme: attributes["name"].to_owned().to_lowercase(),
                image,
                width,
                height,
            })
        }
    }

    #[derive(Debug)]
    pub struct Button {
        pub name: Pressed,
        pub image: Image,
        pub rect: Rect,
    }

    impl Button {
        fn new(t: BytesStart, dir: &str, ctx: &mut Context) -> Result<Self, Box<dyn Error>> {
            let attributes = parse_attributes(t);
            let x = attributes["x"].parse::<f32>().unwrap();
            let y = attributes["y"].parse::<f32>().unwrap();
            let image_path = Path::new("/").join(dir).join(&attributes["image"]);

            let image = Image::from_path(ctx, image_path)?;
            // let image_info = ImageInfo::from_file_path(&image_path)?;
            let width = image.width() as f32;
            let height = image.height() as f32;

            let name = match attributes["name"].as_str() {
                "a" => Pressed::A,
                "b" => Pressed::B,
                "x" => Pressed::X,
                "y" => Pressed::Y,
                "select" => Pressed::Select,
                "start" => Pressed::Start,
                "l" => Pressed::L,
                "r" => Pressed::R,
                "up" => Pressed::Up,
                "down" => Pressed::Down,
                "left" => Pressed::Left,
                "right" => Pressed::Right,
                _ => panic!(),
            };

            Ok(Self {
                name,
                image,
                rect: Rect::new(x, y, width, height),
            })
        }
    }

    fn parse_attributes(t: BytesStart) -> HashMap<String, String> {
        let mut attributes_map = HashMap::new();
        let attributes = t.attributes().map(|a| a.unwrap());
        for attribute in attributes {
            let value = attribute.unescape_value().unwrap().into_owned();
            let mut key = String::new();
            attribute
                .key
                .local_name()
                .into_inner()
                .read_to_string(&mut key)
                .unwrap();

            attributes_map.insert(key, value);
        }
        attributes_map
    }

    pub struct PianoRoll {
        // width of section reserved for each button
        // section_width: f32,
        // extra space left after division: modulo of section_width calculation
        // extra_width: f32,
        rect_width: f32,
        // padding inside each section
        // inside_padding: f32,
        // left_padding: f32,
        // hashmap of all positions
        pub x_positions: HashMap<Pressed, PianoRollRect>,
    }

    impl PianoRoll {
        pub fn new(background: &Theme) -> Self {
            let section_width = (&background.width / 12.0).round();
            let extra_width = (&background.width % 12.0).round();
            let inside_padding = 5.0;
            let rect_width = section_width - (inside_padding * 2.0);
            let left_padding = extra_width / 2.0;

            let mut x_positions = HashMap::new();
            x_positions.insert(
                Pressed::Left,
                PianoRollRect::new(left_padding + inside_padding),
            );
            x_positions.insert(
                Pressed::Up,
                PianoRollRect::new(x_positions[&Pressed::Left].x + section_width),
            );
            x_positions.insert(
                Pressed::Down,
                PianoRollRect::new(x_positions[&Pressed::Up].x + section_width),
            );
            x_positions.insert(
                Pressed::Right,
                PianoRollRect::new(x_positions[&Pressed::Down].x + section_width),
            );
            x_positions.insert(
                Pressed::L,
                PianoRollRect::new(x_positions[&Pressed::Right].x + section_width),
            );
            x_positions.insert(
                Pressed::Select,
                PianoRollRect::new(x_positions[&Pressed::L].x + section_width),
            );
            x_positions.insert(
                Pressed::Start,
                PianoRollRect::new(x_positions[&Pressed::Select].x + section_width),
            );
            x_positions.insert(
                Pressed::R,
                PianoRollRect::new(x_positions[&Pressed::Start].x + section_width),
            );
            x_positions.insert(
                Pressed::Y,
                PianoRollRect::new(x_positions[&Pressed::R].x + section_width),
            );
            x_positions.insert(
                Pressed::B,
                PianoRollRect::new(x_positions[&Pressed::Y].x + section_width),
            );
            x_positions.insert(
                Pressed::X,
                PianoRollRect::new(x_positions[&Pressed::B].x + section_width),
            );
            x_positions.insert(
                Pressed::A,
                PianoRollRect::new(x_positions[&Pressed::X].x + section_width),
            );

            Self {
                rect_width,
                x_positions,
            }
        }

        pub fn update(&mut self, (_, window_height): (f32, f32), events: &[Pressed]) {
            for (_, position) in self.x_positions.iter_mut() {
                position.update(&window_height);
            }

            for event in events.iter() {
                let piano_roll_rect = self.x_positions.get_mut(event).unwrap();
                piano_roll_rect.add(&window_height, &self.rect_width)
            }
        }
    }

    pub struct PianoRollRect {
        x: f32,
        pub positions: Vec<Rect>,
    }

    impl PianoRollRect {
        pub fn new(x: f32) -> Self {
            Self {
                x,
                positions: Vec::new(),
            }
        }

        pub fn add(&mut self, window_height: &f32, rect_width: &f32) {
            self.positions.push(Rect {
                x: self.x,
                y: *window_height / 2.0,
                w: *rect_width,
                h: 1.0,
            })
        }

        pub fn update(&mut self, window_height: &f32) {
            for rect in self.positions.iter_mut() {
                rect.y += 1.0;
            }

            // remove Rect from Vector if y position is larger than window height
            if !self.positions.is_empty() && self.positions[0].y > *window_height {
                self.positions.remove(0);
            }
            dbg!(&self.positions);
        }

        // pub fn Display {}
    }
}
