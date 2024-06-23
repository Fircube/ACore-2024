pub trait StepByOne {
    fn step(&mut self);
}

#[derive(Copy, Clone)]
pub struct Range<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd,
{
    start: T,
    end: T,
}

impl<T> Range<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd,
{
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }
    pub fn get_start(&self) -> T {
        self.start
    }
    pub fn get_end(&self) -> T {
        self.end
    }
}

pub struct RangeIterator<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd,
{
    current: T,
    end: T,
}

impl<T> IntoIterator for Range<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd,
{
    type Item = T;
    type IntoIter = RangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        RangeIterator::new(self.start, self.end)
    }
}

impl<T> RangeIterator<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd,
{
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}

impl<T> Iterator for RangeIterator<T>
    where
        T: StepByOne + Copy + PartialEq + PartialOrd,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let curr = self.current;
            self.current.step();
            Some(curr)
        }
    }
}