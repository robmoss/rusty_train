//! Calculate maximum network flows using the Edmonds-Karp algorithm.
//!
//! A flow network is described by a `N x N` [Matrix], which is indexed by
//! `(row, column)` tuples.
//! The element in row `i` and column `j` defines the maximum flow allowed
//! from node `i` to node `j`.
//!
//! - Each element must be zero or a positive integer.
//! - If element `(i, j)` is non-zero, element `(j, i)` must be zero.
//! - The source node has index `0`.
//! - The sink node has index `N - 1`.
//!
//! See [this article](https://cp-algorithms.com/graph/edmonds_karp.html) for
//! examples and further details.
//!
//! # Examples
//!
//! ```rust
//! use n18tile::ekmf::Matrix;
//!
//! // Construct a 6-node network matrix.
//! let mut network = Matrix::square(6);
//!
//! // Connect the source to nodes 1 and 4.
//! network[(0, 1)] = 4;
//! network[(0, 4)] = 2;
//!
//! // Internal connections between nodes 1-4.
//! network[(1, 2)] = 4;
//! network[(1, 3)] = 2;
//! network[(3, 2)] = 1;
//! network[(4, 3)] = 1;
//!
//! // Connect nodes 2, 3, and 4 to the sink.
//! network[(2, 5)] = 3;
//! network[(3, 5)] = 1;
//! network[(4, 5)] = 3;
//!
//! // Calculate the maximum flow through this network.
//! let max_flow = network.max_flow();
//! assert_eq!(max_flow, 6);
//! ```

/// Defines the capacity graph for a maximum-flow problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    vals: Vec<usize>,
}

impl Matrix {
    /// Returns a square matrix with `n` rows and `n` columns.
    pub fn square(n: usize) -> Self {
        Matrix {
            rows: n,
            cols: n,
            vals: vec![0; n * n],
        }
    }

    /// Returns the number of rows in this matrix.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Returns the number of columns in this matrix.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Returns the index into `self.vals` for the given 2D index.
    fn _index(&self, index: (usize, usize)) -> usize {
        let (row, col) = index;
        row * self.rows + col
    }

    /// Returns the maximum flow through this graph.
    ///
    /// Note that this mutates the matrix.
    /// If this is not desirable, use [max_flow_mat()](Self::max_flow_mat)
    /// instead.
    ///
    /// # Panics
    ///
    /// This panics if the matrix is not square.
    pub fn max_flow(&mut self) -> usize {
        // Ensure this is a square matrix.
        assert_eq!(self.rows(), self.cols());

        let n = self.rows();
        let mut flow = 0;
        while let Some((new_flow, parents)) = bfs(self) {
            flow += new_flow;
            let mut curr_ix = n - 1;
            while curr_ix > 0 {
                let prev_ix = parents[curr_ix].unwrap();
                self[(prev_ix, curr_ix)] -= new_flow;
                self[(curr_ix, prev_ix)] += new_flow;
                curr_ix = prev_ix;
            }
        }
        flow
    }

    /// Returns the maximum flow through this graph, and the matrix of flows
    /// within the graph.
    ///
    /// # Panics
    ///
    /// This panics if the matrix is not square.
    pub fn max_flow_mat(&self) -> (usize, Matrix) {
        let mut output = self.clone();
        let flow = output.max_flow();
        let n = output.rows();
        // There can only be flow along edges that had non-zero capacity to begin
        // with, so we can calculate the flow along each edge by subtracting the
        // remaining capacity from the original capacity, but only where the
        // original capacity is strictly positive (i.e., `> 0`).
        for r in 0..n {
            for c in 0..n {
                let index = (r, c);
                let orig = self[index];
                if orig > 0 {
                    output[index] = orig - output[index];
                } else {
                    output[index] = 0;
                }
            }
        }
        (flow, output)
    }
}

impl std::ops::Index<(usize, usize)> for Matrix {
    type Output = usize;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let ix = self._index(index);
        &self.vals[ix]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let ix = self._index(index);
        &mut self.vals[ix]
    }
}

impl std::fmt::Display for Matrix {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        for row in 0..self.rows {
            if row > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", self[(row, 0)])?;
            for col in 1..self.cols {
                write!(f, ", {}", self[(row, col)])?;
            }
        }
        Ok(())
    }
}

fn bfs(capacity: &mut Matrix) -> Option<(usize, Vec<Option<usize>>)> {
    use std::collections::VecDeque;
    let n = capacity.rows();
    let mut parents: Vec<Option<usize>> = vec![None; n];
    // Records (node_index, flow) tuples.
    let mut queue: VecDeque<(usize, usize)> = VecDeque::with_capacity(n);
    // Start at the source (node 0) with maximum flow.
    queue.push_back((0, usize::MAX));

    while let Some((node_ix, flow)) = queue.pop_front() {
        for i in 0..n {
            if capacity[(node_ix, i)] > 0 && parents[i].is_none() {
                // Visit this node.
                parents[i] = Some(node_ix);
                let flow = std::cmp::min(flow, capacity[(node_ix, i)]);
                if i == (n - 1) {
                    return Some((flow, parents));
                }
                queue.push_back((i, flow))
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {

    use super::Matrix;

    /// A six-node test case from
    /// [CP-Algorithms](https://cp-algorithms.com/graph/edmonds_karp.html).
    #[test]
    fn six_node_network_1() {
        let expected_flow = 10;
        let mut cap = Matrix::square(6);
        cap[(0, 1)] = 7;
        cap[(0, 4)] = 4;
        cap[(1, 2)] = 5;
        cap[(1, 3)] = 3;
        cap[(2, 5)] = 8;
        cap[(3, 2)] = 3;
        cap[(3, 5)] = 5;
        cap[(4, 1)] = 3;
        cap[(4, 3)] = 2;
        let (net_flow, flow_mat) = cap.max_flow_mat();
        println!("{}", flow_mat);
        assert_eq!(net_flow, expected_flow);
    }

    /// A six-node test case from
    /// [INGInious](https://inginious.org/course/competitive-programming/graphs-maxflow).
    #[test]
    fn six_node_network_2() {
        let expected_flow = 6;
        let mut cap = Matrix::square(6);
        cap[(0, 1)] = 4;
        cap[(0, 4)] = 2;
        cap[(1, 2)] = 4;
        cap[(1, 3)] = 2;
        cap[(2, 5)] = 3;
        cap[(3, 2)] = 1;
        cap[(3, 5)] = 1;
        cap[(4, 3)] = 1;
        cap[(4, 5)] = 3;
        let (net_flow, flow_mat) = cap.max_flow_mat();
        println!("{}", flow_mat);
        assert_eq!(net_flow, expected_flow);
    }

    /// A seven-node test case from
    /// [Wikipedia](https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm).
    #[test]
    fn seven_node_network() {
        let expected_flow = 5;
        let mut cap = Matrix::square(7);
        // Source connections (node A).
        cap[(0, 1)] = 3;
        cap[(0, 3)] = 3;
        // Internal connections (nodes B-F).
        cap[(1, 2)] = 4;
        cap[(2, 0)] = 3;
        cap[(2, 3)] = 1;
        cap[(2, 4)] = 2;
        cap[(3, 4)] = 2;
        cap[(3, 5)] = 6;
        cap[(4, 1)] = 1;
        // Sink connections (nodes E and F).
        cap[(4, 6)] = 1;
        cap[(5, 6)] = 9;
        let (net_flow, flow_mat) = cap.max_flow_mat();
        println!("{}", flow_mat);
        assert_eq!(net_flow, expected_flow);
    }
}
