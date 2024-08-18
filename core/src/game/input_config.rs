use crate::game::installation::Edition;
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

pub struct InputConfig {
    pub keyboard: [u8; 14],
    pub joystick: [u8; 14],
}

impl InputConfig {
    pub fn new(edition: &Edition) -> InputConfig {
        InputConfig {
            keyboard: [32, 45, 30, 17, 16, 18, 44, 46, 31, 33, 200, 208, 203, 205],
            joystick: if matches!(edition, Edition::Standard) {
                [
                    226, 225, 224, 227, 228, 229, 230, 231, 232, 233, 252, 253, 254, 255,
                ]
            } else {
                [
                    225, 224, 226, 227, 228, 229, 232, 233, 230, 231, 252, 253, 254, 255,
                ]
            },
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<InputConfig> {
        let mut reader = BufReader::new(File::open(path)?);
        let mut line = String::new();
        let mut current = 0;
        let mut ret = InputConfig {
            keyboard: [0; 14],
            joystick: [0; 14],
        };
        'outer: while current < 28 {
            let len = reader.read_line(&mut line)?;
            if len == 0 {
                break;
            }
            let mut number_start = -1;
            let mut first_number: u8 = 0;
            let mut second_number: u8 = 0;
            for (i, c) in line.chars().enumerate() {
                if c >= '0' && c <= '9' {
                    if number_start < 0 {
                        number_start = i as i32
                    }
                } else if number_start >= 0 {
                    let number: u16 = line[number_start as usize..i].parse().unwrap_or_default();

                    if first_number == 0 {
                        if number < 1 || number > 14 {
                            continue 'outer;
                        }

                        first_number = number as u8;
                        number_start = -1
                    } else {
                        if number > 255 {
                            continue 'outer;
                        }

                        second_number = number as u8;
                        number_start = -2;

                        break;
                    }
                }
            }

            if first_number == 0 {
                continue;
            }

            if number_start >= 0 {
                let number: u16 = line[number_start as usize..].parse().unwrap_or_default();

                if number > 255 {
                    continue;
                }

                second_number = number as u8
            } else if number_start == -1 {
                continue;
            }

            if current < 14 {
                ret.keyboard[first_number as usize - 1] = second_number;
            } else {
                ret.joystick[first_number as usize - 1] = second_number;
            }

            current += 1
        }
        if current >= 28 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "The file needs 28 entries to be valid",
            ))
        } else {
            Ok(ret)
        }
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(self.to_string().as_bytes())?;
        Ok(())
    }
}

impl Display for InputConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Keyboard\r\n")?;
        write!(f, "1. \"Select\"   {}\r\n", self.keyboard[0])?;
        write!(f, "2. \"Exit\"     {}\r\n", self.keyboard[1])?;
        write!(f, "3. \"Misc\"     {}\r\n", self.keyboard[2])?;
        write!(f, "4. \"Menu\"     {}\r\n", self.keyboard[3])?;
        write!(f, "5. \"Toggle\"   {}\r\n", self.keyboard[4])?;
        write!(f, "6. \"Trigger\"  {}\r\n", self.keyboard[5])?;
        write!(f, "7. \"RotLt\"    {}\r\n", self.keyboard[6])?;
        write!(f, "8. \"RotRt\"    {}\r\n", self.keyboard[7])?;
        write!(f, "9. \"Start\"    {}\r\n", self.keyboard[8])?;
        write!(f, "10. \"Select\"  {}\r\n", self.keyboard[9])?;
        write!(f, "11. \"Up\"      {}\r\n", self.keyboard[10])?;
        write!(f, "12. \"Down\"    {}\r\n", self.keyboard[11])?;
        write!(f, "13. \"Left\"    {}\r\n", self.keyboard[12])?;
        write!(f, "14. \"Right\"   {}\r\n", self.keyboard[13])?;
        write!(f, "Joystick\r\n")?;
        write!(f, "1. \"Select\"   {}\r\n", self.joystick[0])?;
        write!(f, "2. \"Exit\"     {}\r\n", self.joystick[1])?;
        write!(f, "3. \"Misc\"     {}\r\n", self.joystick[2])?;
        write!(f, "4. \"Menu\"     {}\r\n", self.joystick[3])?;
        write!(f, "5. \"Toggle\"   {}\r\n", self.joystick[4])?;
        write!(f, "6. \"Trigger\"  {}\r\n", self.joystick[5])?;
        write!(f, "7. \"RotLt\"    {}\r\n", self.joystick[6])?;
        write!(f, "8. \"RotRt\"    {}\r\n", self.joystick[7])?;
        write!(f, "9. \"Start\"    {}\r\n", self.joystick[8])?;
        write!(f, "10. \"Select\"  {}\r\n", self.joystick[9])?;
        write!(f, "11. \"Up\"      {}\r\n", self.joystick[10])?;
        write!(f, "12. \"Down\"    {}\r\n", self.joystick[11])?;
        write!(f, "13. \"Left\"    {}\r\n", self.joystick[12])?;
        write!(f, "14. \"Right\"   {}\r\n", self.joystick[13])
    }
}
