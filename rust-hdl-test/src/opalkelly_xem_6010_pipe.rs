use crate::ok_tools::ok_test_prelude;
use rust_hdl_core::prelude::*;
use rust_hdl_ok::ok_pipe::PipeIn;
use rust_hdl_ok::prelude::*;
use rust_hdl_ok_frontpanel_sys::{make_u16_buffer, OkError};
use rust_hdl_widgets::prelude::*;
use std::num::Wrapping;

#[derive(LogicBlock)]
pub struct OpalKellyXEM6010PipeTest {
    pub hi: OpalKellyHostInterface,
    pub ok_host: OpalKellyHost,
    pub accum: DFF<Bits<16>, MHz48>,
    pub o_wire: WireOut<0x20>,
    pub i_pipe: PipeIn<0x80>,
}

impl OpalKellyXEM6010PipeTest {
    pub fn new() -> Self {
        Self {
            hi: OpalKellyHostInterface::xem_6010(),
            ok_host: Default::default(),
            accum: DFF::new(0_u16.into()),
            o_wire: Default::default(),
            i_pipe: Default::default(),
        }
    }
}

impl Logic for OpalKellyXEM6010PipeTest {
    #[hdl_gen]
    fn update(&mut self) {
        // Interface connections
        self.ok_host.hi.sig_in.next = self.hi.sig_in.val();
        self.hi.sig_out.next = self.ok_host.hi.sig_out.val();
        link!(self.hi.sig_inout, self.ok_host.hi.sig_inout);
        link!(self.hi.sig_aa, self.ok_host.hi.sig_aa);

        // Clock connections
        self.accum.clk.next = self.ok_host.ti_clk.val();

        // Bus connections
        self.o_wire.ok1.next = self.ok_host.ok1.val();
        self.i_pipe.ok1.next = self.ok_host.ok1.val();
        self.ok_host.ok2.next = self.o_wire.ok2.val() | self.i_pipe.ok2.val();

        // Logic
        self.accum.d.next = self.accum.q.val();
        if self.i_pipe.write.val().all() {
            self.accum.d.next = self.accum.q.val() + self.i_pipe.dataout.val();
        }
        self.o_wire.datain.next = self.accum.q.val();
    }
}

#[test]
fn test_opalkelly_xem_6010_pipe() {
    let mut uut = OpalKellyXEM6010PipeTest::new();
    uut.hi.sig_inout.connect();
    uut.hi.sig_in.connect();
    uut.hi.sig_out.connect();
    uut.hi.sig_aa.connect();
    uut.connect_all();
    crate::ok_tools::synth_obj(uut, "opalkelly_xem_6010_pipe");
}

fn sum_vec(t: &[u16]) -> u16 {
    let mut ret = Wrapping(0_u16);
    for x in t {
        ret += Wrapping(*x);
    }
    ret.0
}

#[test]
fn test_opalkelly_xem_6010_pipe_in_runtime() -> Result<(), OkError> {
    let hnd = ok_test_prelude("opalkelly_xem_6010_pipe/top.bit")?;
    let data = (0..1024 * 1024)
        .map(|_| rand::random::<u8>())
        .collect::<Vec<_>>();
    let data_16 = make_u16_buffer(&data);
    let cpu_sum = sum_vec(&data_16);
    hnd.write_to_pipe_in(0x80, &data)?;
    hnd.update_wire_outs();
    let fpga_sum = hnd.get_wire_out(0x20);
    println!("CPU sum {:x}, FPGA sum {:x}", cpu_sum, fpga_sum);
    assert_eq!(cpu_sum, fpga_sum);
    Ok(())
}
