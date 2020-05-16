use fixedbitset::FixedBitSet;
use crate::ExactCover;

#[derive(Clone, Copy, Debug)]
pub struct SudokuClue {
    pub row: u8,
    pub col: u8,
    pub num: u8,
}

#[derive(Clone, Debug)]
pub struct SudokuPuzzle {
    size: u8,
    covered_cols: FixedBitSet,
}

impl SudokuPuzzle {

    pub fn from_tuples(clues: impl Iterator<Item = (u8,u8,u8)>, size: u8) -> SudokuPuzzle {
        if (size as f64).sqrt().fract() >= 1e-10 {
            panic!("size {} not a square number", size)
        }

        let mut covered_cols = FixedBitSet::with_capacity((size*size*4) as usize);

        // validate puzzle clues and pre-compute covered_cols
        for clue in clues {
            let clue = &[clue.0, clue.1, clue.2];
            if clue[0] >= size || clue[1] >= size || clue[2] >= size {
                panic!("clue outside bounds of {}x{} sudoku: {:?}", size, size, clue);
            }
            for col in SudokuPuzzle::ec_cols(clue, size as usize) {
                if covered_cols.put(col) {
                    panic!("conflict with previous clues: {:?}", clue);
                }
            }
        }

        SudokuPuzzle {
            size,
            covered_cols,
        }
    }

    pub fn from_slice(clues: &[u8], size: u8) -> SudokuPuzzle {
        if (size as f64).sqrt().fract() >= 1e-10 {
            panic!("size {} not a square number", size)
        }

        let mut covered_cols = FixedBitSet::with_capacity(size as usize * size as usize * 4);

        // validate puzzle clues and pre-compute covered_cols
        for clue in clues.chunks(3) {
            if clue[0] >= size || clue[1] >= size || clue[2] >= size {
                panic!("clue outside bounds of {}x{} sudoku: {:?}", size, size, clue);
            }
            for col in SudokuPuzzle::ec_cols(clue, size as usize) {
                if covered_cols.put(col) {
                    panic!("conflict with previous clues: {:?}", clue);
                }
            }
        }

        SudokuPuzzle {
            size,
            covered_cols,
        }
    }

    fn ec_cols(clue: &[u8], size: usize) -> Vec<usize> {
        vec![
            SudokuPuzzle::xy_col(clue, size),
            SudokuPuzzle::xn_col(clue, size),
            SudokuPuzzle::yn_col(clue, size),
            SudokuPuzzle::boxn_col(clue, size),
        ]
    }

    fn xy_col(clue: &[u8], size: usize) -> usize {
        clue[0] as usize + clue[1] as usize * size
    }

    fn xn_col(clue: &[u8], size: usize) -> usize {
        size*size + clue[2] as usize + clue[0] as usize * size
    }

    fn yn_col(clue: &[u8], size: usize) -> usize {
        size*size*2 + clue[2] as usize + clue[1] as usize * size
    }

    fn boxn_col(clue: &[u8], size: usize) -> usize {
        size*size*3 + clue[2] as usize + SudokuPuzzle::box_idx(clue, size)*size
    }

    fn box_idx(clue: &[u8], size: usize) -> usize {
        let box_size = (size as f64).sqrt().trunc() as usize;
        clue[0] as usize / box_size + (clue[1] as usize / box_size) * box_size
    }
}


impl ExactCover for SudokuPuzzle {
    type Label = (u8,u8,u8);

    fn exact_cover_rows<'a>(&'a self) -> Box<dyn Iterator<Item = ((u8,u8,u8), Vec<usize>)> + 'a> {
        Box::new(
            // iterate over all row,col,num combinations...
            (0..self.size)
            .flat_map(move |x| (0..self.size).map(move |y| (x,y)))
            .flat_map(move |(x,y)| (0..self.size).map(move |n| (x,y,n)))
            // ...map to exact cover columns with row label...
            .map(move |clue| (clue, SudokuPuzzle::ec_cols(&[clue.0, clue.1, clue.2], self.size as usize)))
            // ...and remove rows that are already covered by the given clues.
            .filter(move |(_, ec_cols)| !ec_cols.iter().any(|&col| self.covered_cols.contains(col)))
            )
    }

    fn exact_cover_num_cols(&self) -> usize {
        self.size as usize * self.size as usize * 4
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use super::*;
    use std::collections::{HashSet, HashMap};

    fn validate_solution(clues: &Vec<(u8,u8,u8)>, solution: &Vec<(u8,u8,u8)>, size: usize) {
        assert_eq!(clues.len() + solution.len(), size * size);
        let mut rows: HashMap<u8, HashSet<u8>> = HashMap::with_capacity(size);
        let mut cols: HashMap<u8, HashSet<u8>> = HashMap::with_capacity(size);
        let mut boxs: HashMap<u8, HashSet<u8>> = HashMap::with_capacity(size);

        for clue in clues.iter().chain(solution.iter()) {
            rows.entry(clue.0)
                .and_modify(|set: &mut HashSet<u8>| {
                    if !set.insert(clue.2) {
                        panic!("row conflict: {:?}", clue);
                    }
                })
                .or_insert_with(|| {
                    let mut set = HashSet::new();
                    set.insert(clue.2);
                    set
                });

            cols.entry(clue.1)
                .and_modify(|set: &mut HashSet<u8>| {
                    if !set.insert(clue.2) {
                        panic!("col conflict: {:?}", clue);
                    }
                })
                .or_insert_with(|| {
                    let mut set = HashSet::new();
                    set.insert(clue.2);
                    set
                });

            let box_size = (size as f64).sqrt() as u8;
            let box_n = clue.0 / box_size + (clue.1 / box_size) * box_size;
            boxs.entry(box_n)
                .and_modify(|set: &mut HashSet<u8>| {
                    if !set.insert(clue.2) {
                        panic!("box conflict: {:?}", clue);
                    }
                })
                .or_insert_with(|| {
                    let mut set = HashSet::new();
                    set.insert(clue.2);
                    set
                });
        }

        let complete_set: HashSet<u8> = (0..size).map(|i| i as u8).collect();
        for map in &[rows, cols, boxs] {
            let key_set: HashSet<u8> = map.keys().copied().collect();
            assert_eq!(key_set, complete_set);
            for val_set in map.values() {
                assert_eq!(*val_set, complete_set);
            }
        }
    }

    #[test]
    fn test_from_tuple() {
        // . . | 1 .
        // . 3 | . 4
        // ----+----
        // 3 . | 4 .
        // . 2 | . .

        let sudoku_clues: Vec<(u8,u8,u8)> = vec![(0,2,0), (1,1,2), (1,3,3), (2,0,2), (2,2,3), (3,1,1)];

        let sudoku = SudokuPuzzle::from_tuples(sudoku_clues.iter().copied(), 4);

        //println!("{:?}", sudoku.covered_cols.ones().map(|b| b.to_string() ).collect::<Vec<String>>().join(" "));
        //for row in sudoku.exact_cover_rows() {
        //    println!("{:?}", row);
        //}

        let mut dl = DancingLinks::new(sudoku);
        let solutions = dl.solve();
        //println!("{:?}", solutions);
        assert_eq!(solutions.len(), 1);
        assert_eq!(solutions[0].len(), 10);

        validate_solution(&sudoku_clues, &solutions[0], 4);
    }

    #[test]
    fn test_from_slice() {

        // 0 1 2   3 4 5   6 7 8

        // . 2 . | . . . | 9 4 .    0
        // . . . | . . 6 | . . 8    1
        // 1 . . | . 8 . | . . .    2
        // ------+-------+------
        // . . 8 | . 7 . | . . 1    3
        // . . . | . . . | . . 9    4
        // . 3 . | 5 . 9 | . . .    5
        // ------+-------+------
        // 4 . . | . 6 3 | 8 . .    6
        // 3 . . | 4 1 5 | . . .    7
        // . . . | . . . | 7 . .    8

        let clues: [u8; 69] = [
            0, 1, 1,
            0, 6, 8,
            0, 7, 3,
            1, 5, 5,
            1, 8, 7,
            2, 0, 0,
            2, 4, 7,
            3, 2, 7,
            3, 4, 6,
            3, 8, 0,
            4, 8, 8,
            5, 1, 2,
            5, 3, 4,
            5, 5, 8,
            6, 0, 3,
            6, 4, 5,
            6, 5, 2,
            6, 6, 7,
            7, 0, 2,
            7, 3, 3,
            7, 4, 0,
            7, 5, 4,
            8, 6, 6,
        ];

        let puzzle = SudokuPuzzle::from_slice(&clues, 9);
        let mut dl = DancingLinks::new(puzzle);
        let solutions = dl.solve();
        println!("{:?}", solutions[0].iter().filter(|clue| clue.2 == 8).cloned().collect::<Vec<(u8,u8,u8)>>());
        assert_eq!(solutions.len(), 1);
        validate_solution(&clues.chunks(3).map(|clue| (clue[0], clue[1], clue[2])).collect::<Vec<(u8,u8,u8)>>(), &solutions[0], 9);
    }
}