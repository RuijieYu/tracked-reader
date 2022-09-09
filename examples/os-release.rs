use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use tracked_reader::reader::Reader;

fn inspect(buf: impl AsRef<[u8]>) {
    let buf = buf.as_ref();
    println!("Read got: {:?}", buf);
    println!("To string: {:?}", String::from_utf8(buf.to_vec()))
}

fn main() -> std::io::Result<()> {
    let mut r = Reader::new(File::open("/etc/os-release")?)?;

    let mut buf = [0; 8];
    println!("Begin = [0], Read 8: {}", r.read(&mut buf)?);
    inspect(buf);

    println!("Seek Start(14): {}", r.seek(SeekFrom::Start(14))?);

    let mut buf = [0; 2];
    println!("Start(14) = [14], Read 2: {}", r.read(&mut buf)?);
    inspect(buf);

    println!("Seek Current(-2): {}", r.seek(SeekFrom::Current(-2))?);

    let mut buf = [0; 2];
    println!("Current(-2) = [14], Read 2: {}", r.read(&mut buf)?);
    inspect(buf);

    println!("Seek End(-10): {}", r.seek(SeekFrom::End(-10))?);

    let mut buf = [0; 10];
    println!("End(-10) = [-10], Read 10: {}", r.read(&mut buf)?);
    inspect(buf);

    let t = r.tracker();
    println!("Tracker: {:?}", t);
    assert_eq!(Some(t.pos()), t.sz());

    Ok(())
}
