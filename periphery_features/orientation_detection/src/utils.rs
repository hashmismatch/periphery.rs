use periphery_core::prelude::v1::*;


pub struct Combination<'a, A: 'a, B: 'a> {
    a: &'a [A],
    a_pos: usize,

    b: &'a [B],
    b_pos: usize,

    finished: bool
}

impl<'a, A, B> Combination<'a, A, B> where A: Copy, B: Copy {
    pub fn new(a: &'a [A], b: &'a [B]) -> Self {
        Combination {
            a: a,
            a_pos: 0,

            b: b,
            b_pos: 0,

            finished: false
        }
    }
}

impl<'a, A, B> Iterator for Combination<'a, A, B> where A: Copy, B: Copy {
    type Item = (A, B);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let ret = (self.a[self.a_pos], self.b[self.b_pos]);
        
        self.b_pos += 1;
        if self.b_pos >= self.b.len() {
            self.b_pos = 0;
            self.a_pos += 1;

            if self.a_pos == self.a.len() {
                self.finished = true;
            }
        }

        Some(ret)
    }
}



pub struct PermutationsWithRepetitions<'a, T: 'a> {
    src: &'a [T],
    buffer: [T; 10],
    permutation: [usize; 10],
    n: usize,
    k: usize,
    finished: bool
}

impl<'a, T> PermutationsWithRepetitions<'a, T> where T: Copy + Default {
    pub fn new(values: &'a [T], n: usize) -> Self {
        PermutationsWithRepetitions {
            src: values,
            buffer: Default::default(),
            permutation: Default::default(),
            n: n,
            k: values.len(),
            finished: false
        }
    }

    fn step(&mut self) {
        if self.finished { return; }
        
        // generate the permutation
        {
            for i in 0..self.n {
                self.buffer[i] = self.src[self.permutation[i]];
            }
        }

        {
            let mut i = 0;
            loop {
                self.permutation[i] += 1;
                if self.permutation[i] < self.k { break; }                
                self.permutation[i] = 0;
                i += 1;
                if i == self.n { 
                    self.finished = true;
                    break;
                }
            }
        }
    }

    pub fn next_permutation(&'a mut self) -> Option<&'a [T]> {
        if self.finished { return None; }
        self.step();
        Some(&self.buffer[..self.n])
    }
}

impl<'a, T> Iterator for PermutationsWithRepetitions<'a, T> where T: Copy + Default {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None; }
        self.step();
        Some(self.buffer[..self.n].into())
    }
}
