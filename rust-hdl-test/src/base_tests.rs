#[cfg(test)]
mod tests {
    use rust_hdl_core::bits::{Bit, Bits, clog2};
    use rust_hdl_core::block::Block;
    use rust_hdl_core::check_connected::check_connected;
    use rust_hdl_core::clock::Clock;
    use rust_hdl_core::constant::Constant;
    use rust_hdl_core::dff::DFF;
    use rust_hdl_core::direction::{In, Local, Out};
    use rust_hdl_core::logic::Logic;
    use rust_hdl_core::module_defines::ModuleDefines;
    use rust_hdl_core::probe::Probe;
    use rust_hdl_core::signal::Signal;
    use rust_hdl_core::simulate;
    use rust_hdl_core::simulate::{Endpoint, simulate, Simulation};
    use rust_hdl_core::synth::Synth;
    use rust_hdl_macros::hdl_gen;
    use rust_hdl_macros::LogicBlock;
    use rust_hdl_macros::LogicInterface;
    use crate::strobe::Strobe;

    #[derive(Copy, Clone, Debug, PartialEq)]
    enum MyState {
        Init,
        Start,
        Running,
        Paused,
        Stopped,
    }

    impl Default for MyState {
        fn default() -> Self {
            Self::Init
        }
    }

    impl Synth for MyState {
        const BITS: usize = clog2(5);
        const ENUM_TYPE: bool = true;
        const TYPE_NAME: &'static str = "MyState";
        fn name(ndx: usize) -> &'static str {
            match ndx {
                0 => "Init",
                1 => "Start",
                2 => "Running",
                3 => "Paused",
                4 => "Stopped",
                _ => "",
            }
        }
    }

    #[derive(Clone, Debug, LogicBlock)]
    struct StateMachine {
        pub clock: Signal<In, Clock>,
        pub advance: Signal<In, Bit>,
        state: DFF<MyState>,
    }

    impl StateMachine {
        pub fn new() -> StateMachine {
            StateMachine {
                clock: Signal::default(),
                advance: Signal::default(),
                state: DFF::new(MyState::Init),
            }
        }
    }

    impl Logic for StateMachine {
        fn update(&mut self) {
            self.state.clk.next = self.clock.val;

            if self.advance.val {
                match self.state.q.val {
                    MyState::Init => self.state.d.next = MyState::Start,
                    MyState::Start => self.state.d.next = MyState::Running,
                    MyState::Running => self.state.d.next = MyState::Paused,
                    MyState::Paused => self.state.d.next = MyState::Stopped,
                    MyState::Stopped => self.state.d.next = MyState::Init,
                }
            }
        }
    }

    #[test]
    fn test_visit_version() {
        let mut uut: Strobe<4> = Strobe::default();
        // Simulate 100 clock cycles
        uut.enable.next = true;
        println!("Starting");
        uut.clock.connect();
        uut.enable.connect();
        uut.connect_all();
        check_connected(&uut);
        let mut strobe_count = 0;
        for clock in 0..100_000_000 {
            uut.clock.next = Clock(clock % 2 == 0);
            if !simulate(&mut uut, 10) {
                panic!("Logic did not converge");
            }
            if uut.strobe.val {
                strobe_count += 1;
            }
        }
        assert_eq!(strobe_count, 6_250_000);
    }

    #[test]
    fn test_enum_state() {
        #[derive(Copy, Clone, Debug, PartialEq)]
        enum MyState {
            Init,
            Start,
            Running,
            Paused,
            Stopped,
        }

        impl Default for MyState {
            fn default() -> Self {
                Self::Init
            }
        }

        impl Synth for MyState {
            const BITS: usize = clog2(5);
            const ENUM_TYPE: bool = true;
            const TYPE_NAME: &'static str = "MyState";
            fn name(ndx: usize) -> &'static str {
                match ndx {
                    0 => "Init",
                    1 => "Start",
                    2 => "Running",
                    3 => "Paused",
                    4 => "Stopped",
                    _ => "",
                }
            }
        }

        #[derive(Clone, Debug, LogicBlock)]
        struct StateMachine {
            pub clock: Signal<In, Clock>,
            pub advance: Signal<In, Bit>,
            state: DFF<MyState>,
        }

        impl StateMachine {
            pub fn new() -> StateMachine {
                StateMachine {
                    clock: Signal::default(),
                    advance: Signal::default(),
                    state: DFF::new(MyState::Init),
                }
            }
        }

        impl Logic for StateMachine {
            #[hdl_gen]
            fn update(&mut self) {
                self.state.clk.next = self.clock.val;

                if self.advance.val {
                    match self.state.q.val {
                        MyState::Init => self.state.d.next = MyState::Start,
                        MyState::Start => self.state.d.next = MyState::Running,
                        MyState::Running => self.state.d.next = MyState::Paused,
                        MyState::Paused => self.state.d.next = MyState::Stopped,
                        MyState::Stopped => self.state.d.next = MyState::Init,
                    }
                }
            }
        }

        let mut uut = StateMachine::new();
        println!("Starting");
        uut.clock.connect();
        uut.advance.connect();
        uut.connect_all();
        check_connected(&uut);
        for clock in 0..10 {
            uut.clock.next = Clock(clock % 2 == 0);
            uut.advance.next = true;
            if !simulate(&mut uut, 10) {
                panic!("Logic did not converge");
            }
            println!("State {:?}", uut.state.q.val);
        }
        let mut defines = ModuleDefines::default();
        uut.accept("uut", &mut defines);
        defines.defines();
    }

    #[test]
    fn test_write_modules() {
        #[derive(Clone, Debug, LogicBlock)]
        struct StrobePair {
            pub clock: Signal<In, Clock>,
            pub enable: Signal<In, Bit>,
            a_strobe: Strobe<4>,
            b_strobe: Strobe<6>,
            increment: Constant<Bits<6>>,
            local: Signal<Local, Bit>,
        }

        impl StrobePair {
            pub fn new() -> StrobePair {
                Self {
                    a_strobe: Strobe::default(),
                    b_strobe: Strobe::default(),
                    clock: Signal::default(),
                    enable: Signal::default(),
                    increment: Constant::new(32_usize.into()),
                    local: Signal::default(),
                }
            }
        }

        impl Logic for StrobePair {
            fn update(&mut self) {}
            fn connect(&mut self) {
                self.a_strobe.enable.connect();
                self.b_strobe.enable.connect();
                self.a_strobe.clock.connect();
                self.b_strobe.clock.connect();
                self.local.connect();
            }
        }

        let mut uut = StrobePair::new();
        // Simulate 100 clock cycles
        //uut.enable.next = true;
        println!("Starting");
        uut.clock.connect();
        uut.enable.connect();
        uut.connect_all();
        check_connected(&uut);
        let mut defines = ModuleDefines::default();
        uut.accept("uut", &mut defines);
        defines.defines();
    }

    #[test]
    fn test_async() {
        #[derive(LogicBlock, Clone, Default)]
        struct Semaphore {
            pub push: Signal<In, Bit>,
            pub pop: Signal<In, Bit>,
            pub clk: Signal<In, Clock>,
            pub empty: Signal<Out, Bit>,
            pub full: Signal<Out, Bit>,
            will_read: Signal<Local, Bit>,
            will_write: Signal<Local, Bit>,
            count: DFF<Bits<4>>,
        }

        impl Logic for Semaphore {
            #[hdl_gen]
            fn update(&mut self) {
                self.count.clk.next = self.clk.val;
                self.count.d.next = self.count.q.val;

                self.will_read.next = !self.empty.val && self.pop.val;
                self.will_write.next = !self.full.val && self.push.val;

                if self.will_read.val && !self.will_write.val {
                    self.count.d.next = self.count.q.val - 1_u32;
                } else if self.will_write.val && !self.will_read.val {
                    self.count.d.next = self.count.q.val + 1_u32;
                }

                self.full.next = self.count.q.val == 15_u32;
                self.empty.next = self.count.q.val == 0_u32;
            }
        }

        let mut uut = Semaphore::default();
        // Simulate 100 clock cycles
        uut.clk.connect();
        uut.pop.connect();
        uut.push.connect();
        uut.connect_all();
        check_connected(&uut);
    }

    /*
    pub async fn fifo_vector_feeder<T: Synthesizable>(
        clock: Clock,
        mut writer: FIFOWriterClient<T>,
        data: Vec<T>,
        prob_pause: f64,
        pause_len: u64,
    ) -> Result<(), HDLError> {
        writer.write.set(false);
        clock.next_negedge().await;
        for datum in data {
            while writer.full.get() {
                writer.write.set(false);
                clock.next_negedge().await;
            }
            if rand::thread_rng().gen::<f64>() < prob_pause {
                writer.write.set(false);
                clock.delay_negedge(pause_len).await;
            }
            writer.input.set(datum);
            writer.write.set(true);
            clock.next_negedge().await;
        }
        writer.write.set(false);
        if writer.overflow.get() {
            Err(HDLError::TestBenchFailed("FIFO overflowed".to_string()))
        } else {
            Ok(())
        }
    }
     */

    struct Circuit {
        x: i32,
        strobe: Strobe<4>,
    }

    fn sample_func(mut ep: Endpoint<Circuit>) -> simulate::Result<()> {
        // Need an initialization stage...
        // Get the initial circuit - this must be serviced first.
        println!("Initialize TB 1");
        let x = ep.init()?;
        println!("Hello from TB 1");
        let mut x = ep.wait(0, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        x.x = 42;
        let mut x = ep.wait(100, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        x.x = 100;
        let x = ep.watch(|m| m.x == 89, x)?;
        println!("Hello from TB1 where x value is {}", x.x);
        let x = ep.wait(250, x)?;
        println!("Hello from TB 1 at time {}", ep.time());
        // This is called last
        ep.done(x)?;
        println!("TB 1 done");
        Ok(())
    }

    fn sample_func2(mut ep: Endpoint<Circuit>) -> simulate::Result<()> {
        let x = ep.init()?;
        println!("Hello from TB 2");
        let mut x = ep.wait(125, x)?;
        println!("Hello from TB 2 at time {}", ep.time());
        x.x = 88;
        ep.done(x)?;
        println!("TB 2 done");
        Ok(())
    }

    #[test]
    fn test_tb() {
        let mut sim = Simulation::new();
        let ep1 = sim.endpoint();
        let sf1 = std::thread::spawn(move || sample_func(ep1));
        let ep2 = sim.endpoint();
        let sf2 = std::thread::spawn(move || sample_func2(ep2));
        let x = Circuit {
            x: 0,
            strobe: Strobe::default(),
        };
        sim.run(x, 1000).unwrap();
        sf1.join().unwrap();
        sf2.join().unwrap();
    }
}