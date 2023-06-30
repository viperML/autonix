use std::{any::Any, ffi::OsString, os::unix::prelude::OsStringExt, path::PathBuf};

use nix_index::files::FileTreeEntry;
use tracing::{info, instrument, trace};
use tracing_test::traced_test;

#[test]
#[traced_test]
fn test_foo() {
    foo();
}

#[instrument(ret)]
fn foo() -> Box<dyn Any> {
    let database = nix_index::database::Reader::open("index-x86_64-linux").unwrap();

    let query = regex::bytes::Regex::new(r"^/bin/\w+").unwrap();

    let res = database.query(&query).run().unwrap().
    filter_map(|v| v.ok())
    .filter(|(s, e)| {
        

        todo!();
    })
    ;

    // .filter(|v| {

    //     // v.as_ref()
    //     //     .ok()
    //     //     .map_or(true, |(store_path, _)| store_path.origin().toplevel)
    // });

    for (store_path, entry) in res {
        let o = OsString::from_vec(entry.path);
        let p = PathBuf::from(o);

        trace!(?p);
        break;
    }

    // return Box::new(database);
    // Box::new(1)
    let foo = 3;
    return Box::new(0);
}
