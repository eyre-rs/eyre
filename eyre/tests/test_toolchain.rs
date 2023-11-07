#[rustversion::attr(not(nightly), ignore)]
//#[cfg_attr(miri, ignore)]
#[test]
fn nightlytest() {
    if !cfg!(nightly_features) {
        panic!("nightly feature isn't set when the toolchain is nightly");
    }
}

#[rustversion::attr(nightly, ignore)]
//#[cfg_attr(miri, ignore)]
#[test]
fn stabletest() {
    if cfg!(nightly_features) {
        panic!("nightly feature is set when the toolchain isn't nightly");
    }
}
