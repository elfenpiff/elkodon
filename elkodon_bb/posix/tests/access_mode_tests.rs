use elkodon_bb_posix::access_mode::*;
use elkodon_bb_testing::assert_that;
use elkodon_pal_posix::*;

#[test]
fn access_mode_prot_flag_conversion_works() {
    assert_that!(AccessMode::None.as_protflag(), eq posix::PROT_NONE);
    assert_that!(AccessMode::Read.as_protflag(), eq posix::PROT_READ);
    assert_that!(AccessMode::Write.as_protflag(), eq posix::PROT_WRITE);
    assert_that!(
        AccessMode::ReadWrite.as_protflag(), eq
        posix::PROT_READ | posix::PROT_WRITE
    );
}

#[test]
fn access_mode_o_flag_conversion_works() {
    assert_that!(AccessMode::None.as_oflag(), eq 0);
    assert_that!(AccessMode::Read.as_oflag(), eq posix::O_RDONLY);
    assert_that!(AccessMode::Write.as_oflag(), eq posix::O_WRONLY);
    assert_that!(AccessMode::ReadWrite.as_oflag(), eq posix::O_RDWR);
}
