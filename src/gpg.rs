use openssl::pkey::Private;

pub fn program_openpgp(pkey: openssl::ec::EcKey<Private>) -> Result<(), ()> {
    Ok(())
}
