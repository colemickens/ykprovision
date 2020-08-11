# ykprovision

_one-step provisioning of PIV on a Yubikey (including a backup)_

(inspired by .)

## Overview

This does the same thing as `yubikey-agent -setup` but it creates the key on your computer
and then loads it on the Yubikey.

**This is fundamentally less secure than using `yubikey-agent`!**

**If you do not fully understand the implications of this, please do not use this tool.**

That having been said, the only recommended way to use this program is with the included VM definiton, so in theory you 

Thanks to [Nix](https://nixos.org), if you have `nix` available, you can run a single command to launch this VM. Further,
due to Nix binary caches, you won't have to build the image, it will be downloaded from a cache for you.

## Usage

```shell

$ nix run --experimental-features 'nix-command flakes' \
  github:colemickens/ykprovision#vm

# this will boot a small NixOS VM and drop you in a shell

$ ykprovision --reset

$ ykprovision ...

$ sudo shutdown now

# the VM memory will be shredded
```

## Background and Future

Originally this started as an experiment to see if the same ECC keys could be used for GPG and PIV.
However, I've abandoned that idea because, I got to the point
where I had to touch GPG again and just got sad. And then upset
with myself for ever thinking this was a good idea.

For what it's worth, as I'm transitioning away from GPG, I am
realizing that it's less important to have an identical backup.
Instead, it's fairly easy to provision multiple yubikeys and 
hen use them with (*well, actually this doesn't exist yet...
see [wip project]()*).


I'm mostly leaving this up as an example of `yubikey-piv.rs`.

If nothing else it was fun to write something in Rust again; always a joy! I truly feel spoiled by Nix and Rust.
