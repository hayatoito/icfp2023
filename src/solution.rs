use crate::prelude::*;
use crate::problem::ProblemId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Solution {
    pub placements: Vec<Point>,
    pub volumes: Vec<Score>,
}

impl Solution {
    #[cfg(test)]
    pub fn example() -> Result<Solution> {
        let s = read_from("problem/example/example-solution.json")?;
        let solution: Solution = serde_json::from_str(&s).unwrap();
        Ok(solution)
    }

    pub fn submission(id: ProblemId) -> Result<Solution> {
        let s = read_from(&format!("solution/submission/{}.json", id))?;
        let solution: Solution = serde_json::from_str(&s).unwrap();
        Ok(solution)
    }

    pub fn best(id: ProblemId) -> Result<Solution> {
        let s = read_from(&format!("solution/best/{}.json", id))?;
        let solution: Solution = serde_json::from_str(&s).unwrap();
        Ok(solution)
    }

    pub fn from(path: impl AsRef<Path>) -> Result<Solution> {
        let s = std::fs::read_to_string(path)?;
        let solution: Solution = serde_json::from_str(&s).unwrap();
        Ok(solution)
    }
}
