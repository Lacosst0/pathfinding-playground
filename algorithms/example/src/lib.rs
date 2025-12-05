use pathfinding::prelude::dijkstra;
use pathfinding::{grid::Grid, prelude::Matrix};

wit_bindgen::generate!({
    path: "../../wit",
    world: "pathfinding",
});
use crate::exports::guest;
use crate::host::*;

struct MyImpl;

impl guest::Guest for MyImpl {
    fn run(input: Vec<Vec<bool>>, start: (u32, u32), end: (u32, u32)) {
        let matrix = Matrix::from_vec(
            input.len(),
            input[0].len(),
            input.into_iter().flatten().collect(),
        )
        .unwrap();

        let grid: Grid = matrix.into();
        
        println!("{grid:?}");

        let result: (Vec<(u32, u32)>, u32) = dijkstra(
            &start,
            |n| {
                grid.neighbours((n.0 as usize, n.1 as usize))
                    .into_iter()
                    .map(|n| ((n.0 as u32, n.1 as u32), 1))
                    .collect::<Vec<((u32, u32), u32)>>()
            },
            |n| *n == end,
        )
        .unwrap();

        println!("{result:?}");

        output(&result.0);
    }
}

export!(MyImpl);
