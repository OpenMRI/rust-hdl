use rust_hdl_core::prelude::*;
use crate::ucf_gen::generate_ucf;
use std::path::PathBuf;
use std::fs::{remove_dir_all, create_dir, File, copy};
use std::process::{Command};
use std::io::Write;

pub fn generate_bitstream_xem_6010<U: Block>(mut uut: U, prefix: &str, assets: &[&str], asset_dir: &str) {
    uut.connect_all();
    check_connected(&uut);
    let verilog_text = generate_verilog(&uut);
    let ucf_text = generate_ucf(&uut);
    let dir = PathBuf::from(prefix);
    let _ = remove_dir_all(&dir);
    let _ = create_dir(&dir);
    let mut v_file = File::create(dir.clone().join("top.v")).unwrap();
    write!(v_file, "{}", verilog_text).unwrap();
    let mut ucf_file = File::create(dir.clone().join("top.ucf")).unwrap();
    write!(ucf_file, "{}", ucf_text).unwrap();
    let asset_dir = PathBuf::from(asset_dir);
    for asset in assets {
        let source = asset_dir.join(asset);
        let dest = dir.clone().join(asset);
        copy(source, dest).unwrap();
    }
    let mut tcl_file = File::create(dir.clone().join("top.tcl")).unwrap();
    write!(tcl_file,
        "\
project new top.xise
project set family Spartan6
project set device xc6slx45
project set package fgg484
project set speed -3
xfile add top.v top.ucf {assets}
project set top top
process run \"Generate Programming File\" -force rerun_all
project close
",
        assets = assets.join(" ")).unwrap();
    let output = Command::new("xtclsh")
        .current_dir(dir.clone())
        .arg(dir.clone().join("top.tcl"))
        .output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let mut out_file = File::create(dir.clone().join("top.out")).unwrap();
    write!(out_file, "{}", stdout).unwrap();
    let mut err_file = File::create(dir.clone().join("top.err")).unwrap();
    write!(err_file, "{}", stderr).unwrap();
    assert!(stdout.contains(r#"Process "Generate Programming File" completed successfully"#));
    assert!(stdout.contains(r#"All constraints were met."#));
}