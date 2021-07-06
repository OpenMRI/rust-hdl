use crate::atom::{Atom, AtomKind};
use crate::block::Block;
use crate::clock::Clock;
use crate::direction::{Direction, In, Out};
use crate::logic::Logic;
use crate::probe::Probe;
use crate::synth::Synth;

#[derive(Copy, Clone, Debug)]
pub struct Signal<D: Direction, T: Synth> {
    pub next: T,
    pub val: T,
    prev: T,
    pub changed: bool,
    claimed: bool,
    dir: std::marker::PhantomData<D>,
}

impl<D: Direction, T: Synth> Atom for Signal<D, T> {
    fn bits(&self) -> usize {
        T::BITS
    }

    fn connected(&self) -> bool {
        self.claimed
    }

    fn changed(&self) -> bool {
        self.changed
    }

    fn kind(&self) -> AtomKind {
        D::KIND
    }

    fn is_enum(&self) -> bool {
        T::ENUM_TYPE
    }

    fn name(&self, ndx: usize) -> &'static str {
        T::name(ndx)
    }

    fn type_name(&self) -> &'static str {
        T::TYPE_NAME
    }
}

impl<D: Direction, T: Synth> Signal<D, T> {
    pub fn connect(&mut self) {
        assert!(!self.claimed);
        self.claimed = true;
    }
}

impl<D: Direction, T: Synth> Logic for Signal<D, T> {
    fn update(&mut self) {}
    fn connect(&mut self) {}
}

impl<D: Direction, T: Synth> Block for Signal<D, T> {
    fn connect_all(&mut self) {}

    fn update_all(&mut self) {
        self.changed = self.val != self.next;
        if self.changed {
            self.prev = self.val;
            self.val = self.next;
        }
    }

    fn has_changed(&self) -> bool {
        self.changed
    }

    fn accept(&self, name: &str, probe: &mut dyn Probe) {
        probe.visit_atom(name, self);
    }
}

impl Signal<In, Clock> {
    #[inline(always)]
    pub fn pos_edge(&self) -> bool {
        self.changed && self.val.0 && !self.prev.0
    }
    #[inline(always)]
    pub fn neg_edge(&self) -> bool {
        self.changed && !self.val.0 && self.prev.0
    }
}

impl<T: Synth> Signal<Out, T> {
    pub fn new_with_default(init: T) -> Signal<Out, T> {
        Self {
            next: T::default(),
            val: init,
            prev: init,
            changed: true,
            claimed: false,
            dir: std::marker::PhantomData,
        }
    }
}

impl<D: Direction, T: Synth> Default for Signal<D, T> {
    fn default() -> Self {
        Self {
            next: T::default(),
            val: T::default(),
            prev: T::default(),
            changed: false,
            claimed: false,
            dir: std::marker::PhantomData,
        }
    }
}