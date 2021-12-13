use rust_hdl::bsp::ok_core::prelude::*;
use rust_hdl::bsp::ok_xem7010::*;
use rust_hdl::core::prelude::*;

mod test_common;
#[cfg(feature = "frontpanel")]
use test_common::wire::*;

#[cfg(feature = "frontpanel")]
#[test]
fn test_opalkelly_xem_7010_synth_wire() {
    let mut uut = OpalKellyWireTest::new::<XEM7010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    XEM7010::synth(uut, target_path!("xem_7010/wire"));
    test_opalkelly_xem_wire_runtime(target_path!("xem_7010/wire/top.bit")).unwrap();
}