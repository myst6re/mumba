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
                } else {
                    if number_start >= 0 {
                        let number: u16 =
                            line[number_start as usize..i].parse().unwrap_or_default();

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

    pub fn to_string(&self) -> String {
        let mut ret = String::new();

        ret.push_str("Keyboard\r\n");
        ret.push_str(format!("1. \"Select\"   {}\r\n", self.keyboard[0]).as_str());
        ret.push_str(format!("2. \"Exit\"     {}\r\n", self.keyboard[1]).as_str());
        ret.push_str(format!("3. \"Misc\"     {}\r\n", self.keyboard[2]).as_str());
        ret.push_str(format!("4. \"Menu\"     {}\r\n", self.keyboard[3]).as_str());
        ret.push_str(format!("5. \"Toggle\"   {}\r\n", self.keyboard[4]).as_str());
        ret.push_str(format!("6. \"Trigger\"  {}\r\n", self.keyboard[5]).as_str());
        ret.push_str(format!("7. \"RotLt\"    {}\r\n", self.keyboard[6]).as_str());
        ret.push_str(format!("8. \"RotRt\"    {}\r\n", self.keyboard[7]).as_str());
        ret.push_str(format!("9. \"Start\"    {}\r\n", self.keyboard[8]).as_str());
        ret.push_str(format!("10. \"Select\"  {}\r\n", self.keyboard[9]).as_str());
        ret.push_str(format!("11. \"Up\"      {}\r\n", self.keyboard[10]).as_str());
        ret.push_str(format!("12. \"Down\"    {}\r\n", self.keyboard[11]).as_str());
        ret.push_str(format!("13. \"Left\"    {}\r\n", self.keyboard[12]).as_str());
        ret.push_str(format!("14. \"Right\"   {}\r\n", self.keyboard[13]).as_str());
        ret.push_str("Joystick\r\n");
        ret.push_str(format!("1. \"Select\"   {}\r\n", self.joystick[0]).as_str());
        ret.push_str(format!("2. \"Exit\"     {}\r\n", self.joystick[1]).as_str());
        ret.push_str(format!("3. \"Misc\"     {}\r\n", self.joystick[2]).as_str());
        ret.push_str(format!("4. \"Menu\"     {}\r\n", self.joystick[3]).as_str());
        ret.push_str(format!("5. \"Toggle\"   {}\r\n", self.joystick[4]).as_str());
        ret.push_str(format!("6. \"Trigger\"  {}\r\n", self.joystick[5]).as_str());
        ret.push_str(format!("7. \"RotLt\"    {}\r\n", self.joystick[6]).as_str());
        ret.push_str(format!("8. \"RotRt\"    {}\r\n", self.joystick[7]).as_str());
        ret.push_str(format!("9. \"Start\"    {}\r\n", self.joystick[8]).as_str());
        ret.push_str(format!("10. \"Select\"  {}\r\n", self.joystick[9]).as_str());
        ret.push_str(format!("11. \"Up\"      {}\r\n", self.joystick[10]).as_str());
        ret.push_str(format!("12. \"Down\"    {}\r\n", self.joystick[11]).as_str());
        ret.push_str(format!("13. \"Left\"    {}\r\n", self.joystick[12]).as_str());
        ret.push_str(format!("14. \"Right\"   {}\r\n", self.joystick[13]).as_str());

        ret
    }
}
