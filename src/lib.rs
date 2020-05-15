use std::collections::HashSet;
use std::fmt::Debug;

pub trait ExactCover
{
    type Label: Copy + Debug;

    fn exact_cover_rows<'a>(&'a self) -> Box<dyn Iterator<Item = (Self::Label, Vec<usize>)> + 'a>;
    fn exact_cover_num_cols(&self) -> usize;
}

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

#[derive(Default, Clone, Copy, Debug)]
struct Node {
    l: usize,
    r: usize,
    u: usize,
    d: usize,
    col: usize,
    data: usize,
}

pub struct DancingLinks<L>
where
    L: Copy + Debug
{
    node_list: Vec<Node>,
    num_cols: usize,
    row_labels: Vec<L>
}

impl<L> DancingLinks<L>
where
    L: Copy + Debug
{
    pub fn new<EC>(ec: EC) -> DancingLinks<L>
    where
        EC: ExactCover<Label = L>
    {
        let num_cols = ec.exact_cover_num_cols();
        let node_list = Vec::new();
        let row_labels = Vec::new();

        let mut dl = DancingLinks {
            node_list,
            num_cols,
            row_labels,
        };

        dl.setup_headers();

        for row in ec.exact_cover_rows() {
            dl.add_row(row);
        }

        dl.remove_empty_cols();

        //for (i, node) in dl.node_list.iter().enumerate() {
        //    println!("{}:\t{:?}", i, node);
        //}

        dl
    }

    fn setup_headers(&mut self) {
        let root = Node {
            col: 0x51deb00b,
            data: 0x51deb00b,
            l: self.num_cols,
            r: 1,
            u: 0,
            d: 0,
        };
        self.node_list.push(root);
        let num_cols = self.num_cols;
        self.node_list.extend((0..num_cols).map(|i| {
            Node {
                l: i,
                r: (i + 2) % (num_cols + 1),
                u: i + 1,
                d: i + 1,
                col: i,
                data: 0,
            }
        }));
    }

    fn header_index(&self, col: usize) -> usize {
        if col >= self.num_cols {
            panic!("header exceeded column bounds: {}", col);
        }
        col + 1
    }

    fn add_row(&mut self, (label, row): (L, Vec<usize>)) {
        let row_num = self.row_labels.len();
        self.row_labels.push(label);
        let mut idx = self.node_list.len();
        let row_start = idx;
        for (i, col) in row.iter().copied().enumerate() {
            if col >= self.num_cols {
                panic!("row labeled {:?} exceeded column bounds: {}", label, col);
            }
            let header = self.header_index(col);
            let new_node = Node {
                l: (i + row.len() - 1) % row.len() + row_start,
                r: (i + row.len() + 1) % row.len() + row_start,
                u: self.node_list[header].u,
                d: header,
                col: header,
                data: row_num,
            };
            self.node_list[new_node.u].d = idx;
            self.node_list[header].u = idx;
            self.node_list[header].data += 1;

            self.node_list.push(new_node);
            idx += 1;
        }
    }

    fn remove_empty_cols(&mut self) {
        for idx in 1..=self.num_cols {
            let node = self.node_list[idx];
            if self.node_list[idx].d == idx {
                self.node_list[node.l].r = node.r;
                self.node_list[node.r].l = node.l;
            }
        }
    }

    fn cover_col(&mut self, header_idx: usize) {
        let header_node = self.node_list[header_idx];

        self.node_list[header_node.l].r = header_node.r;
        self.node_list[header_node.r].l = header_node.l;

        let mut i = header_node.d;
        while i != header_idx {
            let mut j = self.node_list[i].r;
            while j != i {
                let node = self.node_list[j];
                self.node_list[node.d].u = node.u;
                self.node_list[node.u].d = node.d;
                self.node_list[node.col].data -= 1;
                
                j = node.r;
            }
            i = self.node_list[i].d;
        }
    }

    fn uncover_col(&mut self, header_idx: usize) {
        let header_node = self.node_list[header_idx];

        let mut i = self.node_list[header_idx].u;
        while i != header_idx {
            let mut j = self.node_list[i].l;
            while j != i {
                let node = self.node_list[j];
                self.node_list[node.col].data += 1;
                self.node_list[node.u].d = j;
                self.node_list[node.d].u = j;

                j = node.l;
            }
            i = self.node_list[i].u;
        }

        self.node_list[header_node.l].r = header_idx;
        self.node_list[header_node.r].l = header_idx;
    }

    pub fn print_solutions(&mut self) {
        let mut soln_vec = vec![];
        self.search(&mut soln_vec, 0);
    }

    fn search(&mut self, partial_soln: &mut Vec<usize>, k: usize) {
        if self.node_list[0].r == 0 {
            println!("** found solution! **");
            println!("{:#?}", partial_soln.iter().map(|&idx| self.row_labels[self.node_list[idx].data]).collect::<Vec<L>>());
            return;
        }
        
        let col = self.choose_col();
        self.cover_col(col);
        let mut r = self.node_list[col].d;
        while r != col {
            partial_soln.push(r);
            let mut j = self.node_list[r].r;
            while j != r {
                self.cover_col(self.node_list[j].col);
                j = self.node_list[j].r;
            }
            self.search(partial_soln, k+1);
            j = self.node_list[r].l;
            while j != r {
                self.uncover_col(self.node_list[j].col);
                j = self.node_list[j].l;
            }
            partial_soln.pop();
            r = self.node_list[r].d;
        }
        self.uncover_col(col);
    }

    fn choose_col(&self) -> usize {
        let mut min = usize::MAX;
        let mut min_idx = 0xabadb00b;
        let mut c = self.node_list[0].r;
        while c != 0 {
            if self.node_list[c].data < min {
                min_idx = c;
                min = self.node_list[c].data;
            }
            c = self.node_list[c].r;
        }
        if min_idx == 0xabadb00b {
            panic!("error choosing column");
        }
        min_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEC {
        num_cols: usize,
        data: Vec<Vec<usize>>,
    }

    impl ExactCover for TestEC {
        type Label = usize;

        fn exact_cover_num_cols(&self) -> usize {
            self.num_cols
        }

        fn exact_cover_rows<'a>(&'a self) -> Box<dyn Iterator<Item = (usize, Vec<usize>)> + 'a> {
            Box::new(self.data.iter().cloned().enumerate())
        }
    }

    #[test]
    fn test_dl() {
        let test_ec = TestEC {
            num_cols: 6,
            data: vec![vec![0,1], vec![1,2], vec![2,3], vec![3,4], vec![4,5], vec![0,5]],
        };
        let mut dl = DancingLinks::new(test_ec);
        dl.print_solutions();
    }

    #[test]
    fn test_sudoku() {
        // . . | 1 .
        // . 3 | . 4
        // ----+----
        // 3 . | 4 .
        // . 2 | . .
        let sudoku = SudokuPuzzle::new(
            [(0,2,0), (1,1,2), (1,3,3), (2,0,2), (2,2,3), (3,1,1)]
                .iter().map(|(x,y,n)| SudokuClue{row:*x, col:*y, num:*n}),
            4
        );

        let mut dl = DancingLinks::new(sudoku);
        dl.print_solutions();
    }
}
