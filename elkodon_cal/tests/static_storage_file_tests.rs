use elkodon_bb_container::semantic_string::*;
use elkodon_bb_posix::config::*;
use elkodon_bb_posix::directory::Directory;
use elkodon_bb_posix::file::*;
use elkodon_bb_posix::unique_system_id::UniqueSystemId;
use elkodon_bb_system_types::file_name::FileName;
use elkodon_bb_system_types::file_path::FilePath;
use elkodon_bb_testing::assert_that;
use elkodon_cal::static_storage::file::*;

fn generate_name() -> FileName {
    let mut file = FileName::new(b"communication_channel_tests_").unwrap();
    file.push_bytes(
        UniqueSystemId::new()
            .unwrap()
            .value()
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    file
}

#[test]
fn static_storage_file_custom_path_and_suffix_works() {
    let storage_name = generate_name();

    let content = "some storage content".to_string();
    let config = Configuration::default()
        .suffix(unsafe { FileName::new_unchecked(b".blubbme") })
        .path_hint(TEST_DIRECTORY);

    let storage_guard = Builder::new(&storage_name)
        .config(&config)
        .create(content.as_bytes())
        .unwrap();
    assert_that!(*storage_guard.name(), eq storage_name);

    let storage_reader = Builder::new(&storage_name).config(&config).open().unwrap();
    assert_that!(*storage_reader.name(), eq storage_name);

    let content_len = content.len() as u64;
    assert_that!(storage_reader, len content_len);

    let mut read_content = String::from_utf8(vec![b' '; content.len()]).unwrap();
    storage_reader
        .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
        .unwrap();
    assert_that!(read_content, eq content);
}

#[test]
fn static_storage_file_path_is_created_when_it_does_not_exist() {
    let storage_name = generate_name();
    let content = "some more funky content".to_string();
    let non_existing_path =
        FilePath::from_path_and_file(&TEST_DIRECTORY, &generate_name()).unwrap();

    Directory::remove(&non_existing_path.into()).ok();
    let config = Configuration::default()
        .suffix(unsafe { FileName::new_unchecked(b".blubbme") })
        .path_hint(non_existing_path.into());

    let storage_guard = Builder::new(&storage_name)
        .config(&config)
        .create(content.as_bytes());
    assert_that!(storage_guard, is_ok);

    let storage_reader = Builder::new(&storage_name).config(&config).open().unwrap();
    assert_that!(*storage_reader.name(), eq storage_name);

    let content_len = content.len() as u64;
    assert_that!(storage_reader, len content_len);

    let mut read_content = String::from_utf8(vec![b' '; content.len()]).unwrap();
    storage_reader
        .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
        .unwrap();
    assert_that!(read_content, eq content);
}

#[test]
fn static_storage_file_custom_path_and_suffix_list_storage_works() {
    const NUMBER_OF_STORAGES: u64 = 12;
    let config = Configuration::default()
        .suffix(unsafe { FileName::new_unchecked(b".blubbme") })
        .path_hint(
            FilePath::from_path_and_file(&TEST_DIRECTORY, &FileName::new(b"non_existing").unwrap())
                .unwrap()
                .into(),
        );

    let content = "some storage content".to_string();

    let mut storages = vec![];
    for _i in 0..NUMBER_OF_STORAGES {
        let storage_name = generate_name();
        storages.push(
            Builder::new(&storage_name)
                .config(&config)
                .create(content.as_bytes())
                .unwrap(),
        );
    }

    let mut some_files = vec![];
    for _i in 0..NUMBER_OF_STORAGES {
        let storage_name = FilePath::from_path_and_file(&TEST_DIRECTORY, &generate_name()).unwrap();
        FileBuilder::new(&storage_name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .create()
            .unwrap();
        some_files.push(storage_name);
    }

    let contents = Storage::list_cfg(&config).unwrap();
    assert_that!(contents, len NUMBER_OF_STORAGES as usize);

    let contains = |s| {
        for entry in &storages {
            if *entry.name() == s {
                return true;
            }
        }
        false
    };

    for entry in contents {
        assert_that!(contains(entry), eq true);
    }

    for file in &some_files {
        File::remove(file).unwrap();
    }
}
