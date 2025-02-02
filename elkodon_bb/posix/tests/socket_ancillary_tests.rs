use elkodon_bb_container::semantic_string::SemanticString;
use elkodon_bb_elementary::unique_id::*;
use elkodon_bb_posix::config::*;
use elkodon_bb_posix::file::*;
use elkodon_bb_posix::file_descriptor::*;
use elkodon_bb_posix::process::ProcessId;
use elkodon_bb_posix::socket_ancillary::*;
use elkodon_bb_system_types::file_name::FileName;
use elkodon_bb_system_types::file_path::FilePath;
use elkodon_bb_testing::assert_that;
use elkodon_bb_testing::test_requires;
use elkodon_pal_posix::posix::POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS;
use elkodon_pal_posix::posix::POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA;

fn generate_file_name() -> FilePath {
    let mut file = FileName::new(b"unix_datagram_socket_file_tests").unwrap();
    file.push_bytes(UniqueId::new().value().to_string().as_bytes())
        .unwrap();

    FilePath::from_path_and_file(&TEST_DIRECTORY, &file).unwrap()
}

struct TestFixture {
    files: Vec<FilePath>,
}

impl TestFixture {
    fn new() -> TestFixture {
        TestFixture { files: vec![] }
    }

    fn create_file(&mut self) -> File {
        let file_name = generate_file_name();
        let file = FileBuilder::new(&file_name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .create()
            .unwrap();
        self.files.push(file_name);
        file
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        for file in &self.files {
            File::remove(file).expect("failed to cleanup test file");
        }
    }
}

#[test]
fn socket_ancillary_is_empty_when_created() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS);
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA);

    let sut = SocketAncillary::new();
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, is_empty);
}

#[test]
fn socket_ancillary_credentials_work() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS);
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA);

    let mut sut = SocketAncillary::new();

    let mut credentials = SocketCred::new();
    credentials.set_pid(ProcessId::new(123));
    credentials.set_uid(456);
    credentials.set_gid(789);
    sut.set_creds(&credentials);
    assert_that!(sut.get_creds(), eq Some(credentials));
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, is_not_empty);

    credentials.set_pid(ProcessId::new(999));
    credentials.set_uid(888);
    credentials.set_gid(777);
    sut.set_creds(&credentials);
    assert_that!(sut.get_creds(), eq Some(credentials));
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, is_not_empty);

    sut.clear();
    assert_that!(sut.get_creds(), eq None);
}

#[test]
fn socket_ancillary_add_file_descriptors_work() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS);
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA);

    let mut test = TestFixture::new();
    let mut sut = SocketAncillary::new();

    for _i in 0..MAX_FILE_DESCRIPTORS_PER_MESSAGE - 1 {
        assert_that!(sut.add_fd(test.create_file().file_descriptor().clone()), eq true);
        assert_that!(sut, is_not_empty);
        assert_that!(sut.is_full(), eq false);
    }

    assert_that!(sut.add_fd(test.create_file().file_descriptor().clone()), eq true);
    assert_that!(sut, is_not_empty);
    assert_that!(sut.is_full(), eq true);

    assert_that!(sut.add_fd(test.create_file().file_descriptor().clone()), eq false);

    sut.clear();
    assert_that!(sut, is_empty);
    assert_that!(sut.is_full(), eq false);

    assert_that!(sut.add_fd(test.create_file().file_descriptor().clone()), eq true);
}
