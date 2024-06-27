use std::ffi::c_char;

use memdump::{Dump, FromDump};

#[derive(Dump, FromDump)]
#[repr(C)]
struct Test {
    extension_count: u32,
    #[memdump(array(len = extension_count))]
    extension_names: *const u32,
    #[memdump(c_string)]
    string: *const c_char,
}

fn main() {
    let names = [1, 2, 3, 4];
    let dump_source = Test {
        extension_count: names.len() as u32,
        extension_names: names.as_ptr(),
        string: c"string".as_ptr(),
    };

    println!(
        "dump_source.extension_count = {}",
        dump_source.extension_count
    );
    println!(
        "dump_source.extension_names = {:p}\n",
        dump_source.extension_names
    );

    let mut buf = vec![0; 27];
    dump_source.dump(&mut buf);

    println!("buf = {buf:?}");
    println!("buf.as_ptr() = {:p}\n", buf.as_ptr());

    let (from_dump, _) = Test::from_dump(&buf);
    println!("from_dump.extension_count = {}", from_dump.extension_count);
    println!(
        "from_dump.extension_names = {:p}",
        from_dump.extension_names
    );
    println!("extension_names as slice = {:?}", unsafe {
        std::slice::from_raw_parts(
            from_dump.extension_names,
            from_dump.extension_count as usize,
        )
    });
    println!("from_dump.string = {:p}", from_dump.string);
}
