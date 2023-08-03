// For tracing, we would like to add a tracer to the update function.
// That means the trait needs to include a tracer

// trait Synchronous {
//     type Input;
//     type Output;
//     type State;
//     type Tracer;
//     fn update(&self, tracer: &mut Trace, q: Self::State, trigger: Self::Input) -> (Self::Output, Self::State);
//     fn default_output(&self) -> Self::Output;
// }

// Then, in the update function, we can trace the various signals.
//
// For example, in the shot test, we would like to trace the trigger, the state,
// and the counter.

// Some additional thoughts on the issue of tracing.  We really only need to store variable length
// bit strings with each path (or static strings).  So our log messages look like this:

use std::cell::RefCell;

use rust_hdl::prelude::Bits;
use rust_hdl_x_macro::BitSerialize;

use crate::bit_iter::BitIter;

#[derive(Debug, Clone)]
enum TraceMessage {
    EnterModule(&'static str),
    EnterStruct(&'static str),
    Bool(&'static str, bool),
    Short(&'static str, usize, u32),
    Vector(&'static str, Vec<bool>),
    String(&'static str, &'static str),
    ExitStruct(),
    ExitModule(),
}

// A seperate setup pass allows us to eliminate the
// runtime cost of registering items with the tracer.

// The tracing interface is then

pub struct TraceId(usize);

pub enum TracerLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

pub trait Tracer {
    fn id<T: Traceable>(&mut self, name: &'static str, value: T) -> TraceId {
        T::allocate(name, self)
    }
    fn log(&mut self, level: TracerLevel, value: impl Traceable);
    fn enter_module(&mut self, name: &'static str);
    fn enter_struct(&mut self);
    fn field(&mut self, name: &'static str);
    fn item(&mut self, width: usize) -> TraceId;
    fn exit_struct(&mut self);
    fn exit_module(&mut self);
}

pub trait Traceable {
    fn allocate(name: &'static str, tracer: impl Tracer) -> TraceId;
    fn short(&mut self, id: TraceId, value: u64);
    fn long(&mut self, id: TraceId, value: &[u64]);
    fn string(&mut self, id: TraceId, value: &'static str);
}

pub trait TracerSetup {}

/*
So for a sample circuit...

struct Thing {
    pub sub_thing_1: Thing1,
    pub sub_thing_2: Thing2,
}

// Associated probe type?
struct Probes {
    id_input: TraceId,
    id_output: TraceId,
    id_d: TraceId,
    id_q: TraceId,
    probe_sub_thing_1: Thing1::Probes,
    probe_sub_thing_2: Thing2::Probes,
}


impl Synchronous for Thing {
    fn setup(&mut self, tracer: impl Tracer) {
        tracer.enter_module("thing");
        self.id_input = MyInputStruct::allocate(tracer);
        self.id_output = MyOutputStruct::allocate(tracer);
        self.id_q = MyStateStruct::allocate(tracer);
        self.id_d = MyStateStruct::allocate(tracer);
        self.sub_thing_1.setup(tracer);
        self.sub_thing_2.setup(tracer);
        tracer.exit_module();
    }
}

*/

pub struct NullTracer {}

impl Tracer for NullTracer {
    fn enter_module(&self, _name: &'static str) {}
    fn log(&self, _name: &'static str, _value: impl BitSerialize) {}
    fn exit_module(&self) {}
}

pub struct TracerModule<'a, T: Tracer + ?Sized> {
    tracer: &'a T,
}

impl<'a, T: Tracer + ?Sized> Drop for TracerModule<'a, T> {
    fn drop(&mut self) {
        self.tracer.exit_module();
    }
}

pub trait BitSerialize {
    fn serialize(&self, tag: &'static str, serializer: impl BitSerializer);
}

pub trait BitSerializer {
    fn enter_struct(&self, name: &'static str);
    fn bool(&self, tag: &'static str, value: bool);
    fn short(&self, tag: &'static str, bits: usize, value: u32);
    fn long(&self, tag: &'static str, bits: &[bool]);
    fn string(&self, tag: &'static str, value: &'static str);
    fn exit_struct(&self);
}

impl<const N: usize> BitSerialize for Bits<N> {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        match self {
            Bits::Short(x) => {
                if N == 1 {
                    serializer.bool(tag, x.get_bit(0));
                } else {
                    serializer.short(tag, N, x.short())
                }
            }
            Bits::Long(x) => serializer.long(tag, &x.bits()),
        }
    }
}

impl BitSerialize for bool {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        serializer.bool(tag, *self);
    }
}

impl BitSerialize for u8 {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        serializer.short(tag, 8, *self as u32);
    }
}

impl BitSerialize for u16 {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        serializer.short(tag, 16, *self as u32);
    }
}

impl BitSerialize for u32 {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        serializer.short(tag, 32, *self as u32);
    }
}

impl BitSerialize for u64 {
    fn serialize(&self, tag: &'static str, mut serializer: impl BitSerializer) {
        serializer.long(tag, BitIter::new(*self).collect::<Vec<_>>().as_slice());
    }
}

#[derive(BitSerialize)]
struct TwoBits {
    bit_1: bool,
    bit_2: bool,
}

#[derive(BitSerialize)]
struct NoWorky {
    bit_1: bool,
    bit_2: bool,
    part_3: u8,
    nibble_4: u16,
}

// A simple in memory tracer is then
struct InMemoryTracer {
    messages: RefCell<Vec<TraceMessage>>,
}

impl InMemoryTracer {
    fn new() -> Self {
        Self {
            messages: RefCell::new(Vec::new()),
        }
    }
}

impl Tracer for InMemoryTracer {
    fn enter_module(&self, name: &'static str) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::EnterModule(name));
    }
    fn log(&self, name: &'static str, value: impl BitSerialize) {
        value.serialize(name, self);
    }
    fn exit_module(&self) {
        self.messages.borrow_mut().push(TraceMessage::ExitModule());
    }
}

impl BitSerializer for InMemoryTracer {
    fn enter_struct(&self, name: &'static str) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::EnterStruct(name));
    }
    fn bool(&self, tag: &'static str, value: bool) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::Bool(tag, value));
    }
    fn short(&self, tag: &'static str, bits: usize, value: u32) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::Short(tag, bits, value));
    }
    fn long(&self, tag: &'static str, bits: &[bool]) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::Vector(tag, bits.to_vec()));
    }
    fn string(&self, tag: &'static str, value: &'static str) {
        self.messages
            .borrow_mut()
            .push(TraceMessage::String(tag, value));
    }
    fn exit_struct(&self) {
        self.messages.borrow_mut().push(TraceMessage::ExitStruct());
    }
}

impl<T: BitSerializer> BitSerializer for &T {
    fn enter_struct(&self, name: &'static str) {
        (*self).enter_struct(name);
    }
    fn bool(&self, tag: &'static str, value: bool) {
        (*self).bool(tag, value);
    }
    fn short(&self, tag: &'static str, bits: usize, value: u32) {
        (*self).short(tag, bits, value);
    }
    fn long(&self, tag: &'static str, values: &[bool]) {
        (*self).long(tag, values);
    }
    fn string(&self, tag: &'static str, value: &'static str) {
        (*self).string(tag, value);
    }
    fn exit_struct(&self) {
        (*self).exit_struct();
    }
}
