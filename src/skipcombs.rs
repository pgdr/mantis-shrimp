pub struct SkippableCombinations<I>
where
    I: Iterator,
{
    data: Vec<I::Item>,
    pointers: Vec<usize>,
    stop: Vec<usize>,
    last_moved: usize,
    finished: bool,
}

impl<I> SkippableCombinations<I>
where
    I: Iterator,
{
    fn new(data: Vec<I::Item>, k: usize) -> Self {
        let n = data.len();
        if n < k {
            return SkippableCombinations {
                data: vec![],
                pointers: vec![],
                stop: vec![],
                finished: true,
                last_moved: 0,
            };
        }

        let mut pointers: Vec<usize> = (0..k as usize).collect();
        let mut stop: Vec<usize> = (n - k..n).collect();
        pointers.insert(0, 0); // Add guard elements
        stop.insert(0, usize::MAX);
        SkippableCombinations {
            data,
            pointers,
            stop,
            finished: false,
            last_moved: usize::MAX,
        }
    }

    pub fn skip_prefix(&mut self, prefix_length: usize) {
        assert!(prefix_length < self.pointers.len());

        // This index is one larger than expected because of the guard element
        let mut ix = prefix_length;

        if ix == 0 {
            self.finished = true;
            return;
        }

        if self.last_moved <= ix {
            // We already advanced past this prefix
            self.last_moved = usize::MAX;
            return;
        }
        while self.pointers[ix as usize] == self.stop[ix as usize] {
            ix -= 1;
        }

        // Advance pointer
        self.pointers[ix] += 1;
        // This does not 'count' as moving the pointer: otherwise successive calls to
        // .skip_prefix would not work.
        self.last_moved = usize::MAX;

        // Reset subsequent pointers
        let mut pos = self.pointers[ix];
        while ix < self.pointers.len() {
            self.pointers[ix] = pos;
            ix += 1;
            pos += 1;
        }
    }
}

impl<I, T> Iterator for SkippableCombinations<I>
where
    I: Iterator<Item = T>,
    T: Clone,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // Collect elements at current pointer positions.
        // Note that the first element is a guard element so we must skip it.
        let current = self
            .pointers
            .iter()
            .skip(1)
            .map(|&ix| self.data[ix].clone())
            .collect();

        // Advance pointers
        // 1) Find a pointer which can be advanced (is not at 'stop'), starting at the back
        //    Note: this loop always terminates at the guard element at index 0
        let mut ix = self.pointers.len() - 1;
        while self.pointers[ix as usize] == self.stop[ix as usize] {
            ix -= 1;
        }

        if ix == 0 {
            // We hit the guard element
            self.finished = true;
        } else {
            // 2) Advance pointer
            self.pointers[ix] += 1;
            self.last_moved = ix;

            // 3) Reset all following pointers to positions right after the
            //    one we just moved
            let mut pos = self.pointers[ix];
            while ix < self.pointers.len() {
                self.pointers[ix] = pos;
                ix += 1;
                pos += 1;
            }
        }

        Some(current)
    }
}

pub trait SkippableCombinationsIter<I>
where
    I: Iterator,
{
    fn combinations_skippable(self, k: usize) -> SkippableCombinations<I>;
}

impl<I> SkippableCombinationsIter<I> for I
where
    I: Iterator + Sized,
{
    fn combinations_skippable(self, k: usize) -> SkippableCombinations<I> {
        let data: Vec<I::Item> = self.collect();
        SkippableCombinations::new(data, k)
    }
}
