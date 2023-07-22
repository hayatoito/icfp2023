use rand::rngs::StdRng;

use crate::prelude::*;

use crate::draw;
use crate::problem::*;
use crate::solution::*;
use crate::solver::*;
use rand::Rng;

// https://gitlab.com/rafaelbocquet-cpcontests/icfpc23/-/blob/main/cxx/solve.cpp

#[derive(Default)]
struct AttNode {
    angle: f64,
    // Attendee's index.
    index: usize,
    nblock: u32,
    x: Coord,
    y: Coord,
}

struct LocalState<'a> {
    problem_id: ProblemId,
    problem: &'a Problem,
    spec: Spec,
    place: Vec<Point>,
    q: Vec<Score>,
    angles: Vec<Vec<AttNode>>,
    scores: Vec<Score>,
    score: Score,
}

// problem.hpp

fn norm_angle(mut angle: f64) -> f64 {
    while angle < 0.0 {
        angle += 2.0 * std::f64::consts::PI;
    }
    while angle > 2.0 * std::f64::consts::PI {
        angle -= 2.0 * std::f64::consts::PI;
    }
    angle
}

// Score doesn't match judge's one because volumes or q are muliplied at last.
pub fn score(problem: &Problem, problem_id: ProblemId, spec: Spec, solution: &Solution) -> Score {
    let st = LocalState::new(problem, problem_id, spec, &solution);
    st.score
}

impl<'a> LocalState<'a> {
    fn new(problem: &'a Problem, problem_id: ProblemId, spec: Spec, solution: &Solution) -> Self {
        let nm = solution.placements.len();
        let natt = problem.attendees.len();

        let place = solution.placements.clone();

        let angles = (0..nm)
            .map(|_| {
                (0..natt)
                    .map(|j| AttNode {
                        angle: 0.0,
                        index: j,
                        nblock: 0,
                        x: problem.attendees[j].x,
                        y: problem.attendees[j].y,
                    })
                    .collect()
            })
            .collect();

        let mut state = LocalState {
            problem_id,
            problem,
            spec,
            place,
            q: vec![1.0; nm],
            angles,
            scores: vec![0.0; nm],
            score: 0.0,
        };

        state.cal_q();

        for i in 0..nm {
            state.make_angles(i);
        }

        state.update_score(Some(&solution.volumes));
        // state.update_score(None);
        state
    }

    fn update_score(&mut self, volumes: Option<&[f64]>) {
        self.score = if let Some(volumes) = volumes {
            (0..self.scores.len())
                .map(|i| {
                    // self.q[i] * self.scores[i] * volumes[i]
                    let a = self.q[i] * self.scores[i];
                    // // println!("m: {i}: score: {a}, volume: {}", volumes[i]);
                    a * volumes[i]
                })
                .sum()
        } else {
            (0..self.scores.len())
                .map(|i| (self.q[i] * self.scores[i]).max(0.0) * 10.0)
                .sum()
        };
    }

    fn cal_q(&mut self) {
        if self.is_full_round() {
            for i in 0..self.q.len() {
                for j in 0..self.q.len() {
                    if i != j && self.problem.musicians[i] == self.problem.musicians[j] {
                        self.q[i] += 1.0 / self.place[i].distance(self.place[j]);
                    }
                }
            }
        }
    }

    fn to_solution(&self) -> Solution {
        let volumes = self
            .scores
            .iter()
            .map(|score| if *score <= 1e-6 { 0.0 } else { 10.0 })
            .collect();
        Solution {
            placements: self.place.clone(),
            volumes,
        }
    }

    fn report_progress(&self) -> Result<()> {
        // Save svg for reporting.
        let solution = self.to_solution();
        draw::draw_solution(
            self.problem_id,
            &solution,
            project_path(format!("draw/wip/{}.svg", self.problem_id)),
        )
    }

    fn is_full_round(&self) -> bool {
        matches!(self.spec, Spec::V2)
    }

    fn make_angles(&mut self, i: usize) {
        for j in 0..self.problem.attendees.len() {
            let a = &mut self.angles[i][j];
            let x = a.x - self.place[i].x;
            let y = a.y - self.place[i].y;
            a.angle = norm_angle(y.atan2(x));
            a.nblock = 0;
        }
        // Sort attendeees by angle from musician i to attendee.
        self.angles[i].sort_by_key(|a| OrderedFloat(a.angle));

        self.scores[i] = 0.0;
        for j in 0..self.problem.attendees.len() {
            let d2 = self.place[i].distance_squared(self.problem.attendees[j].point());
            self.scores[i] +=
                1e6 * self.problem.attendees[j].tastes[self.problem.musicians[i]] / d2;
        }

        for j in 0..self.place.len() {
            if i != j {
                self.add_blocks(i, j);
            }
        }

        if self.is_full_round() {
            self.add_pillars(i);
        }
    }

    fn add_blocks(&mut self, i: usize, j: usize) {
        let [r1, r2] = self.blocks_range(i, j);
        for k in r1.into_iter().chain(r2.into_iter()) {
            if self.angles[i][k].nblock == 0 {
                let d2 = self.place[i]
                    .distance_squared(self.problem.attendees[self.angles[i][k].index].point());
                self.scores[i] -= 1e6
                    * self.problem.attendees[self.angles[i][k].index].tastes
                        [self.problem.musicians[i]]
                    / d2;
            }
            self.angles[i][k].nblock += 1;
        }
    }

    fn rem_blocks(&mut self, i: usize, j: usize) {
        let [r1, r2] = self.blocks_range(i, j);
        for k in r1.into_iter().chain(r2.into_iter()) {
            self.angles[i][k].nblock -= 1;
            if self.angles[i][k].nblock == 0 {
                let d2 = self.place[i]
                    .distance_squared(self.problem.attendees[self.angles[i][k].index].point());
                self.scores[i] += 1e6
                    * self.problem.attendees[self.angles[i][k].index].tastes
                        [self.problem.musicians[i]]
                    / d2;
            }
        }
    }

    fn add_pillars(&mut self, i: usize) {
        for j in 0..self.problem.pillars.len() {
            let dp2 = self.place[i].distance_squared(self.problem.pillars[j].center_point());
            let [r1, r2] = self.pillars_range(i, j);
            for k in r1.into_iter().chain(r2.into_iter()) {
                let d2 = self.place[i]
                    .distance_squared(self.problem.attendees[self.angles[i][k].index].point());
                if d2 > dp2 {
                    if self.angles[i][k].nblock == 0 {
                        self.scores[i] -= 1e6
                            * self.problem.attendees[self.angles[i][k].index].tastes
                                [self.problem.musicians[i]]
                            / d2;
                    }
                    self.angles[i][k].nblock += 1;
                }
            }
        }
    }

    fn find_index(&self, angles: &[AttNode], angle: f64) -> usize {
        let mut left = 0;
        let mut right = angles.len();

        while left < right {
            let mid = left + (right - left) / 2;
            if angles[mid].angle < angle {
                left = mid + 1;
            } else {
                right = mid
            }
        }
        left
    }

    fn blocks_range(&self, i: usize, j: usize) -> [Range<usize>; 2] {
        let d = self.place[i].distance(self.place[j]);
        let angle = (self.place[j].y - self.place[i].y).atan2(self.place[j].x - self.place[i].x);
        let alpha = (BLOCK_RADIUS / d).asin();
        let angle0 = norm_angle(angle - alpha);
        let angle1 = norm_angle(angle + alpha);
        let ix0 = self.find_index(&self.angles[i], angle0);
        let ix1 = self.find_index(&self.angles[i], angle1);
        if angle0 < angle1 {
            assert!(ix0 <= ix1);
            [(ix0..ix1), 0..0]
        } else {
            assert!(ix1 <= ix0);
            [(ix0..self.problem.attendees.len()), (0..ix1)]
        }
    }

    fn pillars_range(&self, i: usize, j: usize) -> [Range<usize>; 2] {
        let center = self.problem.pillars[j].center_point();
        let radius = self.problem.pillars[j].radius;

        let d = self.place[i].distance(center);
        let angle = (center.y - self.place[i].y).atan2(center.x - self.place[i].x);
        let alpha = (radius / d).asin();
        let angle0 = norm_angle(angle - alpha);
        let angle1 = norm_angle(angle + alpha);
        let ix0 = self.find_index(&self.angles[i], angle0);
        let ix1 = self.find_index(&self.angles[i], angle1);
        if angle0 < angle1 {
            [(ix0..ix1), 0..0]
        } else {
            [(ix0..self.problem.attendees.len()), (0..ix1)]
        }
    }

    fn do_move(&mut self, i: usize, to: Point) {
        let nm = self.place.len();

        for j in 0..nm {
            if i != j {
                self.rem_blocks(j, i);
            }
        }

        if self.is_full_round() {
            for j in 0..nm {
                if i != j && self.problem.musicians[i] == self.problem.musicians[j] {
                    self.q[j] -= 1.0 / self.place[i].distance(self.place[j]);
                }
            }
        }

        self.place[i] = to;

        self.q[i] = 1.0;
        if self.is_full_round() {
            for j in 0..nm {
                if i != j && self.problem.musicians[i] == self.problem.musicians[j] {
                    let qplus = 1.0 / self.place[i].distance(self.place[j]);
                    self.q[i] += qplus;
                    self.q[j] += qplus;
                }
            }
        }

        self.make_angles(i);

        for j in 0..nm {
            if i != j {
                self.add_blocks(j, i);
            }
        }

        self.update_score(None);
    }

    fn do_swap(&mut self, a: usize, b: usize) {
        let nm = self.place.len();

        if self.is_full_round() {
            for i in [a, b] {
                for j in 0..nm {
                    if i != j && self.problem.musicians[i] == self.problem.musicians[j] {
                        self.q[j] -= 1.0 / self.place[i].distance(self.place[j]);
                    }
                }
            }
        }

        self.place.swap(a, b);
        self.make_angles(a);
        self.make_angles(b);

        if self.is_full_round() {
            self.q[a] = 1.0;
            self.q[b] = 1.0;
            for i in [a, b] {
                for j in 0..nm {
                    if i != j && self.problem.musicians[i] == self.problem.musicians[j] {
                        let qplus = 1.0 / self.place[i].distance(self.place[j]);
                        self.q[i] += qplus;
                        self.q[j] += qplus;
                    }
                }
            }
            // Consider double count a <=> b
            if self.problem.musicians[a] == self.problem.musicians[b] {
                let qplus = 1.0 / self.place[a].distance(self.place[b]);
                self.q[a] -= qplus;
                self.q[b] -= qplus;
            }
        }
        self.update_score(None);
    }

    #[allow(dead_code)]
    fn assert_score(&self) {
        let solution = self.to_solution();
        let new_state = LocalState::new(self.problem, self.problem_id, self.spec, &solution);
        assert_relative_eq!(self.score, new_state.score, max_relative = 1.0);
    }
}

#[derive(Copy, Clone, derive_more::Display)]
pub enum End {
    #[display(fmt = "iter-{}", "_0.to_string()")]
    MaxIteration(usize),
    #[display(fmt = "duration-{}", "_0.as_secs()")]
    MaxDuration(std::time::Duration),
}

pub fn run_sa(
    name: &str,
    rng: &mut StdRng,
    problem: &Problem,
    problem_id: ProblemId,
    spec: Spec,
    solution: &Solution,
    temp0: Option<f64>,
    end: End,
) -> Result<(Score, Solution)> {
    let nm = problem.musicians.len();

    let mut st = LocalState::new(problem, problem_id, spec, solution);

    let mut sc = st.score;

    let timer = std::time::Instant::now();

    let temp0 = temp0.unwrap_or_else(|| sc.abs() / (problem.musicians.len() as f64).sqrt());
    let mut temp = temp0;

    let mut best = sc;
    // let mut best_solution = solution.clone();
    let mut best_solution = st.to_solution();

    let mut naccept_positive = 0;
    let mut naccept_negative = 0;
    let mut ntotal = 0;

    let mut ncollide = 0;
    let mut nmove = 0;
    let sa_plot = project_path(format!("stats/sa/{}/{}.data", name, problem_id));
    std::fs::create_dir_all(sa_plot.parent().unwrap())?;

    let mut sa_plot = std::fs::File::create(sa_plot)?;
    writeln!(
        sa_plot,
        "iteration score best temperature acceptrate acceptrate_positive acceptrate_negative"
    )?;

    let mut niter = 0;

    loop {
        niter += 1;

        if niter % 1_000 == 0 {
            let done = match end {
                End::MaxIteration(max_iter) => niter as f64 / max_iter as f64,
                End::MaxDuration(max_duration) => {
                    timer.elapsed().as_millis() as f64 / max_duration.as_millis() as f64
                }
            };
            temp = temp0 * (1.0 - done);

            if temp < 0.0 {
                return Ok((best, best_solution));
            }
        }

        if niter % 10_000 == 0 {
            let accept_rate =
                (naccept_positive + naccept_negative) as f64 / 1.0f64.max(ntotal as f64);
            let accept_rate_positive = naccept_positive as f64 / 1.0f64.max(ntotal as f64);
            let accept_rate_negative = naccept_negative as f64 / 1.0f64.max(ntotal as f64);
            info!("temp: {temp:.1}, niter: {niter}, sc: {sc:.1}, best: {best:.1}, ncollide: {ncollide}, nmove: {nmove}, ntotal: {ntotal}, naccept_positive: {naccept_positive}, naccept_negative: {naccept_negative}, accept_rate: {accept_rate:.02}");

            writeln!(
                sa_plot,
                "{} {:.1} {:.1} {:.1} {:.3} {:.3} {:.3}",
                niter, sc, best, temp, accept_rate, accept_rate_positive, accept_rate_negative
            )?;
            naccept_positive = 0;
            naccept_negative = 0;
            ntotal = 0;
        }

        if niter % 100_000 == 0 {
            st.report_progress().unwrap();

            let solution = st.to_solution();
            st = LocalState::new(problem, problem_id, spec, &solution);
        }

        // Swap
        if rng.gen_range(0..10) == 0 {
            let a = rng.gen_range(0..nm);
            let b = rng.gen_range(0..nm);
            if a == b {
                continue;
            }
            st.do_swap(a, b);

            let sc2 = st.score;
            // let delta = sc2 - sc;

            ntotal += 1;
            if sc2 >= sc || ((sc2 - sc) / temp).exp() > rng.gen_range(0.0..1.0) {
                if sc2 >= sc {
                    naccept_positive += 1;
                } else {
                    naccept_negative += 1;
                }
                sc = sc2;
                if sc > best {
                    best = sc;
                    best_solution = st.to_solution();
                }
            } else {
                st.do_swap(a, b);
            }
            continue;
        }

        // Move
        let id = rng.gen_range(0..nm);
        let p0 = st.place[id];

        let collides = |p: Point| -> bool {
            !st.problem.on_stage(p)
                || (0..nm)
                    .any(|j| id != j && p.distance_squared(st.place[j]) < MUSICIAN_RADIUS_2 + EPS)
        };

        let p = match rng.gen_range(0..10) {
            0 => problem.random_point_on_stage(rng),
            1 => {
                let dist = 40.0 * rng.gen_range(0.0f64..1.0).powi(2);
                let angle = rng.gen_range(0.0f64..2.0 * std::f64::consts::PI);

                let dx = angle.cos();
                let dy = angle.sin();

                let mut lo = 0.0;
                let mut hi = dist;
                while hi - lo > 1e-3 {
                    let mi = (lo + hi) / 2.0;
                    if collides(Point::new(p0.x + mi * dx, p0.y + mi + dy)) {
                        hi = mi;
                    } else {
                        lo = mi;
                    }
                }
                Point::new(p0.x + lo * dx, p0.y + lo * dy)
            }

            // }
            // 2 => {
            //   // TODO: Gradient
            // };
            _ => {
                let dist = 40.0 * rng.gen_range(0.0f64..1.0).powi(2);
                let angle = rng.gen_range(0.0f64..2.0 * std::f64::consts::PI);
                Point::new(p0.x + dist * angle.cos(), p0.y + dist * angle.sin())
            }
        };

        if collides(p) {
            ncollide += 1;
            continue;
        }
        nmove += 1;
        st.do_move(id, p);

        let sc2 = st.score;
        ntotal += 1;

        if sc2 >= sc || ((sc2 - sc) / temp).exp() > rng.gen_range(0.0..1.0) {
            if sc2 >= sc {
                naccept_positive += 1;
            } else {
                naccept_negative += 1;
            }
            sc = sc2;
            if sc > best {
                best = sc;
                best_solution = st.to_solution();
            }
        } else {
            st.do_move(id, p0);
        }
    }
}

pub struct SolverSa {
    problem_id: ProblemId,
    problem: Problem,
    temp0: Option<f64>,
    end: End,
    initial_solution: Solution,
}

impl SolverSa {
    fn initial_solution(problem: &Problem) -> Solution {
        let mut rng = SeedableRng::from_seed([0; 32]);
        let mut placements = vec![];

        while placements.len() < problem.musicians.len() {
            let p = problem.random_point_on_stage(&mut rng);
            if placements
                .iter()
                .all(|q| p.distance_squared(*q) > MUSICIAN_RADIUS_2 + EPS)
            {
                placements.push(p);
            }
        }
        Solution {
            placements,
            volumes: vec![10.0; problem.musicians.len()],
        }
    }

    pub fn new(
        problem_id: ProblemId,
        temp0: Option<f64>,
        end: End,
        initial_solution: Option<Solution>,
    ) -> Result<Self> {
        let problem = Problem::new(problem_id)?;
        let initial_solution = initial_solution.unwrap_or(Self::initial_solution(&problem));
        Ok(Self {
            problem_id,
            problem,
            temp0,
            end,
            initial_solution,
        })
    }
}

impl Solver for SolverSa {
    fn problem_id(&self) -> ProblemId {
        self.problem_id
    }

    fn name(&self) -> String {
        format!("sa-temp0-{:.0}-{}", self.temp0.unwrap_or(0.0), self.end)
    }

    fn solve(&mut self) -> Result<Solved> {
        let mut rng = SeedableRng::from_seed([0; 32]);

        let (score, solution) = run_sa(
            &self.name(),
            &mut rng,
            &self.problem,
            self.problem_id,
            self.problem_id.into(),
            &self.initial_solution,
            self.temp0,
            self.end,
        )?;
        let Solution {
            placements,
            volumes,
        } = solution;
        Ok(Solved {
            problem_id: self.problem_id,
            solver_name: self.name(),
            score,
            placements,
            volumes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn solver_sa() -> Result<()> {
        let cases = [(60, 34597619.50674734)];
        for (id, score) in cases {
            let mut solver = SolverSa::new(id, Some(100.0), End::MaxIteration(10_000), None)?;
            let solved = solver.solve()?;
            assert_eq!(solved.score, score);
        }
        Ok(())
    }

    #[test]
    fn sa_score_example_problem() -> Result<()> {
        let problem = Problem::example()?;
        let solution = Solution::example()?;
        assert_relative_eq!(
            score(
                &problem,
                // dummy
                0,
                Spec::V1,
                &solution,
            ),
            5343.0,
            max_relative = 1.0
        );
        assert_relative_eq!(
            score(
                &problem,
                // dummy
                0,
                Spec::V2,
                &solution,
            ),
            3270.0,
            max_relative = 1.0
        );
        Ok(())
    }
}
