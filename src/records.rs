use crate::game::MAX_LEVELS;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Record {
    pub best_time: Option<f32>,
    pub best_score: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Improvement {
    pub time: bool,
    pub score: bool,
}

pub struct Records {
    entries: Vec<Record>,
    path: PathBuf,
}

fn default_path() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let dir = PathBuf::from(appdata).join("ZombiePlantGUI");
        let _ = fs::create_dir_all(&dir);
        dir.join("records.txt")
    } else {
        PathBuf::from("zombie-plant-gui-records.txt")
    }
}

impl Records {
    pub fn load() -> Self {
        Self::from_path(default_path())
    }

    pub fn from_path(path: PathBuf) -> Self {
        let mut entries = vec![Record::default(); MAX_LEVELS];
        if let Ok(text) = fs::read_to_string(&path) {
            for line in text.lines() {
                let mut parts = line.split_whitespace();
                let Some(level) = parts.next().and_then(|p| p.parse::<usize>().ok()) else {
                    continue;
                };
                if level == 0 || level > MAX_LEVELS {
                    continue;
                }
                let Some(time) = parts.next().and_then(|p| p.parse::<f32>().ok()) else {
                    continue;
                };
                let Some(score) = parts.next().and_then(|p| p.parse::<i32>().ok()) else {
                    continue;
                };
                entries[level - 1] = Record {
                    best_time: Some(time),
                    best_score: score,
                };
            }
        }
        Self { entries, path }
    }

    pub fn get(&self, level: usize) -> Record {
        if level >= 1 && level <= MAX_LEVELS {
            self.entries[level - 1]
        } else {
            Record::default()
        }
    }

    /// Save a completed run and report which records were broken.
    pub fn submit(&mut self, level: usize, time: f32, score: i32) -> Improvement {
        if level < 1 || level > MAX_LEVELS {
            return Improvement::default();
        }
        let entry = &mut self.entries[level - 1];
        let faster = entry.best_time.is_none_or(|best| time < best);
        let higher = score > entry.best_score;
        if faster {
            entry.best_time = Some(time);
        }
        if higher {
            entry.best_score = score;
        }
        let _ = self.save();
        Improvement {
            time: faster,
            score: higher,
        }
    }

    fn save(&self) -> std::io::Result<()> {
        let mut text = String::new();
        for (i, entry) in self.entries.iter().enumerate() {
            if let Some(time) = entry.best_time {
                text.push_str(&format!("{} {:.3} {}\n", i + 1, time, entry.best_score));
            }
        }
        fs::write(&self.path, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(name);
        let _ = fs::remove_file(&path);
        path
    }

    #[test]
    fn records_keep_fastest_time_and_highest_score() {
        let path = temp_path("zpg-records-best.txt");
        let mut records = Records::from_path(path.clone());
        let first = records.submit(1, 90.0, 2000);
        assert!(first.time && first.score);

        let worse = records.submit(1, 120.0, 1500);
        assert!(!worse.time && !worse.score);

        let better_score = records.submit(1, 95.0, 2600);
        assert!(!better_score.time && better_score.score);

        let reloaded = Records::from_path(path);
        assert_eq!(reloaded.get(1).best_time, Some(90.0));
        assert_eq!(reloaded.get(1).best_score, 2600);
    }

    #[test]
    fn records_are_per_level() {
        let path = temp_path("zpg-records-levels.txt");
        let mut records = Records::from_path(path);
        records.submit(2, 60.0, 3000);
        assert_eq!(records.get(1).best_time, None);
        assert_eq!(records.get(2).best_time, Some(60.0));
        assert_eq!(records.get(3).best_score, 0);
    }
}
