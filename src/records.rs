use crate::game::MAX_LEVELS;
use std::fs;
use std::path::PathBuf;

/// Best run for one endless difficulty.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Record {
    pub best_wave: usize,
    pub longest_time: f32,
    pub best_score: i32,
    pub stars: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Improvement {
    pub wave: bool,
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
                let parts: Vec<&str> = line.split_whitespace().collect();
                let Some(level) = parts.first().and_then(|p| p.parse::<usize>().ok()) else {
                    continue;
                };
                if level == 0 || level > MAX_LEVELS {
                    continue;
                }
                if parts.len() >= 5 {
                    let Some(wave) = parts[1].parse::<usize>().ok() else {
                        continue;
                    };
                    let Some(time) = parts[2].parse::<f32>().ok() else {
                        continue;
                    };
                    let Some(score) = parts[3].parse::<i32>().ok() else {
                        continue;
                    };
                    let stars = parts[4].parse::<u8>().unwrap_or(0);
                    entries[level - 1] = Record {
                        best_wave: wave,
                        longest_time: time,
                        best_score: score,
                        stars,
                    };
                } else if parts.len() >= 3 {
                    // Legacy finite-level file: `level time score [stars]`.
                    // Treat a finished level as having reached its unlock
                    // wave so old save files keep their progression.
                    let Some(time) = parts[1].parse::<f32>().ok() else {
                        continue;
                    };
                    let Some(score) = parts[2].parse::<i32>().ok() else {
                        continue;
                    };
                    let stars = parts.get(3).and_then(|p| p.parse::<u8>().ok()).unwrap_or(0);
                    let wave = match level {
                        1 => 10,
                        2 => 15,
                        _ => 20,
                    };
                    entries[level - 1] = Record {
                        best_wave: wave,
                        longest_time: time,
                        best_score: score,
                        stars,
                    };
                }
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

    /// Save a finished endless run and report which records were broken.
    /// Longer survival time and higher score are both improvements.
    pub fn submit(
        &mut self,
        level: usize,
        wave: usize,
        time: f32,
        score: i32,
        stars: u8,
    ) -> Improvement {
        if level < 1 || level > MAX_LEVELS {
            return Improvement::default();
        }
        let entry = &mut self.entries[level - 1];
        let better_wave = wave > entry.best_wave;
        let longer_time = time > entry.longest_time;
        let higher_score = score > entry.best_score;
        let more_stars = stars > entry.stars;
        if better_wave {
            entry.best_wave = wave;
        }
        if longer_time {
            entry.longest_time = time;
        }
        if higher_score {
            entry.best_score = score;
        }
        if more_stars {
            entry.stars = stars;
        }
        let _ = self.save();
        Improvement {
            wave: better_wave,
            time: longer_time,
            score: higher_score,
            stars: more_stars,
        }
    }

    fn save(&self) -> std::io::Result<()> {
        let mut text = String::new();
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.best_wave > 0 {
                text.push_str(&format!(
                    "{} {} {:.3} {} {}\n",
                    i + 1,
                    entry.best_wave,
                    entry.longest_time,
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
    fn records_keep_best_wave_longest_time_and_score() {
        let path = temp_path("zpg-records-best.txt");
        let mut records = Records::from_path(path.clone());
        let first = records.submit(1, 7, 90.0, 2000, 2);
        assert!(first.wave && first.time && first.score && first.stars);

        let worse = records.submit(1, 5, 60.0, 1500, 1);
        assert!(!worse.wave && !worse.time && !worse.score && !worse.stars);

        let mixed = records.submit(1, 9, 80.0, 2600, 3);
        assert!(mixed.wave && !mixed.time && mixed.score && mixed.stars);

        let reloaded = Records::from_path(path);
        assert_eq!(reloaded.get(1).best_wave, 9);
        assert_eq!(reloaded.get(1).longest_time, 90.0);
        assert_eq!(reloaded.get(1).best_score, 2600);
        assert_eq!(reloaded.get(1).stars, 3);
    }

    #[test]
    fn records_are_per_difficulty() {
        let path = temp_path("zpg-records-levels.txt");
        let mut records = Records::from_path(path);
        records.submit(2, 12, 60.0, 3000, 3);
        assert_eq!(records.get(1).best_wave, 0);
        assert_eq!(records.get(2).best_wave, 12);
        assert_eq!(records.get(3).best_score, 0);
    }

    #[test]
    fn old_save_files_migrate_to_wave_records() {
        let path = temp_path("zpg-records-legacy.txt");
        fs::write(&path, "1 60.000 1000 2\n").unwrap();
        let records = Records::from_path(path);
        assert_eq!(records.get(1).best_wave, 10);
        assert_eq!(records.get(1).longest_time, 60.0);
        assert_eq!(records.get(1).best_score, 1000);
        assert_eq!(records.get(1).stars, 2);
    }
}
