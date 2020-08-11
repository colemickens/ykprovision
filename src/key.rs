use openssl::nid::Nid;
use openssl::pkey::Private;

// todo maybe returning the pkey is better and then re-getting the type out later?
// but at this point, we ONLY provision EC keys anyway
pub fn create() -> Result<(openssl::x509::X509, openssl::ec::EcKey<Private>), ()> {
    // PIV Setup
    // create an EcKey from the binary form of a EcPoint
    let group = openssl::ec::EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let parent_key = openssl::ec::EcKey::generate(&group).unwrap();
    let yubi_key = openssl::ec::EcKey::generate(&group).unwrap();

    let parent_pkey = openssl::pkey::PKey::from_ec_key(parent_key).unwrap();
    let yubi_pkey = openssl::pkey::PKey::from_ec_key(yubi_key.clone()).unwrap();

    let parent = {
        // TBH I'm pretty sure it could've just been self-signed?
        // is this just for notoriety?
        let mut s = openssl::x509::X509Name::builder().unwrap();
        s.append_entry_by_nid(Nid::ORGANIZATIONNAME, "ykprovision")
            .unwrap();
        s.append_entry_by_nid(Nid::ORGANIZATIONALUNITNAME, crate::VERSION)
            .unwrap();
        let mut b = openssl::x509::X509Builder::new().unwrap();
        b.set_subject_name(&s.build()).unwrap();
        b.set_version(2).unwrap();
        b.set_pubkey(&parent_pkey).unwrap();
        b.build()
    };

    let cert = {
        let mut s = openssl::x509::X509Name::builder().unwrap();
        s.append_entry_by_nid(Nid::COMMONNAME, "SSH Key").unwrap();

        let nb = openssl::asn1::Asn1Time::days_from_now(0).unwrap();
        let na = openssl::asn1::Asn1Time::days_from_now(42 * 365).unwrap();
        let mut sn = openssl::bn::BigNum::new().unwrap();
        sn.rand(20, openssl::bn::MsbOption::MAYBE_ZERO, false)
            .unwrap();

        let snasn = sn.to_asn1_integer().unwrap();
        println!("snasn: {:?}", &sn);

        let mut b = openssl::x509::X509Builder::new().unwrap();
        b.set_not_after(&na).unwrap();
        b.set_not_before(&nb).unwrap();
        b.set_serial_number(&snasn).unwrap();
        b.set_issuer_name(parent.issuer_name()).unwrap();
        b.set_subject_name(&s.build()).unwrap();
        b.set_version(2).unwrap();
        b.set_pubkey(&yubi_pkey).unwrap();
        b.append_extension(
            openssl::x509::extension::KeyUsage::new()
                .key_agreement()
                .digital_signature()
                .build()
                .unwrap(),
        )
        .unwrap();

        b.sign(&parent_pkey, openssl::hash::MessageDigest::sha256())
            .unwrap();

        b.build()
    };

    Ok((cert, yubi_key))
}
