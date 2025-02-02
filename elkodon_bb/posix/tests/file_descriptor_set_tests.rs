use elkodon_bb_container::semantic_string::SemanticString;
use elkodon_bb_posix::config::*;
use elkodon_bb_posix::file_descriptor::FileDescriptorBased;
use elkodon_bb_posix::file_descriptor_set::*;
use elkodon_bb_posix::unique_system_id::UniqueSystemId;
use elkodon_bb_posix::unix_datagram_socket::*;
use elkodon_bb_system_types::file_name::FileName;
use elkodon_bb_system_types::file_path::FilePath;
use elkodon_bb_testing::assert_that;
use elkodon_bb_testing::test_requires;
use elkodon_pal_posix::posix;
use elkodon_pal_posix::posix::POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS;
use std::time::Duration;
use std::time::Instant;

static TIMEOUT: Duration = Duration::from_millis(10);

fn generate_socket_name() -> FilePath {
    let mut file = FileName::new(b"file_descriptor_set_tests").unwrap();
    file.push_bytes(
        UniqueSystemId::new()
            .unwrap()
            .value()
            .to_string()
            .as_bytes(),
    )
    .unwrap();

    FilePath::from_path_and_file(&TEST_DIRECTORY, &file).unwrap()
}

#[test]
fn file_descriptor_set_timed_wait_blocks_at_least_timeout() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS);

    let socket_name = generate_socket_name();

    let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    let fd_set = FileDescriptorSet::new();
    let _guard = fd_set.add(&sut_receiver).unwrap();

    assert_that!(fd_set.contains(&sut_receiver), eq true);
    assert_that!(fd_set.contains(&sut_sender),  eq false);

    let start = Instant::now();

    let mut result = vec![];
    fd_set
        .timed_wait(TIMEOUT, FileEvent::Read, |fd| {
            result.push(unsafe { fd.native_handle() })
        })
        .unwrap();

    assert_that!(start.elapsed(), time_at_least TIMEOUT);
    assert_that!(result, len 0);
}

#[test]
fn file_descriptor_set_add_and_remove_works() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS);

    let fd_set = FileDescriptorSet::new();
    let mut sockets = vec![];
    let number_of_fds: usize = core::cmp::min(128, posix::FD_SETSIZE);

    for _ in 0..number_of_fds {
        let socket_name = generate_socket_name();
        sockets.push(
            UnixDatagramReceiverBuilder::new(&socket_name)
                .creation_mode(CreationMode::PurgeAndCreate)
                .create()
                .unwrap(),
        );
    }

    let mut counter = 0;
    let mut guards = vec![];
    for fd in &sockets {
        counter += 1;
        assert_that!(fd_set.contains(fd), eq false);
        let guard = fd_set.add(fd);
        assert_that!(guard, is_ok);
        guards.push(guard);
        assert_that!(fd_set.contains(fd), eq true);
        assert_that!(fd_set.len(), eq counter);
    }

    let mut counter = 0;
    for fd in sockets.iter().rev() {
        counter += 1;
        assert_that!(fd_set.contains(fd), eq true);
        guards.pop();
        assert_that!(fd_set.contains(fd), eq false);
        assert_that!(fd_set.len(), eq number_of_fds - counter);
    }
}

#[test]
fn file_descriptor_set_timed_wait_works() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS);

    let socket_name = generate_socket_name();

    let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    let fd_set = FileDescriptorSet::new();
    let _guard = fd_set.add(&sut_receiver).unwrap();
    let send_data: Vec<u8> = vec![1u8, 3u8, 3u8, 7u8, 13u8, 37u8];
    sut_sender.blocking_send(send_data.as_slice()).unwrap();

    let mut result = vec![];
    fd_set
        .timed_wait(TIMEOUT, FileEvent::Read, |fd| {
            result.push(unsafe { fd.native_handle() })
        })
        .unwrap();

    assert_that!(result, len 1);
    assert_that!(result[0], eq unsafe{sut_receiver.file_descriptor().native_handle()});
}
