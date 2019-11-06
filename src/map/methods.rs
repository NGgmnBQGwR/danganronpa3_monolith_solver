use std::str::FromStr;

use super::{monolith_solver, MonolithMap, Tile};

#[derive(Debug, Clone)]
pub enum SolvingMethods {
    Method1,
    Method2,
    Method3,
    Method4,
    Method5,
    Method6,
    Method7,
    Method8,
    Method9,
    Method10,
}

impl SolvingMethods {
    pub fn default() -> SolvingMethods {
        SolvingMethods::Method4
    }

    pub fn solve(self, map: MonolithMap) -> Vec<Tile> {
        match self {
            SolvingMethods::Method1 => monolith_solver::solve_1(map),
            SolvingMethods::Method2 => monolith_solver::solve_2(map),
            SolvingMethods::Method3 => monolith_solver::solve_3(map),
            SolvingMethods::Method4 => monolith_solver::solve_4(map),
            SolvingMethods::Method5 => monolith_solver::solve_5(map),
            SolvingMethods::Method6 => monolith_solver::solve_6(map),
            SolvingMethods::Method7 => monolith_solver::solve_7(map),
            SolvingMethods::Method8 => monolith_solver::solve_8(map),
            SolvingMethods::Method9 => monolith_solver::solve_9(map),
            SolvingMethods::Method10 => monolith_solver::solve_10(map),
        }
    }
}

impl FromStr for SolvingMethods {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Method1" => Ok(Self::Method1),
            "Method2" => Ok(Self::Method2),
            "Method3" => Ok(Self::Method3),
            "Method4" => Ok(Self::Method4),
            "Method5" => Ok(Self::Method5),
            "Method6" => Ok(Self::Method6),
            "Method7" => Ok(Self::Method7),
            "Method8" => Ok(Self::Method8),
            "Method9" => Ok(Self::Method9),
            "Method10" => Ok(Self::Method10),
            _ => Err(format!("Unknown solving method '{}'", s)),
        }
    }
}
