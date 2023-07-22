use crate::prelude::*;
use crate::problem::*;
use crate::solution::*;

use fd_lock::RwLock;

// - problem: {id}.json
// - solution
//   - all: {id}-{score}.json
//   - best: {id}.json
//   - submission: {id}.json
// - stats
//   - userboard.json
//   - For gnuplot:
//   - stats-problem.data
//   - stats-score.data

// UserBoard should be used only when the contest is open.

#[derive(Serialize, Deserialize, Debug)]
pub struct Userboard {
    #[serde(rename = "Success")]
    pub success: Problems,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Problems {
    pub problems: Vec<Option<Score>>,
}

impl Userboard {
    pub fn best_score(&self, id: ProblemId) -> Option<Score> {
        assert!(id > 0);
        let index = (id - 1) as usize;
        self.success.problems[index]
    }
}

impl Userboard {
    pub fn new() -> Result<Userboard> {
        Ok(serde_json::from_str(&read_from("stats/userboard.json")?)?)
    }

    pub fn total_score(&self) -> Score {
        self.success.problems.iter().flat_map(|a| a).sum()
    }

    #[allow(dead_code)]
    fn stats(&self) {
        println!("# total: {}", self.total_score());
        println!("# id score");
        for (id, score) in self.success.problems.iter().enumerate() {
            let id = (id + 1) as ProblemId;
            let score = score.unwrap_or(0.0);
            println!("{id} {score}",);
        }
    }
}

// After contests

#[derive(Serialize, Deserialize, Debug)]
pub struct BestScore(HashMap<ProblemId, Score>);

impl BestScore {
    pub fn new() -> Result<Self> {
        let f = RwLock::new(
            std::fs::File::open(project_path("stats/best-score.json")).context("best score new")?,
        );
        let f = f.read()?;
        Ok(Self(serde_json::from_reader(f.deref())?))
    }

    pub fn score(&self, id: ProblemId) -> Option<Score> {
        self.0.get(&id).cloned()
    }

    pub fn total_score(&self) -> Score {
        self.0.values().sum()
    }

    pub fn update(id: ProblemId, new_score: Score) -> Result<()> {
        use std::io::{Seek as _, SeekFrom};

        // This op should be atomic.

        let path = project_path("stats/best-score.json");
        let f = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .context("best score update")?;
        let mut f = RwLock::new(f);
        let mut f = f.write()?;

        let mut map: HashMap<ProblemId, Score> =
            serde_json::from_reader(f.deref()).context("best score update 2")?;
        map.insert(id, new_score);

        // Truncate
        f.set_len(0)?;
        f.seek(SeekFrom::Start(0))?;

        let json = serde_json::to_string(&map)?;
        write!(f, "{}", json)?;
        Ok(())
    }

    pub fn refresh() -> Result<()> {
        let mut map = HashMap::new();
        for id in 1..=90 {
            if let Ok(best_solution) = Solution::best(id) {
                let problem = Problem::new(id)?;
                let score = crate::solver_sa::score(&problem, id, id.into(), &best_solution);
                map.insert(id, score);
            }
        }

        let path = project_path("stats/best-score.json");
        let f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .context("best score refresh")?;
        let mut f = RwLock::new(f);
        let mut f = f.write()?;
        let json = serde_json::to_string(&map)?;
        write!(f, "{}", json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn read_userboard() -> Result<()> {
        let userboard = Userboard::new()?;
        // [2023-07-07 Fri 22:49] Lightning. # of Problems = 55
        // Full: Day1  # of Problems = 90
        assert_eq!(userboard.success.problems.len(), 90);
        Ok(())
    }
}
