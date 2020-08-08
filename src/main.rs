extern crate atoi;
extern crate openssl;
extern crate regex;
extern crate rpassword;
extern crate yubikey_piv as ykpiv;

const VERSION: &str = "1.0.1";
use openssl::nid::Nid;
fn main() {
    // generate private key

    let mut yk = ykpiv::yubikey::YubiKey::open().unwrap();

    if ykpiv::certificate::Certificate::read(&mut yk, ykpiv::key::SlotId::Authentication).is_ok() {
        //log.Fatal("already setup, bailing");
    }

    yk.authenticate(ykpiv::mgm::MgmKey::default()).unwrap();

    // PIV PIN/PUK Setup
    // set management key
    let mgm_key = ykpiv::mgm::MgmKey::generate().unwrap();
    mgm_key.set_protected(&mut yk).unwrap();

    // set pin
    let default_pin = [1, 2, 3, 4, 5, 6];
    let default_puk = [1, 2, 3, 4, 5, 6, 7, 8];

    let mut new_pin: String;
    let re = regex::Regex::new(r"^\d{4}$").unwrap(); //  TODO: real pin constraints?
    loop {
        new_pin = rpassword::prompt_password_stdout("Specify PIN/PUK: ").unwrap();
        if re.is_match(&new_pin) {
            break;
        }
        println!("Invalid PIN.  Please try again.")
    }

    // idk, im tired, this should work, maybe write a test
    let (new_pin, _) = {
        use atoi::FromRadix10;
        u32::from_radix_10(new_pin.as_bytes())
    };
    let new_pin = new_pin.to_be_bytes();
    yk.change_pin(&default_pin, &new_pin).unwrap();
    yk.change_puk(&default_puk, &new_pin).unwrap();

    // PIV Setup
    // create an EcKey from the binary form of a EcPoint
    let group = openssl::ec::EcGroup::from_curve_name(Nid::SECP256K1).unwrap();
    let parent_key = openssl::ec::EcKey::generate(&group).unwrap();
    let yubi_key = openssl::ec::EcKey::generate(&group).unwrap();

    let parent_pkey = openssl::pkey::PKey::from_ec_key(parent_key).unwrap();
    let yubi_pkey = openssl::pkey::PKey::from_ec_key(yubi_key.clone()).unwrap();

    let parent = {
        let mut s = openssl::x509::X509Name::builder().unwrap();
        s.append_entry_by_nid(Nid::ORGANIZATIONNAME, "ykprovision")
            .unwrap();
        s.append_entry_by_nid(Nid::ORGANIZATIONALUNITNAME, VERSION)
            .unwrap();
        let mut b = openssl::x509::X509Builder::new().unwrap();
        b.set_subject_name(&s.build()).unwrap();
        b.set_pubkey(&parent_pkey).unwrap();
        b.build()
    };

    let cert = {
        let mut s = openssl::x509::X509Name::builder().unwrap();
        s.append_entry_by_nid(Nid::COMMONNAME, "SSH Key").unwrap();

        let nb = openssl::asn1::Asn1Time::days_from_now(0).unwrap();
        let na = openssl::asn1::Asn1Time::days_from_now(42 * 365).unwrap();
        let mut sn = openssl::bn::BigNum::new().unwrap();
        sn.rand(128, openssl::bn::MsbOption::MAYBE_ZERO, false)
            .unwrap();

        let mut b = openssl::x509::X509Builder::new().unwrap();
        b.set_not_after(&na).unwrap();
        b.set_not_before(&nb).unwrap();
        b.set_serial_number(&sn.to_asn1_integer().unwrap()).unwrap();
        b.set_issuer_name(parent.issuer_name()).unwrap();
        b.set_subject_name(&s.build()).unwrap();
        b.set_pubkey(&yubi_pkey).unwrap();
        b.append_extension(
            openssl::x509::extension::KeyUsage::new()
                .key_agreement()
                .digital_signature()
                .build()
                .unwrap(),
        )
        .unwrap();

        // TODO: golang sets parent authority key id on hte yubicert
        // tbh, I think this whole thing could've been self-signed...

        b.sign(&parent_pkey, openssl::hash::MessageDigest::sha256())
            .unwrap();

        b.build()
    };

    //let key_data = yubi_key.private_key().to_asn1_integer().unwrap();
    let key_data = yubi_key.private_key().to_vec();
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

    // generate parent key+cert
    // sign + set cert

    drop(yk);

    // GPG Setup
    // - temp GNUPGHOME
    // - exec
    // - exec

    println!("All done!");
}
