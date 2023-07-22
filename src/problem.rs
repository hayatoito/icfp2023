use crate::prelude::*;

pub type ProblemId = u64;

pub const MAX_PROMLEM_ID: ProblemId = 90;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Spec {
    V1,
    V2,
}

const V1_PROBLEM_ID_END: ProblemId = 55;
const V2_PROBLEM_ID_START: ProblemId = V1_PROBLEM_ID_END + 1;

impl From<ProblemId> for Spec {
    fn from(id: ProblemId) -> Self {
        match id {
            0..=V1_PROBLEM_ID_END => Spec::V1,
            V2_PROBLEM_ID_START.. => Spec::V2,
        }
    }
}

// #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Problem {
    pub room_width: Coord,
    pub room_height: Coord,
    pub stage_width: Coord,
    pub stage_height: Coord,
    pub stage_bottom_left: [Coord; 2],
    pub musicians: Vec<Instrument>,
    pub attendees: Vec<Attendee>,
    // Spec v2
    pub pillars: Vec<Pillar>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attendee {
    pub x: Coord,
    pub y: Coord,
    pub tastes: Vec<Score>,
}

impl Attendee {
    pub fn point(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn taste_avg(&self) -> Score {
        self.tastes.iter().sum::<Score>() / (self.tastes.len() as Score)
    }

    pub fn taste_max(&self) -> Score {
        self.tastes
            .iter()
            .map(|i| ordered_float::OrderedFloat(*i))
            .max()
            .unwrap()
            .0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pillar {
    pub center: [Coord; 2],
    pub radius: Coord,
}

impl Pillar {
    pub fn center_point(&self) -> Point {
        Point::new(self.center[0], self.center[1])
    }
}

impl Problem {
    pub fn new(id: ProblemId) -> Result<Problem> {
        let s = read_from(format!("problem/{}.json", id))?;
        let problem: Problem = serde_json::from_str(&s).unwrap();
        debug!("problem: {problem:?}");
        Ok(problem)
    }

    pub fn example() -> Result<Problem> {
        let s = read_from("problem/example/example-problem.json")?;
        let problem: Problem = serde_json::from_str(&s).unwrap();
        debug!("problem: {problem:?}");
        Ok(problem)
    }

    pub fn on_stage(&self, p: Point) -> bool {
        p.x >= self.stage_bottom_left[0] + MUSICIAN_RADIUS
            && p.x <= self.stage_bottom_left[0] + self.stage_width - MUSICIAN_RADIUS
            && p.y >= self.stage_bottom_left[1] + MUSICIAN_RADIUS
            && p.y <= self.stage_bottom_left[1] + self.stage_height - MUSICIAN_RADIUS
    }

    pub fn random_point_on_stage(&self, rng: &mut StdRng) -> Point {
        let (minx, maxx) = (
            self.stage_bottom_left[0] + MUSICIAN_RADIUS,
            self.stage_bottom_left[0] + self.stage_width - MUSICIAN_RADIUS,
        );

        let (miny, maxy) = (
            self.stage_bottom_left[1] + MUSICIAN_RADIUS,
            self.stage_bottom_left[1] + self.stage_height - MUSICIAN_RADIUS,
        );

        Point::new(
            if minx == maxx {
                minx
            } else {
                rng.gen_range(minx..maxx)
            },
            if miny == maxy {
                miny
            } else {
                rng.gen_range(miny..maxy)
            },
        )
    }

    pub fn taste_avg(&self) -> Score {
        self.attendees.iter().map(|a| a.taste_avg()).sum::<Score>()
            / (self.attendees.len() as Score)
    }

    pub fn taste_max_avg(&self) -> Score {
        self.attendees.iter().map(|a| a.taste_max()).sum::<Score>()
            / (self.attendees.len() as Score)
    }

    pub fn stage_center(&self) -> Point {
        Point::new(
            self.stage_bottom_left[0] + self.stage_width / 2.0,
            self.stage_bottom_left[1] + self.stage_height / 2.0,
        )
    }

    fn distance_to_stage_squared(&self, p: Point) -> Coord {
        let minx = self.stage_bottom_left[0];
        let maxx = self.stage_bottom_left[0] + self.stage_width;

        let miny = self.stage_bottom_left[1];
        let maxy = self.stage_bottom_left[1] + self.stage_height;

        let bottom_left = Point::new(minx, miny);
        let bottom_right = Point::new(maxx, miny);

        let top_right = Point::new(maxx, maxy);
        let top_left = Point::new(minx, maxy);

        let lines = [
            (bottom_left, bottom_right),
            (bottom_right, top_right),
            (top_right, top_left),
            (top_left, bottom_left),
        ];

        lines
            .into_iter()
            .map(|line| OrderedFloat(point_to_segment_distance_squared(p, line)))
            .min()
            .unwrap()
            .0
    }

    pub fn tentative_score(&self) -> Score {
        let inst_cnt = inst_cnt(&self.musicians);
        inst_cnt
            .into_iter()
            .map(|(inst, cnt)| {
                let impact = self
                    .attendees
                    .iter()
                    .map(|a| {
                        let d2 = self.distance_to_stage_squared(a.point());
                        a.tastes[inst as usize] / d2
                    })
                    .sum::<Score>();
                if impact > 0.0 {
                    impact * (cnt as Score)
                } else {
                    0.0
                }
            })
            .sum::<Score>()
            // impact mulplier
            * 1_000_000.0
            // Volume
            * 10.0
    }
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "musicians.len(): {}, attendees.len(): {}",
            self.musicians.len(),
            self.attendees.len()
        )
    }
}

pub fn dump_problem(problem_id: ProblemId) -> Result<()> {
    let problem = Problem::new(problem_id)?;
    println!("id: {problem_id}, {}", problem);
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn read_problem() -> Result<()> {
        let problem = Problem::new(1)?;
        assert_eq!(problem.room_width, 3649.0);
        assert_eq!(problem.musicians.len(), 1059);

        let problem = Problem::new(42)?;
        assert_eq!(problem.room_width, 1000.0);
        assert_eq!(problem.musicians.len(), 5);
        Ok(())
    }
}
