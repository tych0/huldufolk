# Huldufólk

`huldufolk` is an implementation of the userspace side of the Linux kernel's
`CONFIG_STATIC_USERMODEHLEPER` option. Using it should make your Linux more
secure!

## Threat Model

Usermode helper binaries are always run as "real" root, and are thus an
attractive place to launch code that runs with escalated privileges.

In particular, it can be an easier way to run arbitrary code as root once some
other in-kernel exploit is successful, e.g.:

* It may be easier to call `run_cmd` or `call_usermodehelper` with a custom
  binary vs. manipulating memory protections to achieve code execution, as in
  https://googleprojectzero.blogspot.com/2018/09/.
* It's an easy way to hand flow control of a successful exploit back to
  userspace, as in https://www.openwall.com/lists/oss-security/2017/02/04/1 or
  https://www.openwall.com/lists/oss-security/2016/12/07/3

Thus, the chief threat is that the kernel may accidentally execute something it
didn't intend to as root, via the usermodehelper script. There are two ways
this can happen:

1. Memory corruption, e.g. the exploits above, or from the [original patch
   series](https://lkml.org/lkml/2017/1/16/468) by changing some of the hard
   coded strings in the kernel. The original series also a bunch of these
   strings const, but there are various mentions of tiny races which can be
   won. In any case, dispatching to a trusted userspace binary eliminates
   these.
1. Someone overwrites `/proc/sys/kernel/modprobe` or similar control file to
   cause the kernel to execute the binary they want. Usermode helper control
   files require root to be written to, but some clever userspace attacker may
   be able to figure out a way to write to this file.

In our initial design, we're mostly concerned with offering an implementation
of a static usermode-helper that is general purpose enough to be used in the
various distros. Thus, it needs to be configurable by administrators, and since
the usermode helper infrastructure doesn't allow us to export any type of
configuration via the environment, we need to use a config file.

Of course, this means that a clever attacker who can write to
`/proc/sys/kernel/modprobe` can instead just write to
`/etc/usermode-helper.conf` instead. By using a config file, we're not really
protecting against attack type two above. One could imagine an entirely static
(i.e. config file free) version of this binary down the road to solve this
problem. But also, someone who can write to arbitrary files as root already has
lots of power anyway, so it's not a high priority :)

Finally, the dispatcher can also protect against bugs in the binaries
themselves by applying the principle of least privilege. In particular,
`/sbin/modprobe` presumably doesn't need `CAP_SYS_MKNOD` or `CAP_SYS_ADMIN`, so
a dispatcher could drop these privileges before running the right binary,
protecting against bugs in the userspace code as well.

## History and Design

Linux 4.11 introduced a pair of compile time options,
`CONFIG_STATIC_USERMODEHELPER` and `CONFIG_STATIC_USERMODEHELPER_PATH`, which
when set, force all usermode helper executions to go through the path specified
(`/sbin/usermode-helper` by default), but passes the name of the original
executable as `argv[0]`, so that userspace can figure out which thing was
originally intended to be executed. While at Docker I wrote an initial
implementation of this for
(LinuxKit)[https://github.com/linuxkit/linuxkit/pull/2037], but it was not
really suitable for general purpose Linux distros. If we wanted to make it
useful, we'd need a configuration file. But since it's written in C, then we'd
have to parse stuff with that configuration file, and that would probably have
bugs.

So yes, this is written in rust. I understand that makes me a hipster, but I
already have a beard so I'm dangerously close anyway. However, rust seems like
a good choice for a low-level system utility such-as the static
usermode-helper. But, it also means that packaging it may be a pain for
distros. Today we don't have a lot of experience packaging rust binaries, and
this may be a pain point. For that reason, I've tried to keep the number of
dependencies small. However, it does mean that we'll hopefully have less bugs,
since this thing will necessarily run as root.

The basic design is that the configuration file says what's allowed to be run,
with potential additional restrictions like argument filters or dropped
capabilities, so if there's a bug in the helper itself, it can't do too much
damage.

There is also a default configuration file with comments indicating what paths
are used where, so that you can strip out things you don't want. A general
purpose distro should hopefully be able to use this config file with little
fuss.

## TODO

* argument filters (probably based on regexes?)
* setting NNP?
* Namespaces?
* seccomp filters? Is there some nice language for specifying these in config
  files? Perhaps we want to do something else?

## Name

Huldufólk are hidden elves from Icelandic folklore. One can see evidence of
their "existence" when traveling Iceland today.

> [Huldufólk] prevented many children from wandering away from human
> habitations, taught Iceland's topographical history, and instilled fear and
> respect for the harsh powers of nature.
>
> -- <cite>Ólina Thorvarðardóttir </cite>

It seems only natural to want to also prevent child processes from wondering
away :)

Plus, static-usermodehelper is boring.
