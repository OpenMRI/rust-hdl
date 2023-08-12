use rust_hdl::prelude::Bits;

use crate::{
    basic_logger::BasicLoggerBuilder,
    log::{LogBuilder, TagID},
    logger::Logger,
    synchronous::Synchronous,
};

struct BitCounter<const N: usize> {
    tag_input: TagID<bool>,
    tag_output: TagID<Bits<N>>,
}

impl<const N: usize> BitCounter<N> {
    fn new(mut builder: impl LogBuilder) -> Self {
        let tag_input = builder.tag("input");
        let tag_output = builder.tag("output");
        Self {
            tag_input,
            tag_output,
        }
    }
}

impl<const N: usize> Synchronous for BitCounter<N> {
    type State = Bits<N>;
    type Input = bool;
    type Output = Bits<N>;

    fn compute(
        &self,
        mut logger: impl Logger,
        input: Self::Input,
        state: Self::State,
    ) -> (Self::Output, Self::State) {
        logger.log(self.tag_input, input);
        let new_state = if input { state + 1 } else { state };
        let output = new_state;
        logger.log(self.tag_output, output);
        (output, new_state)
    }
}

#[test]
fn test_counter_with_bits_argument() {
    let mut logger_builder = BasicLoggerBuilder::default();
    let counter: BitCounter<24> = BitCounter::new(&mut logger_builder);
    let mut logger = logger_builder.build();
    let mut state: Bits<24> = Default::default();
    let mut last_output = Default::default();
    for cycle in 0..100_000_000 {
        let (output, new_state) = counter.compute(&mut logger, cycle % 2 == 0, state);
        state = new_state;
        last_output = output;
        //        println!("{} {}", output, state);
    }
    println!("Last output {last_output:x}");
}
