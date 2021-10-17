use rust_hdl_ok_frontpanel_sys::OkError;

#[test]
fn test_opalkelly_xem_6010_synth_spi() {
    let mut uut = OpalKellySPITest::new::<XEM6010>();
    uut.hi.link_connect_dest();
    uut.connect_all();
    rust_hdl_test_ok_common::ok_tools::synth_obj_6010(uut, "xem_6010_spi");
}

#[test]
fn test_opalkelly_xem_6010_spi_reg_read_runtime() -> Result<(), OkError> {
    spi::test_opalkelly_spi_reg_read_runtime("xem_6010_spi/top.bit")
}

#[test]
fn test_opalkelly_spi_reg_write_xem_6010_runtime() -> Result<(), OkError> {
    spi::test_opalkelly_spi_reg_write_runtime("xem_6010_spi/top.bit")
}

#[test]
fn test_opalkelly_xem_6010_spi_single_conversion_runtime() -> Result<(), OkError> {
    spi::test_opalkelly_spi_single_conversion_runtime("xem_6010_spi/top.bit")
}
