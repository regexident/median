// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An implementation of a heap-allocated, efficient O(n) median filter.

use std::fmt;

#[derive(Clone, PartialEq, Eq)]
struct ListNode<T> {
    value: Option<T>,
    previous: usize,
    next: usize,
}

impl<T> fmt::Debug for ListNode<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "@{:?}-{:?}-@{:?}", self.previous, self.value, self.next)
    }
}

/// An implementation of a median filter with linear complexity.
///
/// While the common naïve implementation of a median filter
/// has a worst-case complexity of `O(n^2)` (due to having to sort the sliding window)
/// the use of a combination of linked list and ring buffer allows for
/// a worst-case complexity of `O(n)`.
#[derive(Clone, Debug)]
pub struct Filter<T> {
    // Buffer of list nodes:
    buffer: Vec<ListNode<T>>,
    // Cursor into circular buffer of data:
    cursor: usize,
    // Cursor to beginning of circular list:
    head: usize,
    // Cursor to median of circular list:
    median: usize,
}

impl<T> Filter<T>
where
    T: Clone + PartialOrd,
{
    /// Creates a new median filter with a given window size.
    pub fn new(size: usize) -> Self {
        let mut buffer = Vec::with_capacity(size);
        for i in 0..size {
            buffer.push(ListNode {
                value: None,
                previous: (i + size - 1) % size,
                next: (i + 1) % size,
            });
        }
        Filter {
            buffer,
            cursor: 0,
            head: 0,
            median: 0,
        }
    }

    /// Returns the window size of the filter.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if the filter has a length of `0`.
    #[inline]
    pub fn is_empty(&self) -> usize {
        self.len()
    }

    /// Returns the filter buffer's current median value, panicking if empty.
    #[inline]
    pub fn median(&self) -> T {
        assert!(!self.buffer.is_empty());

        unsafe { self.read_median() }
    }

    /// Returns the filter buffer's current min value, panicking if empty.
    #[inline]
    pub fn min(&self) -> T {
        assert!(!self.buffer.is_empty());

        unsafe { self.read_min() }
    }

    /// Returns the filter buffer's current max value, panicking if empty.
    #[inline]
    pub fn max(&self) -> T {
        assert!(!self.buffer.is_empty());

        unsafe { self.read_max() }
    }

    /// Applies a median filter to the consumed value.
    ///
    /// # Implementation
    ///
    /// The algorithm makes use of a ring buffer of the same size as its filter window.
    /// Inserting values into the ring buffer appends them to a linked list that is *embedded*
    /// inside said ring buffer (using relative integer jump offsets as links).
    ///
    /// # Example
    ///
    /// Given a sequence of values `[3, 2, 4, 6, 5, 1]` and a buffer of size 5,
    /// the buffer would be filled like this:
    ///
    /// ```plain
    /// new(5)  consume(3)  consume(2)  consume(4)  consume(6)  consume(5)  consume(1)
    /// ▶︎[ ]      ▷[3]       ┌→[3]       ┌→[3]─┐     ┌→[3]─┐    ▶︎┌→[3]─┐      ▷[1]─┐
    ///  [ ]      ▶︎[ ]      ▷└─[2]      ▷└─[2] │    ▷└─[2] │    ▷└─[2] │    ▶︎┌─[2]←┘
    ///  [ ]       [ ]        ▶︎[ ]         [4]←┘     ┌─[4]←┘     ┌─[4]←┘     └→[4]─┐
    ///  [ ]       [ ]         [ ]        ▶︎[ ]       └→[6]       │ [6]←┐     ┌→[6] │
    ///  [ ]       [ ]         [ ]         [ ]        ▶︎[ ]       └→[5]─┘     └─[5]←┘
    /// ```
    ///
    /// # Algorithm
    ///
    /// 1. **Remove node** at current cursor (`▶︎`) from linked list, if it exists.
    ///    (by re-wiring its predecessor to its successor).
    /// 2. **Initialize** `current` and `median` index to first node of linked list (`▷`).
    /// 3. **Walk through** linked list, **searching** for insertion point.
    /// 4. **Shift median index** on every other hop (thus ending up in the list's median).
    /// 5. **Insert value** into ring buffer and linked list respectively.
    /// 6. **Update index** to linked list's first node, if necessary.
    /// 7. **Update ring buffer**'s cursor.
    /// 8. **Return median value**.
    ///
    /// (_Based on Phil Ekstrom, Embedded Systems Programming, November 2000._)

    pub fn consume(&mut self, value: T) -> T {
        // If the current head is about to be overwritten
        // we need to make sure to have the head point to
        // the next node after the current head:
        unsafe {
            self.move_head_forward();
        }

        // Remove the node that is about to be overwritten
        // from the linked list:
        unsafe {
            self.remove_node();
        }

        // Initialize `self.median` pointing
        // to the first (smallest) node in the sorted list:
        unsafe {
            self.initialize_median();
        }

        // Search for the insertion index in the linked list
        // in regards to `value` as the insertion index.
        unsafe {
            self.insert_value(&value);
        }

        // Update head to newly inserted node if
        // cursor's value <= head's value or head is empty:
        unsafe {
            self.update_head(&value);
        }

        // If the filter has an even window size, then shift the median
        // back one slot, so that it points to the left one
        // of the middle pair of median values
        unsafe {
            self.adjust_median_for_even_length();
        }

        // Increment and wrap data in pointer:
        unsafe {
            self.increment_cursor();
        }

        // Read node value from buffer at `self.medium`:
        unsafe { self.read_median() }
    }

    #[inline]
    fn should_insert(&self, value: &T, current: usize, index: usize) -> bool {
        if let Some(ref v) = self.buffer[current].value {
            (index + 1 == self.len()) || (v >= value)
        } else {
            true
        }
    }

    #[inline]
    unsafe fn move_head_forward(&mut self) {
        if self.cursor == self.head {
            self.head = self.buffer[self.head].next;
        }
    }

    #[inline]
    unsafe fn remove_node(&mut self) {
        let (predecessor, successor) = {
            let node = &self.buffer[self.cursor];
            (node.previous, node.next)
        };
        self.buffer[predecessor].next = successor;
        self.buffer[self.cursor] = ListNode {
            previous: usize::max_value(),
            value: None,
            next: usize::max_value(),
        };
        self.buffer[successor].previous = predecessor;
    }

    #[inline]
    unsafe fn initialize_median(&mut self) {
        self.median = self.head;
    }

    #[inline]
    unsafe fn insert_value(&mut self, value: &T) {
        let mut current = self.head;
        let buffer_len = self.len();
        let mut has_inserted = false;
        for index in 0..buffer_len {
            if !has_inserted {
                let should_insert = self.should_insert(value, current, index);
                if should_insert {
                    // Insert previously removed node with new value
                    // into linked list at given insertion index.
                    self.insert(value, current);
                    has_inserted = true;
                }
            }

            // Shift median on every other element in the list,
            // so that it ends up in the middle, eventually:
            self.shift_median(index, current);

            current = self.buffer[current].next;
        }
    }

    #[inline]
    unsafe fn insert(&mut self, value: &T, current: usize) {
        let successor = current;
        let predecessor = self.buffer[current].previous;
        debug_assert!(self.buffer.len() == 1 || current != self.cursor);
        self.buffer[predecessor].next = self.cursor;
        self.buffer[self.cursor] = ListNode {
            previous: predecessor,
            value: Some(value.clone()),
            next: successor,
        };
        self.buffer[successor].previous = self.cursor;
    }

    #[inline]
    unsafe fn shift_median(&mut self, index: usize, current: usize) {
        if (index & 0b1 == 0b1) && (self.buffer[current].value.is_some()) {
            self.median = self.buffer[self.median].next;
        }
    }

    #[inline]
    unsafe fn update_head(&mut self, value: &T) {
        let should_update_head = if let Some(ref head) = self.buffer[self.head].value {
            value <= head
        } else {
            true
        };

        if should_update_head {
            self.head = self.cursor;
            self.median = self.buffer[self.median].previous;
        }
    }

    #[inline]
    unsafe fn adjust_median_for_even_length(&mut self) {
        if self.len() % 2 == 0 {
            self.median = self.buffer[self.median].previous;
        }
    }

    #[inline]
    unsafe fn increment_cursor(&mut self) {
        self.cursor = (self.cursor + 1) % (self.len());
    }

    #[inline]
    unsafe fn read_median(&self) -> T {
        let index = self.median;
        self.buffer[index].value.clone().unwrap()
    }

    #[inline]
    unsafe fn read_min(&self) -> T {
        let index = self.head;
        self.buffer[index].value.clone().unwrap()
    }

    #[inline]
    unsafe fn read_max(&self) -> T {
        let index = (self.cursor + self.len() - 1) % (self.len());
        self.buffer[index].value.clone().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_filter {
        ($size:expr, $input:expr, $output:expr) => {
            let filter = Filter::new($size);
            let output: Vec<_> = $input
                .iter()
                .scan(filter, |filter, &input| Some(filter.consume(input)))
                .collect();
            assert_eq!(output, $output);
        };
    }

    #[test]
    fn single_peak_4() {
        let input = vec![10, 20, 30, 100, 30, 20, 10];
        let output = vec![10, 10, 20, 20, 30, 30, 20];

        test_filter!(4, input, output);
    }

    #[test]
    fn single_peak_5() {
        let input = vec![10, 20, 30, 100, 30, 20, 10];
        let output = vec![10, 10, 20, 20, 30, 30, 30];
        test_filter!(5, input, output);
    }

    #[test]
    fn single_valley_4() {
        let input = vec![90, 80, 70, 10, 70, 80, 90];
        let output = vec![90, 80, 80, 70, 70, 70, 70];
        test_filter!(4, input, output);
    }

    #[test]
    fn single_valley_5() {
        let input = vec![90, 80, 70, 10, 70, 80, 90];
        let output = vec![90, 80, 80, 70, 70, 70, 70];
        test_filter!(5, input, output);
    }

    #[test]
    fn single_outlier_4() {
        let input = vec![10, 10, 10, 100, 10, 10, 10];
        let output = vec![10, 10, 10, 10, 10, 10, 10];
        test_filter!(4, input, output);
    }

    #[test]
    fn single_outlier_5() {
        let input = vec![10, 10, 10, 100, 10, 10, 10];
        let output = vec![10, 10, 10, 10, 10, 10, 10];
        test_filter!(5, input, output);
    }

    #[test]
    fn triple_outlier_4() {
        let input = vec![10, 10, 100, 100, 100, 10, 10];
        let output = vec![10, 10, 10, 10, 100, 100, 10];
        test_filter!(4, input, output);
    }

    #[test]
    fn triple_outlier_5() {
        let input = vec![10, 10, 100, 100, 100, 10, 10];
        let output = vec![10, 10, 10, 10, 100, 100, 100];
        test_filter!(5, input, output);
    }

    #[test]
    fn quintuple_outlier_4() {
        let input = vec![10, 100, 100, 100, 100, 100, 10];
        let output = vec![10, 10, 100, 100, 100, 100, 100];
        test_filter!(4, input, output);
    }

    #[test]
    fn quintuple_outlier_5() {
        let input = vec![10, 100, 100, 100, 100, 100, 10];
        let output = vec![10, 10, 100, 100, 100, 100, 100];
        test_filter!(5, input, output);
    }

    #[test]
    fn alternating_4() {
        let input = vec![10, 20, 10, 20, 10, 20, 10];
        let output = vec![10, 10, 10, 10, 10, 10, 10];
        test_filter!(4, input, output);
    }

    #[test]
    fn alternating_5() {
        let input = vec![10, 20, 10, 20, 10, 20, 10];
        let output = vec![10, 10, 10, 10, 10, 20, 10];
        test_filter!(5, input, output);
    }

    #[test]
    fn ascending_4() {
        let input = vec![10, 20, 30, 40, 50, 60, 70];
        let output = vec![10, 10, 20, 20, 30, 40, 50];
        test_filter!(4, input, output);
    }

    #[test]
    fn ascending_5() {
        let input = vec![10, 20, 30, 40, 50, 60, 70];
        let output = vec![10, 10, 20, 20, 30, 40, 50];
        test_filter!(5, input, output);
    }

    #[test]
    fn descending_4() {
        let input = vec![70, 60, 50, 40, 30, 20, 10];
        let output = vec![70, 60, 60, 50, 40, 30, 20];
        test_filter!(4, input, output);
    }

    #[test]
    fn descending_5() {
        let input = vec![70, 60, 50, 40, 30, 20, 10];
        let output = vec![70, 60, 60, 50, 50, 40, 30];
        test_filter!(5, input, output);
    }

    #[test]
    fn min_max_median() {
        let mut filter = Filter::new(5);
        for input in vec![70, 50, 30, 10, 20, 40, 60] {
            filter.consume(input);
        }
        assert_eq!(filter.min(), 10);
        assert_eq!(filter.max(), 60);
        assert_eq!(filter.median(), 30);
    }
}
