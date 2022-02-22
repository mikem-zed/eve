use crate::linux::block::get_blk_devices;

#[test]
fn simple_cmdline() {}
#[test]
fn blr_Read() {
    use crate::linux::block;
    let res = get_blk_devices(false);
    assert!(res.is_ok())
}
