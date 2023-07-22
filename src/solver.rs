use crate::draw;
use crate::prelude::*;

use crate::problem::*;
use crate::solution::*;
use crate::stats::*;

pub trait Solver {
    fn problem_id(&self) -> ProblemId;
    fn name(&self) -> String;
    fn solve(&mut self) -> Result<Solved>;
}

pub struct Solved {
    pub problem_id: ProblemId,
    pub solver_name: String,
    pub score: Score,
    pub placements: Vec<Point>,
    pub volumes: Vec<Score>,
}

impl Solved {
    fn save_solution(&self) -> Result<()> {
        self.save_solution_to(&format!(
            "solution/all/{}-{}-{}.json",
            self.problem_id, self.solver_name, self.score
        ))?;

        self.save_solution_to(&format!(
            "solution/{}/{}.json",
            self.solver_name, self.problem_id
        ))
    }

    fn save_solution_to(&self, name: &str) -> Result<()> {
        let json = serde_json::to_string(&self.solution())?;
        write_to(name, &json)
    }

    pub fn save_best_if(&self) -> Result<()> {
        let best_score = BestScore::new()?;
        let is_best = match best_score.score(self.problem_id) {
            Some(best) => {
                if best < self.score {
                    println!(
                        "💘 problem_id: {}, best: {best} < score: {}",
                        self.problem_id, self.score
                    );
                    true
                } else {
                    info!(
                        "😢 problem_id: {}, best: {best} >= score: {}",
                        self.problem_id, self.score,
                    );
                    false
                }
            }
            None => {
                println!(
                    "💘 problem_id: {}, best: None < score: {}",
                    self.problem_id, self.score
                );
                true
            }
        };
        if is_best {
            self.save_solution_to(&format!("solution/best/{}.json", self.problem_id))?;
            BestScore::update(self.problem_id, self.score)?;
        }
        Ok(())
    }

    fn solution(&self) -> Solution {
        Solution {
            placements: self.placements.clone(),
            volumes: self.volumes.clone(),
        }
    }

    pub fn plot(&self) -> Result<()> {
        draw::draw_solution(
            self.problem_id,
            &self.solution(),
            project_path(format!("draw/all/{}-{}.svg", self.problem_id, self.score)),
        )
    }
}

pub fn solve<T: Solver>(mut solver: T) -> Result<()> {
    println!("Solving... {}", solver.problem_id());
    let solved = solver.solve()?;
    println!("Solved {}. score: {}", solver.problem_id(), solved.score);
    solved.save_solution()?;
    solved.save_best_if()?;
    solved.plot()?;
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn step_by_test() {
        assert_eq!((0..50).step_by(10).collect::<Vec<_>>(), [0, 10, 20, 30, 40]);
    }

    #[test]
    fn clamp_test() {
        assert_eq!(3.clamp(3, 5), 3);
        assert_eq!(2.clamp(3, 5), 3);
        assert_eq!(8.clamp(3, 5), 5);
        // assert_eq!(8.clamp(10, 5), 5);
    }
}
