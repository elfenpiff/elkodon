#[cfg(target_os = "windows")]
mod win32_select {
    use elkodon_bb_testing::assert_that;
    use elkodon_pal_posix::posix::{settings::FD_SET_CAPACITY, *};
    use win32_handle_translator::*;

    #[test]
    fn fd_set_capacity_correct() {
        let sut = fd_set::new();
        assert_that!(FD_SET_CAPACITY, eq sut.fd_array.len());
    }

    #[test]
    fn fd_set_setting_fds_works() {
        let mut sut = fd_set::new();

        let mut socket_fd = vec![];
        for i in 0..FD_SET_CAPACITY {
            socket_fd.push(
                HandleTranslator::get_instance().add(FdHandleEntry::Socket(SocketHandle { fd: i })),
            );
        }

        for fd in socket_fd {
            assert_that!(unsafe {FD_ISSET(fd, &sut)}, eq false);
            unsafe { FD_SET(fd, &mut sut) }
            assert_that!(unsafe {FD_ISSET(fd, &sut)}, eq true);
        }
    }

    #[test]
    fn fd_set_clear_works() {
        let mut sut = fd_set::new();

        let mut socket_fd = vec![];
        for i in 0..FD_SET_CAPACITY {
            socket_fd.push(
                HandleTranslator::get_instance().add(FdHandleEntry::Socket(SocketHandle { fd: i })),
            );
        }

        for fd in &socket_fd {
            unsafe { FD_SET(*fd, &mut sut) }
        }

        unsafe { FD_ZERO(&mut sut) };

        for fd in &socket_fd {
            assert_that!(unsafe {FD_ISSET(*fd, &sut)}, eq false);
        }
    }

    #[test]
    fn fd_set_unsetting_fds_front_to_back_works() {
        let mut sut = fd_set::new();

        let mut socket_fd = vec![];
        for i in 0..FD_SET_CAPACITY {
            socket_fd.push(
                HandleTranslator::get_instance().add(FdHandleEntry::Socket(SocketHandle { fd: i })),
            );
        }

        for fd in &socket_fd {
            unsafe { FD_SET(*fd, &mut sut) }
        }

        for fd in &socket_fd {
            assert_that!(unsafe {FD_ISSET(*fd, &sut)}, eq true);
            unsafe { FD_CLR(*fd, &mut sut) }
            assert_that!(unsafe {FD_ISSET(*fd, &sut)}, eq false);
        }
    }

    #[test]
    fn fd_set_unsetting_fds_back_to_front_works() {
        let mut sut = fd_set::new();

        let mut socket_fd = vec![];
        for i in 0..FD_SET_CAPACITY {
            socket_fd.push(
                HandleTranslator::get_instance().add(FdHandleEntry::Socket(SocketHandle { fd: i })),
            );
        }

        for fd in &socket_fd {
            unsafe { FD_SET(*fd, &mut sut) }
        }

        for fd in socket_fd.iter().rev() {
            assert_that!(unsafe {FD_ISSET(*fd, &sut)}, eq true);
            unsafe { FD_CLR(*fd, &mut sut) }
            assert_that!(unsafe {FD_ISSET(*fd, &sut)}, eq false);
        }
    }
}
