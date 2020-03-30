#![feature(test)]

extern crate test;
use test::Bencher;

extern crate rand;
use rand::{Rng, XorShiftRng};

extern crate noisy_float;
use noisy_float::prelude::*;

extern crate median;
use median::Filter as LinkedListFilter;

extern crate arraydeque;
use arraydeque::{Array, ArrayDeque, Wrapping};

use std::{fmt, mem};

const FILTER_WIDTH: usize = 20;
const ITERATIONS: usize = 10_000;

pub struct QuickSelectFilter<A>
where
    A: Array,
{
    buffer: ArrayDeque<A, Wrapping>,
    aux_buffer: mem::MaybeUninit<A>,
}

impl<A> QuickSelectFilter<A>
where
    A: Array,
    A::Item: PartialOrd + Clone + fmt::Debug,
{
    pub fn new() -> Self {
        let mut aux_buffer = mem::MaybeUninit::<A>::uninit();
        QuickSelectFilter {
            buffer: ArrayDeque::new(),
            aux_buffer,
        }
    }

    #[cfg_attr(feature = "flame_it", flame)]
    pub fn consume(&mut self, value: A::Item) -> A::Item {
        self.buffer.push_back(value);
        {
            let size = self.buffer.len();
            let slice = &mut self.aux_buffer.assume_init().as_mut_slice()[0..size];
            for (index, item) in self.buffer.iter().enumerate() {
                slice[index] = item.clone();
            }
        }
        self.find_median().clone()
    }

    #[cfg_attr(feature = "flame_it", flame)]
    fn find_median(&mut self) -> &A::Item {
        let size = self.buffer.len();

        let mut low = 0;
        let mut high = size - 1;
        let k = (size - 1) / 2;

        let partition = |slice: &mut [A::Item], low, high, pivot| {
            slice.swap(pivot, high);
            let mut index = low;
            for i in low..high {
                if slice[i] < slice[high] {
                    slice.swap(index, i);
                    index += 1;
                }
            }
            slice.swap(high, index);
            index
        };

        let size = self.buffer.len();
        let slice = &mut self.aux_buffer.assume_init().as_mut_slice()[0..size];
        if low == high {
            &slice[low]
        } else {
            loop {
                let pivot = if high == low {
                    low
                } else {
                    low + ((high - low) / 2) // middle element
                };
                let pivot = partition(slice, low, high, pivot);
                if k == pivot {
                    return &slice[k];
                } else if k < pivot {
                    high = pivot - 1;
                } else {
                    low = pivot + 1;
                }
            }
        }
    }
}

impl<A> Default for QuickSelectFilter<A>
where
    A: Array,
    A::Item: PartialOrd + Clone + fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

#[bench]
fn placebo(b: &mut Bencher) {
    b.iter(|| {
        let mut rng = XorShiftRng::new_unseeded();
        for i in 0..ITERATIONS {
            let signal = (i as f32).sin();
            let noise = rng.gen::<f32>();
            let value = r32(signal + noise);
            test::black_box(value);
        }
    })
}

#[bench]
fn linked_list(b: &mut Bencher) {
    b.iter(|| {
        let mut rng = XorShiftRng::new_unseeded();
        let mut filter = LinkedListFilter::new(FILTER_WIDTH);
        for i in 0..ITERATIONS {
            let signal = (i as f32).sin();
            let noise = rng.gen::<f32>();
            let value = r32(signal + noise);
            let filtered = filter.consume(value);
            test::black_box(filtered);
        }
    })
}

#[bench]
fn quick_select(b: &mut Bencher) {
    b.iter(|| {
        let mut rng = XorShiftRng::new_unseeded();
        let mut filter: QuickSelectFilter<[_; FILTER_WIDTH]> = QuickSelectFilter::new();
        for i in 0..ITERATIONS {
            let signal = (i as f32).sin();
            let noise = rng.gen::<f32>();
            let value = r32(signal + noise);
            let filtered = filter.consume(value);
            test::black_box(filtered);
        }
    })
}
