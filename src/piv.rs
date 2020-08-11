use openssl::pkey::Private;

const DEFAULT_PIN: &str = "123456";
const DEFAULT_PUK: &str = "12345678";

fn encode_pin(s: &str) -> [u8; 8] {
    let mut s = s.as_bytes().to_vec();
    s.resize_with(8, || 0xff);
    let mut res = [0xff; 8];
    res.copy_from_slice(&s);
    res
}

pub fn setup(cert: openssl::x509::X509, pkey: openssl::ec::EcKey<Private>) -> Result<(), ()> {
    let default_pin = encode_pin(&DEFAULT_PIN);
    let default_puk = encode_pin(&DEFAULT_PUK);
    // open a single yk
    let mut yk = ykpiv::yubikey::YubiKey::open().unwrap();

    // <reset>
    // TODO: uh, remove this, or put behind a flag or new cmd
    println!("resetting device");
    let mut rng = rand::thread_rng();
    loop {
        let b: [u8; 8] = rand::Rng::gen(&mut rng);
        let v = yk.verify_pin(&b);
        if v.is_err() {
            if yk.get_pin_retries().unwrap() <= 0 {
                break;
            }
        }
    }
    ykpiv::yubikey::YubiKey::block_puk(&mut yk).unwrap();
    yk.reset_device().unwrap();
    println!("DONE resetting device");
    // </reset>

    // at this point default pins should work...
    println!("check default pin {:?}", &default_pin);
    yk.verify_pin(&default_pin).unwrap();
    println!("DONE check default pin");

    if ykpiv::certificate::Certificate::read(&mut yk, ykpiv::key::SlotId::Authentication).is_ok() {
        //log.Fatal("already setup, bailing");
    }
    yk.authenticate(ykpiv::mgm::MgmKey::default()).unwrap();

    // PIV PIN/PUK Setup
    // set management key
    println!("setting management key");
    let mgm = ykpiv::mgm::MgmKey::generate().unwrap();
    mgm.set(&mut yk, None).unwrap();
    println!("DONE setting management key");

    // set pin
    let mut new_pin: String;
    let re = regex::Regex::new(r"^\d{8}$").unwrap(); //  TODO: real pin constraints?
    loop {
        new_pin = rpassword::prompt_password_stdout("Specify PIN/PUK: ").unwrap();
        if re.is_match(&new_pin) {
            break;
        }
        println!("Invalid PIN.  Please try again.")
    }

    // idk, im tired, this should work, maybe write a test
    let p = encode_pin(&new_pin);
    yk.change_pin(&default_pin, &p).unwrap();
    yk.change_puk(&default_puk, &p).unwrap();

    //let key_data = yubi_key.private_key().to_asn1_integer().unwrap();
    let key_data = pkey.private_key().to_vec();
    ykpiv::key::import_ecc_key(
        &mut yk,
        ykpiv::key::SlotId::Authentication,
        ykpiv::key::AlgorithmId::EccP256,
        &key_data,
        ykpiv::policy::TouchPolicy::Always,
        ykpiv::policy::PinPolicy::Always,
    )
    .unwrap();

    let cert_der = cert.to_der().unwrap();
    let cert = ykpiv::certificate::Certificate::from_bytes(cert_der).unwrap();
    cert.write(
        &mut yk,
        ykpiv::key::SlotId::Authentication,
        ykpiv::certificate::CertInfo::Uncompressed,
    )
    .unwrap();
    yk.deauthenticate().unwrap();

    drop(yk);

    Ok(())
}
