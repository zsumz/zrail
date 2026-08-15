//! Legacy linkage attributes with safety obligations.

#[export_name = "zrail_fixture"]
pub fn exposed() {}

#[link_section = ".zrail"]
pub static VALUE: u8 = 1;
