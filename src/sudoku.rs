use std::collections::HashSet;
use crate::ExactCover;

#[derive(Clone, Copy, Debug)]
pub struct SudokuClue {
    pub row: u8,
    pub col: u8,
    pub num: u8,
}

#[derive(Clone, Debug)]
pub struct SudokuPuzzle {
    clues: Vec<SudokuClue>,
    size: usize,
    covered_cols: HashSet<usize>,
}

impl SudokuPuzzle {
    pub fn new<C>(clues: C, size: usize) -> SudokuPuzzle
    where
        C: IntoIterator<Item = SudokuClue>,
    {
        if (size as f64).sqrt().fract() >= 1e-10 {
            panic!("size {} not a square number", size)
        }

        // most sudoku puzzles have less than half the spaces filled
        let mut clue_vec = Vec::with_capacity(size*size / 2);
        let mut covered_cols = HashSet::with_capacity(size*size / 2);

        // validate puzzle clues and pre-compute covered_cols
        for clue in clues {
            if clue.row as usize >= size || clue.col as usize >= size || clue.num as usize >= size {
                panic!("clue outside bounds of {}x{} sudoku: {:?}", size, size, clue);
            }
            for &cols in &SudokuPuzzle::ec_cols(&clue, size) {
                if !covered_cols.insert(cols) {
                    panic!("conflict with previous clues: {:?}", clue);
                }
            }
            clue_vec.push(clue)
        }

        SudokuPuzzle {
            clues: clue_vec,
            size,
            covered_cols,
        }
    }

    fn ec_cols(clue: &SudokuClue, size: usize) -> Vec<usize> {
        vec![
            SudokuPuzzle::xy_col(clue, size),
            SudokuPuzzle::xn_col(clue, size),
            SudokuPuzzle::yn_col(clue, size),
            SudokuPuzzle::boxn_col(clue, size),
        ]
    }

    fn xy_col(clue: &SudokuClue, size: usize) -> usize {
        clue.row as usize + clue.col as usize * size
    }

    fn xn_col(clue: &SudokuClue, size: usize) -> usize {
        size*size + clue.num as usize + clue.row as usize * size
    }

    fn yn_col(clue: &SudokuClue, size: usize) -> usize {
        size*size*2 + clue.num as usize + clue.col as usize * size
    }

    fn boxn_col(clue: &SudokuClue, size: usize) -> usize {
        size*size*3 + clue.num as usize + SudokuPuzzle::box_idx(clue, size)*size
    }

    fn box_idx(clue: &SudokuClue, size: usize) -> usize {
        let box_size = (size as f64).sqrt().trunc() as usize;
        clue.row as usize / box_size + (clue.col as usize / box_size) * box_size
    }
}

impl ExactCover for SudokuPuzzle {
    type Label = SudokuClue;

    fn exact_cover_rows<'a>(&'a self) -> Box<dyn Iterator<Item = (SudokuClue, Vec<usize>)> + 'a> {
        Box::new(
            // iterate over all row,col,num combinations...
            (0..self.size)
            .flat_map(move |x| (0..self.size).map(move |y| (x,y)))
            .flat_map(move |(x,y)| (0..self.size).map(move |n| SudokuClue {
                row: x as u8,
                col: y as u8,
                num: n as u8}))
            // ...map to exact cover columns with row label...
            .map(move |clue| (clue, SudokuPuzzle::ec_cols(&clue, self.size)))
            // ...and remove rows that are already covered by the given clues.
            .filter(move |(_, ec_cols)| !ec_cols.iter().any(|col| self.covered_cols.contains(col)))
            )
    }

    fn exact_cover_num_cols(&self) -> usize {
        self.size*self.size*4
    }
}