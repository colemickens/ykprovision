# ykprovision
*one-step provisioning of PGP* and *PIV with a shared-key on a Yubikey (and a backup!)*

(inspired by [`yubikey-agent -setup`](https://github.com/FiloSottile/yubikey-agent).)

## WARNING

The goal of this repo is to **soon deprecate it**, hopefully in favor of
`age` or `rage` with PIV support, along with some sort of non-GPG pass-like application.

(Though if `age` remains unable to provision duplicate yubikeys, I might consider keeping this around. OTOH the need
for dupliate keys is less important now with `sops`... thoughts?)

Anyway, **for now**, I need my passwords on Android, which means I need my decryption key in OpenPGP form still.
Hacking this together is probably faster than installing the Android dev env.

#### ideas
* write a much more user friendly version of Sops in Rust, much stricter feature set
* Use Sops instead of Pass
* Hack Android-Password-Store to remote need for 
* Create a mode where we only generate on device! and don't support GPG! so we can still be a good provisioner

## Overview

Here's a one-line command that will:

1. Configure MUK/PUK just like yubikey-agent does.
2. Create an ECDSA256 key.
3. Use yubkey-piv.rs to import key to yubikey
4. Execute GPG to import key to openpgp

Oh and it's written in Rust if you're into that.

## Feedback

ALL feedback is welcome. File an issue. Shoot me an email. cole.mickens@gmail.com

## Usage

```console
$ ykprovision

[INFO] foo...
[INFO] foo...
```

## Random

I love Rust. More than other things. Don't @
