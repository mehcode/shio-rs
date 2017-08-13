#![allow(unused_doc_comment)]

error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }
}
