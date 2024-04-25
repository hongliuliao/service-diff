use std::fs::File;
use std::io::{BufReader, BufRead};

pub struct ReplayLogReader {
    reader: BufReader<File>
}

#[derive(Debug)]
pub struct ReplayLog {
    pub data: String
}

impl ReplayLogReader {
    pub fn new(file_path: &str) -> Result<Self, std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        Ok(ReplayLogReader {
            reader,
        })
    }

    pub fn read_lines(&mut self, num_lines: usize) -> Result<Vec<ReplayLog>, std::io::Error> {
        let mut lines = Vec::new();
        let mut line = String::new();

        for _ in 0..num_lines {
            if self.reader.read_line(&mut line)? == 0 {
                break;
            }
            if line.trim().is_empty() {
                continue;
            }
            let new_line = line.trim();
            let relay_log = ReplayLog{data: String::from(new_line)};
            lines.push(relay_log);
            line.clear();
        }

        Ok(lines)
    }
}


