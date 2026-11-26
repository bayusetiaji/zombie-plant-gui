use crate::game::MAX_LEVELS;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Record {
    pub best_time: Option<f32>,
    pub best_score: i32,
    pub stars: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Improvement {
    pub time: bool,
    pub score: bool,
    pub stars: bool,
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
                // Older save files have no star field; treat it as zero.
                let stars = parts.next().and_then(|p| p.parse::<u8>().ok()).unwrap_or(0);
                entries[level - 1] = Record {
                    best_time: Some(time),
                    best_score: score,
                    stars,
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
    pub fn submit(&mut self, level: usize, time: f32, score: i32, stars: u8) -> Improvement {
        if level < 1 || level > MAX_LEVELS {
            return Improvement::default();
        }
        let entry = &mut self.entries[level - 1];
        let faster = entry.best_time.is_none_or(|best| time < best);
        let higher = score > entry.best_score;
        let more_stars = stars > entry.stars;
        if faster {
            entry.best_time = Some(time);
        }
        if higher {
            entry.best_score = score;
        }
        if more_stars {
            entry.stars = stars;
        }
        let _ = self.save();
        Improvement {
            time: faster,
            score: higher,
            stars: more_stars,
        }
    }

    fn save(&self) -> std::io::Result<()> {
        let mut text = String::new();
        for (i, entry) in self.entries.iter().enumerate() {
            if let Some(time) = entry.best_time {
                text.push_str(&format!(
                    "{} {:.3} {} {}\n",
                    i + 1,
                    time,
                    entry.best_score,
                    entry.stars
                ));
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
        let first = records.submit(1, 90.0, 2000, 2);
        assert!(first.time && first.score && first.stars);

        let worse = records.submit(1, 120.0, 1500, 1);
        assert!(!worse.time && !worse.score && !worse.stars);

        let better_score = records.submit(1, 95.0, 2600, 3);
        assert!(!better_score.time && better_score.score && better_score.stars);

        let reloaded = Records::from_path(path);
        assert_eq!(reloaded.get(1).best_time, Some(90.0));
        assert_eq!(reloaded.get(1).best_score, 2600);
        assert_eq!(reloaded.get(1).stars, 3);
    }

    #[test]
    fn records_are_per_level() {
        let path = temp_path("zpg-records-levels.txt");
        let mut records = Records::from_path(path);
        records.submit(2, 60.0, 3000, 3);
        assert_eq!(records.get(1).best_time, None);
        assert_eq!(records.get(2).best_time, Some(60.0));
        assert_eq!(records.get(3).best_score, 0);
    }

    #[test]
    fn old_save_files_without_stars_still_load() {
        let path = temp_path("zpg-records-legacy.txt");
        fs::write(&path, "1 60.000 1000\n").unwrap();
        let records = Records::from_path(path);
        assert_eq!(records.get(1).best_time, Some(60.0));
        assert_eq!(records.get(1).best_score, 1000);
        assert_eq!(records.get(1).stars, 0);
    }
}
