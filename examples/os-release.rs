use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use log::LevelFilter;
use simple_logger::SimpleLogger;

#[allow(unused_imports)]
use tracked_reader::tracker::DEFAULT_CHUNK;
use tracked_reader::{reader::Reader, tracker::Report};

fn inspect_buf(buf: impl AsRef<[u8]>) {
    let buf = buf.as_ref();
    log::debug!("Read got: {:?}", buf);
    log::debug!("To string: {:?}", String::from_utf8(buf.to_vec()))
}

fn inspect_reader(r: &Reader<impl Read + Seek>) {
    let t = r.tracker();
    log::debug!("Tracker: {:?}", t);
    log::info!("Report:\n{}", Report::<4>::create(t));
}

fn main() -> std::io::Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_colors(false)
        .init()
        .unwrap();
    let mut r = Reader::new(File::open("/etc/os-release")?)?;

    let mut buf = [0; 8];
    let read = r.read(&mut buf)?;
    log::trace!("Begin = [0], Read 8: {}", read);
    inspect_buf(buf);
    inspect_reader(&r);

    let seek = r.seek(SeekFrom::Start(14))?;
    log::trace!("Seek Start(14): {}", seek);
    inspect_reader(&r);

    let mut buf = [0; 2];
    let read = r.read(&mut buf)?;
    log::trace!("Start(14) = [14], Read 2: {}", read);
    inspect_buf(buf);
    inspect_reader(&r);

    let seek = r.seek(SeekFrom::Current(-2))?;
    log::trace!("Seek Current(-2): {}", seek);
    inspect_reader(&r);

    let mut buf = [0; 2];
    let read = r.read(&mut buf)?;
    log::trace!("Current(-2) = [14], Read 2: {}", read);
    inspect_buf(buf);
    inspect_reader(&r);

    let seek = r.seek(SeekFrom::End(-10))?;
    log::trace!("Seek End(-10): {}", seek);

    let mut buf = [0; 10];
    let read = r.read(&mut buf)?;
    log::trace!("End(-10) = [-10], Read 10: {}", read);
    inspect_buf(buf);
    inspect_reader(&r);

    let t = r.tracker();
    assert_eq!(Some(t.pos()), t.sz());
    inspect_reader(&r);

    Ok(())
}
