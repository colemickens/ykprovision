extern crate atoi;
extern crate openssl;
extern crate rand;
extern crate regex;
extern crate rpassword;
extern crate yubikey_piv as ykpiv;

const VERSION: &str = "0.1.0";

// TODO: version
// TODO: logging
// TODO: VM automation

mod gpg;
mod key;
mod piv;

fn main() {
    // TODO: check entropy?
    // (hopefully openssl is doing so internally)

    let (crt, key) = key::create().unwrap();
    piv::setup(crt.clone(), key.clone()).unwrap();

    // never implemented:
    // - in retrospect, how dare I author more code that touches gpg at all
    // - better to spend effort writing a different rusty 'sops/gopass thing' and porting it to Android
    // gpg::program_openpgp(key.clone()).unwrap();

    println!("All done!");
}
