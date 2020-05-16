use std::fmt::Debug;

pub trait ExactCover
{
    type Label: Copy + Debug;

    fn exact_cover_rows<'a>(&'a self) -> Box<dyn Iterator<Item = (Self::Label, Vec<usize>)> + 'a>;
    fn exact_cover_num_cols(&self) -> usize;
}

pub mod sudoku;

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
        let row_len = row.len();
        for (i, &col) in row.iter().enumerate() {
            if col >= self.num_cols {
                panic!("row labeled {:?} exceeded column bounds: {}", label, col);
            }
            let header = self.header_index(col);
            let new_node = Node {
                l: (i + row_len - 1) % row_len + row_start,
                r: (i + row_len + 1) % row_len + row_start,
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

    pub fn solve(&mut self) -> Vec<Vec<L>> {
        let mut partial_soln = vec![];
        let mut solution_list = vec![];
        self.search(&mut partial_soln, 0, &mut solution_list);
        solution_list.iter().map(|solution| solution.iter().map(|&idx| {
            self.row_labels[self.node_list[idx].data]
        }).collect()).collect()
    }

    fn search(&mut self, partial_soln: &mut Vec<usize>, k: usize, solution_list: &mut Vec<Vec<usize>>) {
        if self.node_list[0].r == 0 {
            solution_list.push(partial_soln.clone());
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
            self.search(partial_soln, k+1, solution_list);
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
        let solutions = dl.solve();
        assert_eq!(solutions.len(), 2);
        assert_eq!(solutions[0].len(), 3);
        assert_eq!(solutions[1].len(), 3);
    }
}
