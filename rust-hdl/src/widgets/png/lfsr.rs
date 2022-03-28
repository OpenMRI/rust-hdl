use crate::core::prelude::*;
use crate::widgets::dff::DFF;

// Adopted from Alchitry.com Lucid module `pn_gen`
// This version does not provide seed setting.  It generates a fixed sequence.
#[derive(LogicBlock)]
pub struct LFSRSimple {
    pub clock: Signal<In, Clock>,
    pub strobe: Signal<In, Bit>,
    pub num: Signal<Out, Bits<32>>,
    x: DFF<Bits<32>>,
    y: DFF<Bits<32>>,
    z: DFF<Bits<32>>,
    w: DFF<Bits<32>>,
    t: Signal<Local, Bits<32>>,
}

const SEED: u128 = 0x843233523a613966423b622562592c62;

impl Default for LFSRSimple {
    fn default() -> Self {
        Self {
            clock: Default::default(),
            strobe: Default::default(),
            num: Default::default(),
            x: DFF::new((SEED & 0xFFFF_FFFF_u128).into()),
            y: DFF::new(((SEED >> 32) & 0xFFFF_FFFF_u128).into()),
            z: DFF::new(((SEED >> 64) & 0xFFFF_FFFF_u128).into()),
            w: DFF::new(((SEED >> 96) & 0xFFFF_FFFF_u128).into()),
            t: Default::default(),
        }
    }
}

impl Logic for LFSRSimple {
    #[hdl_gen]
    fn update(&mut self) {
        self.x.clk.next = self.clock.val();
        self.y.clk.next = self.clock.val();
        self.z.clk.next = self.clock.val();
        self.w.clk.next = self.clock.val();
        self.num.next = self.w.q.val();
        self.t.next = self.x.q.val() ^ (self.x.q.val() << 11_usize);
        self.x.d.next = self.x.q.val();
        self.y.d.next = self.y.q.val();
        self.w.d.next = self.w.q.val();
        self.z.d.next = self.z.q.val();
        if self.strobe.val() {
            self.x.d.next = self.y.q.val();
            self.y.d.next = self.z.q.val();
            self.z.d.next = self.w.q.val();
            self.w.d.next = self.w.q.val()
                ^ (self.w.q.val() >> 19_usize)
                ^ self.t.val()
                ^ (self.t.val() >> 8_usize);
        }
    }
}

#[test]
fn test_lfsr_simple_synthesizes() {
    let mut uut = TopWrap::new(LFSRSimple::default());
    uut.uut.clock.connect();
    uut.uut.strobe.connect();
    uut.connect_all();
    yosys_validate("lfsr", &generate_verilog(&uut)).unwrap();
}
